// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Adding and deleting cues around the selection, and undoing and redoing
//! changes to the show

use crate::back::{Change, Command, CueId, Place};
use crate::front::App;

impl App {
    /// Adds a cue next to the selection, on the side that `place` picks, or
    /// at the end of the list without a selection.  The new cue is
    /// selected, if the back end added it.
    pub(super) fn with_new_cue(self, place: fn(CueId) -> Place) -> Self {
        let id = self.workspace.next_cue_id();
        let add = Command::Add {
            list: self.list,
            kind: self.new_kind,
            at: self.selected_here().map_or(Place::End, place),
        };
        let app = self.send(add);
        Self {
            selected: app
                .workspace
                .cue(id)
                .map_or(app.selected, |cue| Some(cue.id)),
            ..app
        }
    }

    /// Deletes a number of cues from the selection down, as one change that
    /// undo takes back whole, and selects the cue after them, or the one
    /// before them at the end of the list.  A count past the end of the
    /// list deletes to the end.
    pub(super) fn without_selected(self, count: usize) -> Self {
        let Some(first) = self
            .selected_here()
            .and_then(|id| self.current().position(id))
        else {
            return self;
        };
        let cues = &self.current().cues;
        let ids: Vec<CueId> = cues[first..].iter().take(count).map(|cue| cue.id).collect();
        let neighbour = cues
            .get(first + ids.len())
            .or_else(|| first.checked_sub(1).and_then(|before| cues.get(before)))
            .map(|cue| cue.id);
        let app = self.send(Command::Delete(ids));
        let kept = app.selected.filter(|&id| app.workspace.cue(id).is_some());
        Self {
            selected: kept.or(neighbour),
            ..app
        }
    }

    /// Takes back the latest change to the show, and selects the cue it
    /// brought back or changed back
    pub(super) fn undone(self) -> Self {
        let cue = self.workspace.undoable().and_then(Change::cue);
        let place = self.selected_place();
        self.send(Command::Undo).showing(cue, place)
    }

    /// Makes again the change that undo took back last, and selects the cue
    /// it added or changed
    pub(super) fn redone(self) -> Self {
        let cue = self.workspace.redoable().and_then(Change::cue);
        let place = self.selected_place();
        self.send(Command::Redo).showing(cue, place)
    }

    /// Returns the row of the selected cue in the list on screen
    fn selected_place(&self) -> Option<usize> {
        self.selected_here()
            .and_then(|id| self.current().position(id))
    }

    /// Selects a cue that undo or redo touched, if it is still there.  A
    /// selected cue that they took away leaves the selection to the cue
    /// that has taken its place, as deleting does.
    fn showing(self, cue: Option<CueId>, place: Option<usize>) -> Self {
        let gone = |id: &CueId| self.workspace.cue(*id).is_none();
        match cue.filter(|id| !gone(id)) {
            Some(id) => self.with_selection(id),
            None if self.selected.as_ref().is_some_and(gone) => {
                let cues = &self.current().cues;
                let selected = place
                    .and_then(|row| cues.get(row).or(cues.last()))
                    .map(|cue| cue.id);
                Self { selected, ..self }
            }
            None => self,
        }
    }
}

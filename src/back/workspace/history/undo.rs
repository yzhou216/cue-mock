// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Undo and redo on a workspace, which take changes back and make them
//! again without recording them as new changes

use super::{Change, Placed, Turn};
use crate::back::CueId;
use crate::back::workspace::Workspace;

impl Workspace {
    /// Returns the change that undo takes back next.  Show mode, which
    /// locks the cues, leaves only moves of standby to undo.
    pub fn undoable(&self) -> Option<&Change> {
        self.history
            .undo
            .last()
            .filter(|change| self.allows(change))
    }

    /// Returns the change that redo makes again next, which show mode
    /// limits as it does undo
    pub fn redoable(&self) -> Option<&Change> {
        self.history
            .redo
            .last()
            .filter(|change| self.allows(change))
    }

    /// Returns the way the latest change to the show went, with the change
    /// that undo or redo acts on now
    pub fn latest_change(&self) -> Option<(Turn, &Change)> {
        self.history.latest()
    }

    pub(in crate::back::workspace) fn undone(self) -> Self {
        if self.undoable().is_none() {
            return self;
        }
        let (history, change) = self.history.undoing();
        let workspace = Self { history, ..self };
        match change {
            Some(change) => workspace.reverted(change),
            None => workspace,
        }
    }

    pub(in crate::back::workspace) fn redone(self) -> Self {
        if self.redoable().is_none() {
            return self;
        }
        let (history, change) = self.history.redoing();
        let workspace = Self { history, ..self };
        match change {
            Some(change) => workspace.remade(change),
            None => workspace,
        }
    }

    /// Returns the workspace with a change made to it recorded
    pub(in crate::back::workspace) fn recorded(self, change: Change) -> Self {
        Self {
            history: self.history.recorded(change),
            ..self
        }
    }

    /// Returns the workspace after GO has moved the standby of a list on,
    /// so that undo no longer moves it back
    pub(in crate::back::workspace) fn played_from(self, list: usize) -> Self {
        Self {
            history: self.history.without_moves_of(list),
            ..self
        }
    }

    fn allows(&self, change: &Change) -> bool {
        !self.show_mode || matches!(change, Change::Standby(_))
    }

    fn reverted(self, change: Change) -> Self {
        match change {
            Change::Added(placed) => self.removed(&[placed.cue.id]),
            Change::Deleted { cues, standby } => {
                let restored = cues.into_iter().fold(self, Workspace::with_placed);
                standby.into_iter().fold(restored, |workspace, standby| {
                    workspace.with_standby_from(standby.list, standby.after, standby.before)
                })
            }
            Change::Edited { cue, undo, .. } => self.changed(cue, undo),
            Change::Standby(standby) => {
                self.with_standby_from(standby.list, standby.after, standby.before)
            }
        }
    }

    fn remade(self, change: Change) -> Self {
        match change {
            Change::Added(placed) => self.with_placed(placed),
            Change::Deleted { cues, .. } => {
                let ids: Vec<CueId> = cues.iter().map(|placed| placed.cue.id).collect();
                self.removed(&ids)
            }
            Change::Edited { cue, redo, .. } => self.changed(cue, redo),
            Change::Standby(standby) => {
                self.with_standby_from(standby.list, standby.before, standby.after)
            }
        }
    }

    /// Puts a cue back at its place in its list
    fn with_placed(self, placed: Placed) -> Self {
        self.with_list(placed.list, |list| {
            list.with_cue_at(placed.index, placed.cue)
        })
    }

    /// Moves the standby of a list from `from` to `to`, unless something
    /// has moved it since
    fn with_standby_from(self, list: usize, from: Option<CueId>, to: Option<CueId>) -> Self {
        let unmoved = self
            .lists
            .get(list)
            .is_some_and(|cues| cues.standby == from);
        if unmoved {
            self.with_standby(list, to)
        } else {
            self
        }
    }
}

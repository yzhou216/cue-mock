// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes to the cue lists: standby, loading, and the edits that show
//! mode locks, which [`super::history`] records for undo

use std::collections::BTreeSet;

use super::Workspace;
use super::history::{Change, Placed, Standby};
use crate::back::items::replaced;
use crate::back::{Active, CueId, CueList, Edit, Kind, Media, Place};

/// Returns the next whole number above every numeric cue number, which
/// keeps new cue numbers apart across all lists as QLab does.  Numbers too
/// large to count up from leave the next number at the largest there is.
fn next_number<'a>(numbers: impl Iterator<Item = &'a str>) -> String {
    let highest = numbers
        .filter_map(|number| number.trim().parse::<f64>().ok())
        .fold(0.0_f64, f64::max);
    (highest.floor() as u64).saturating_add(1).to_string()
}

impl Workspace {
    pub(super) fn with_standby(self, list: usize, standby: Option<CueId>) -> Self {
        self.with_list(list, |cues| cues.with_standby(standby))
    }

    /// Moves standby as the operator asks, recording the move unless
    /// standby is there already
    pub(super) fn moved_standby(self, list: usize, cue: Option<CueId>) -> Self {
        let before = self.lists.get(list).map(|cues| cues.standby);
        let workspace = self.with_standby(list, cue);
        let after = workspace.lists.get(list).map(|cues| cues.standby);
        match before.zip(after) {
            Some((before, after)) if before != after => {
                workspace.recorded(Change::Standby(Standby {
                    list,
                    before,
                    after,
                }))
            }
            _ => workspace,
        }
    }

    /// Adds a cue with the next identity, numbered after the highest cue of
    /// any list
    pub(super) fn with_new_cue(self, list: usize, kind: Kind, at: Place) -> Self {
        let id = self.next_cue_id();
        let number = next_number(self.cues().map(|cue| cue.number.as_str()));
        let workspace = Self {
            next_id: id.0 + 1,
            ..self.with_list(list, |cues| cues.with_new_cue(id, number, kind, at))
        };
        match workspace.place_of(id) {
            Some(placed) => workspace.recorded(Change::Added(placed)),
            None => workspace,
        }
    }

    /// Deletes cues as one change, with the places they had and where
    /// standby was before and after
    pub(super) fn without(self, ids: &[CueId]) -> Self {
        // Cues in the order of their places, which is the order undo puts
        // them back in
        let cues = {
            let mut cues: Vec<Placed> = ids.iter().filter_map(|&id| self.place_of(id)).collect();
            cues.sort_by_key(|placed| (placed.list, placed.index));
            cues.dedup_by_key(|placed| (placed.list, placed.index));
            cues
        };
        let lists: BTreeSet<usize> = cues.iter().map(|placed| placed.list).collect();
        let before: Vec<(usize, Option<CueId>)> = lists
            .into_iter()
            .map(|list| (list, self.lists[list].standby))
            .collect();
        let workspace = self.removed(ids);
        let standby = before
            .into_iter()
            .map(|(list, before)| Standby {
                list,
                before,
                after: workspace.lists[list].standby,
            })
            .collect();
        if cues.is_empty() {
            workspace
        } else {
            workspace.recorded(Change::Deleted { cues, standby })
        }
    }

    /// Edits a cue, recording the edit unless it leaves the setting as it
    /// was
    pub(super) fn edited(self, id: CueId, edit: Edit) -> Self {
        let undo = self
            .cue(id)
            .map(|cue| cue.reverting(&edit))
            .filter(|undo| *undo != edit);
        let workspace = self.changed(id, edit.clone());
        match undo {
            Some(undo) => workspace.recorded(Change::Edited {
                cue: id,
                undo,
                redo: edit,
            }),
            None => workspace,
        }
    }

    /// Removes cues from their lists and stops those that are running
    pub(super) fn removed(self, ids: &[CueId]) -> Self {
        ids.iter().fold(self, |workspace, &id| {
            workspace
                .with_list_of(id, |list| list.without(id))
                .stopped(id)
        })
    }

    /// Edits a cue.  A running audio cue stops when it gets a new file, and
    /// it takes a new level at once, which ends a fade on it.  A cue that a
    /// panic or a fade is stopping keeps fading out, so that no edit brings
    /// it back.
    pub(super) fn changed(self, id: CueId, edit: Edit) -> Self {
        let audio = self.cue(id).is_some_and(|cue| cue.kind == Kind::Audio);
        let workspace = match edit {
            Edit::File(_) => self.stopped(id),
            Edit::Level(level) if audio => self.with_active(id, |active| match active.fade {
                Some(fade) if fade.then_stop => active,
                _ => Active {
                    level,
                    fade: None,
                    ..active
                },
            }),
            _ => self,
        };
        workspace.with_list_of(id, |list| list.edited(id, edit))
    }

    /// Records what loading the file of a cue found, if the cue still has
    /// that file.  A run whose sound could not start then ends, and the
    /// rest of its chain plays on, as GO skips a broken cue.
    pub(super) fn loaded(self, id: CueId, file: &str, media: Media) -> Self {
        let current = self.cue(id).is_some_and(|cue| cue.file == file);
        let workspace = match media {
            Media::Failed(_) if current => self.skipped(id),
            _ => self,
        };
        workspace.with_list_of(id, |list| list.loaded(id, file, media))
    }

    /// Returns a cue with its place in the lists
    fn place_of(&self, id: CueId) -> Option<Placed> {
        self.lists.iter().enumerate().find_map(|(list, cues)| {
            cues.position(id).map(|index| Placed {
                list,
                index,
                cue: cues.cues[index].clone(),
            })
        })
    }

    /// Changes the list that holds a cue, or nothing when no list holds it
    fn with_list_of(self, id: CueId, change: impl FnOnce(CueList) -> CueList) -> Self {
        match self.list_of(id) {
            Some(index) => self.with_list(index, change),
            None => self,
        }
    }

    pub(super) fn with_list(self, index: usize, change: impl FnOnce(CueList) -> CueList) -> Self {
        Self {
            lists: replaced(self.lists, index, change),
            ..self
        }
    }
}

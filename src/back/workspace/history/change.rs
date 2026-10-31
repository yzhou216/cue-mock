// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the show records, which is only what it touched

use std::mem::{self, Discriminant};

use crate::back::{Cue, CueId, Edit};

/// Change to the show that the operator made
#[derive(Debug, Clone)]
pub enum Change {
    Added(Placed),
    /// Cues deleted, with the places they had in order, and the standby of
    /// each list they left
    Deleted {
        cues: Vec<Placed>,
        standby: Vec<Standby>,
    },
    /// One setting of a cue edited, with the edit that takes it back and
    /// the edit that makes it again
    Edited {
        cue: CueId,
        undo: Edit,
        redo: Edit,
    },
    /// Standby of a list moved by the operator
    Standby(Standby),
}

/// Cue at its place in a list
#[derive(Debug, Clone)]
pub struct Placed {
    pub list: usize,
    pub index: usize,
    pub cue: Cue,
}

/// Standby of a list before and after a change moved it
#[derive(Debug, Clone)]
pub struct Standby {
    pub list: usize,
    pub before: Option<CueId>,
    pub after: Option<CueId>,
}

/// Way that the latest change went
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Done,
    Undone,
    Redone,
}

impl Change {
    /// Returns the first cue that the change added, deleted, or edited
    pub fn cue(&self) -> Option<CueId> {
        match self {
            Change::Added(placed) => Some(placed.cue.id),
            Change::Deleted { cues, .. } => cues.first().map(|placed| placed.cue.id),
            Change::Edited { cue, .. } => Some(*cue),
            Change::Standby(_) => None,
        }
    }

    /// Returns whether `next` edits the setting that this change edited, so
    /// that the two make one change
    pub(super) fn takes(&self, next: &Change) -> bool {
        self.setting()
            .is_some_and(|setting| next.setting() == Some(setting))
    }

    /// Returns this change followed by `next` as one change, which takes
    /// back to before this one and makes again what `next` made
    pub(super) fn then(self, next: Change) -> Change {
        match (self, next) {
            (Change::Edited { cue, undo, .. }, Change::Edited { redo, .. }) => {
                Change::Edited { cue, undo, redo }
            }
            (_, next) => next,
        }
    }

    pub(super) fn moves_standby_of(&self, list: usize) -> bool {
        matches!(self, Change::Standby(standby) if standby.list == list)
    }

    /// Returns the cue and the setting that an edit changed
    fn setting(&self) -> Option<(CueId, Discriminant<Edit>)> {
        match self {
            Change::Edited { cue, redo, .. } => Some((*cue, mem::discriminant(redo))),
            _ => None,
        }
    }
}

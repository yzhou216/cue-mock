// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue lists, each with its own standby position

use crate::back::items::{inserted, replaced};
use crate::back::{Cue, CueId, Edit, Kind, Media, Place};

/// Named list of cues with its own standby position
#[derive(Debug, Clone)]
pub struct CueList {
    pub name: String,
    pub cues: Vec<Cue>,
    /// Cue the next GO fires, or `None` past the end of the list
    pub standby: Option<CueId>,
}

impl CueList {
    /// Creates a list with standby on its first cue
    pub fn new(name: &str, cues: Vec<Cue>) -> Self {
        let standby = cues.first().map(|cue| cue.id);
        Self {
            name: name.to_owned(),
            cues,
            standby,
        }
    }

    pub fn position(&self, id: CueId) -> Option<usize> {
        self.cues.iter().position(|cue| cue.id == id)
    }

    pub fn after(&self, id: CueId) -> Option<CueId> {
        self.position(id)
            .and_then(|index| self.cues.get(index + 1))
            .map(|cue| cue.id)
    }

    /// Returns the cue `offset` rows away from `from`, clamped to the list,
    /// or the first cue when `from` is not in this list
    pub fn step(&self, from: Option<CueId>, offset: isize) -> Option<CueId> {
        let last = self.cues.len().checked_sub(1)?;
        let index = from
            .and_then(|id| self.position(id))
            .map_or(0, |index| index.saturating_add_signed(offset).min(last));
        self.cues.get(index).map(|cue| cue.id)
    }

    /// Returns the cues one GO on `start` fires, each paired with its delay
    /// after the GO.  The chain follows auto-continue and auto-follow until
    /// it reaches a cue that waits for GO.
    pub fn chain(&self, start: CueId) -> impl Iterator<Item = (&Cue, f32)> {
        let from = self.position(start).unwrap_or(self.cues.len());
        self.cues[from..].iter().scan(Some(0.0), |next, cue| {
            let delay = (*next)?;
            *next = cue.next_delay().map(|gap| delay + gap);
            Some((cue, delay))
        })
    }

    /// Moves standby to a cue, treating a cue of another list as the end
    pub(in crate::back) fn with_standby(self, standby: Option<CueId>) -> Self {
        let standby = standby.filter(|&id| self.position(id).is_some());
        Self { standby, ..self }
    }

    /// Inserts a cue with a number and a name at a place in the list.  As
    /// in QLab, a new fade or stop cue targets the cue it follows when it
    /// can act on that cue.
    pub(in crate::back) fn with_new_cue(
        self,
        id: CueId,
        number: String,
        kind: Kind,
        at: Place,
    ) -> Self {
        let index = match at {
            Place::After(cue) => self.position(cue).map(|index| index + 1),
            Place::Before(cue) => self.position(cue),
            Place::End => None,
        }
        .unwrap_or(self.cues.len());
        let cue = Cue::new(id, number, kind, format!("New {kind} cue")).build();
        let target = index
            .checked_sub(1)
            .and_then(|before| self.cues.get(before))
            .filter(|before| cue.can_target(before))
            .map(|before| before.id);
        Self {
            cues: inserted(self.cues, index, Cue { target, ..cue }),
            ..self
        }
    }

    /// Removes a cue.  Standby on that cue moves on to the next cue, or
    /// past the end after the last cue, so that GO never goes back to a cue
    /// that has played.
    pub(in crate::back) fn without(self, id: CueId) -> Self {
        let standby = if self.standby == Some(id) {
            self.after(id)
        } else {
            self.standby
        };
        let cues = self.cues.into_iter().filter(|cue| cue.id != id).collect();
        Self {
            cues,
            standby,
            ..self
        }
    }

    /// Puts a cue back at an index, or at the end when the list has grown
    /// shorter
    pub(in crate::back) fn with_cue_at(self, index: usize, cue: Cue) -> Self {
        let index = index.min(self.cues.len());
        Self {
            cues: inserted(self.cues, index, cue),
            ..self
        }
    }

    pub(in crate::back) fn edited(self, id: CueId, edit: Edit) -> Self {
        self.with_cue(id, |cue| cue.edited(edit))
    }

    pub(in crate::back) fn loaded(self, id: CueId, file: &str, media: Media) -> Self {
        self.with_cue(id, |cue| cue.loaded(file, media))
    }

    fn with_cue(self, id: CueId, change: impl FnOnce(Cue) -> Cue) -> Self {
        let cues = match self.position(id) {
            Some(index) => replaced(self.cues, index, change),
            None => self.cues,
        };
        Self { cues, ..self }
    }
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes that the operator makes to one setting of a cue

use super::{Advance, Cue, CueId, Media, Seconds};

/// Change to one setting of a cue
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Number(String),
    Name(String),
    File(String),
    /// Cue that a fade or stop cue acts on, or none
    Target(Option<CueId>),
    Notes(String),
    PreWait(Seconds),
    Duration(Seconds),
    PostWait(Seconds),
    Advance(Advance),
    /// Level in dB
    Level(f32),
    StopTarget(bool),
}

impl Cue {
    /// Returns the cue with one edit applied.  A new file has not been
    /// loaded yet.
    pub fn edited(self, edit: Edit) -> Self {
        match edit {
            Edit::Number(number) => Self { number, ..self },
            Edit::Name(name) => Self { name, ..self },
            Edit::File(file) => Self {
                file,
                media: Media::Unloaded,
                ..self
            },
            Edit::Target(target) => Self { target, ..self },
            Edit::Notes(notes) => Self { notes, ..self },
            Edit::PreWait(pre_wait) => Self { pre_wait, ..self },
            Edit::Duration(duration) => Self { duration, ..self },
            Edit::PostWait(post_wait) => Self { post_wait, ..self },
            Edit::Advance(advance) => Self { advance, ..self },
            Edit::Level(level) => Self { level, ..self },
            Edit::StopTarget(stop_target) => Self {
                stop_target,
                ..self
            },
        }
    }

    /// Returns the edit that sets what `edit` changes back to its value in
    /// this cue, which is how undo takes an edit back
    pub fn reverting(&self, edit: &Edit) -> Edit {
        match edit {
            Edit::Number(_) => Edit::Number(self.number.clone()),
            Edit::Name(_) => Edit::Name(self.name.clone()),
            Edit::File(_) => Edit::File(self.file.clone()),
            Edit::Target(_) => Edit::Target(self.target),
            Edit::Notes(_) => Edit::Notes(self.notes.clone()),
            Edit::PreWait(_) => Edit::PreWait(self.pre_wait.clone()),
            Edit::Duration(_) => Edit::Duration(self.duration.clone()),
            Edit::PostWait(_) => Edit::PostWait(self.post_wait.clone()),
            Edit::Advance(_) => Edit::Advance(self.advance),
            Edit::Level(_) => Edit::Level(self.level),
            Edit::StopTarget(_) => Edit::StopTarget(self.stop_target),
        }
    }
}

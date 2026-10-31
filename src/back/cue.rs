// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue data model, with the cue lists that hold cues

mod edit;
mod kind;
mod list;
mod media;
mod seconds;

use bon::Builder;
use derive_more::Display;

pub use edit::Edit;
pub use kind::{Advance, Kind};
pub use list::CueList;
pub use media::{Media, Problem};
pub use seconds::Seconds;

/// Level in dB that counts as silence, which is the bottom of every level
pub const SILENCE: f32 = -60.0;

/// Identity of a cue that stays stable when the operator renumbers it
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CueId(pub u32);

/// One cue of a cue list, which displays as its number and name
///
/// [`Cue::new`] takes the settings every cue needs and returns a builder
/// for the rest, which `build` finishes.  By default a cue has no target,
/// no waits, and a manual advance.  Fade and wait cues last five seconds,
/// and fade cues fade their target out and stop it, which is what most
/// fades do.
#[derive(Debug, Clone, PartialEq, Builder, Display)]
#[builder(start_fn = new)]
#[display("{number} {name}")]
pub struct Cue {
    #[builder(start_fn)]
    pub id: CueId,
    #[builder(start_fn, into)]
    pub number: String,
    #[builder(start_fn)]
    pub kind: Kind,
    #[builder(start_fn, into)]
    pub name: String,
    /// Sound file an audio cue plays
    #[builder(default, into)]
    pub file: String,
    /// What loading the file found
    #[builder(skip)]
    pub media: Media,
    /// Cue a fade or stop cue acts on
    pub target: Option<CueId>,
    #[builder(default, into)]
    pub pre_wait: Seconds,
    /// Length of the action of a fade or wait cue
    #[builder(default = if kind.has_duration() { 5.0 } else { 0.0 }, into)]
    pub duration: Seconds,
    #[builder(default, into)]
    pub post_wait: Seconds,
    #[builder(default)]
    pub advance: Advance,
    #[builder(default, into)]
    pub notes: String,
    /// Level in dB an audio cue plays at, or a fade cue fades its target to
    #[builder(default = if kind == Kind::Fade { SILENCE } else { 0.0 })]
    pub level: f32,
    /// Whether a fade cue stops its target once the fade is over
    #[builder(default = kind == Kind::Fade)]
    pub stop_target: bool,
}

impl Cue {
    /// Returns the length of the cue's action.  It is zero for instant cues
    /// and for audio cues whose file has not loaded.
    pub fn action(&self) -> f32 {
        match (self.kind, &self.media) {
            (Kind::Audio, Media::Loaded { duration }) => *duration,
            (Kind::Fade | Kind::Wait, _) => self.duration.value(),
            _ => 0.0,
        }
    }

    /// Returns whether this cue can act on `other`.  Fade cues act on audio
    /// cues, stop cues on any other cue, and the remaining kinds on none.
    pub fn can_target(&self, other: &Cue) -> bool {
        other.id != self.id
            && match self.kind {
                Kind::Fade => other.kind == Kind::Audio,
                Kind::Stop => true,
                Kind::Audio | Kind::Wait | Kind::Memo => false,
            }
    }

    /// Returns the delay between this cue firing and it firing the next cue,
    /// or `None` when the next cue waits for GO
    pub fn next_delay(&self) -> Option<f32> {
        match self.advance {
            Advance::Manual => None,
            Advance::Continue => Some(self.pre_wait.value() + self.post_wait.value()),
            Advance::Follow => Some(self.pre_wait.value() + self.action()),
        }
    }

    /// Returns the cue with what loading `file` found, unless the cue has
    /// been given another file since loading began
    pub fn loaded(self, file: &str, media: Media) -> Self {
        if self.file == file {
            Self { media, ..self }
        } else {
            self
        }
    }
}

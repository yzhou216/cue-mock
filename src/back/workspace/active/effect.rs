// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What fired cues do once their pre-wait is over, and the fades they make

use crate::back::CueId;

/// Change to the level of an audio cue that a fade cue or a panic makes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    /// Fade cue making the change, or `None` for a panic
    pub by: Option<CueId>,
    /// Level in dB the fade ends at
    pub to: f32,
    /// Seconds until the fade ends
    pub remaining: f64,
    /// Whether the faded cue stops once the fade ends
    pub then_stop: bool,
}

impl Fade {
    /// Returns whether a newer fade may take over from this one.  Nothing
    /// takes over from a panic, and only a fade that stops the cue takes
    /// over from another that stops it, so that no fade brings back a cue
    /// on its way out.
    pub(in crate::back) fn yields_to(&self, newer: &Fade) -> bool {
        self.by.is_some() && (newer.then_stop || !self.then_stop)
    }
}

/// What a fired cue does once its pre-wait is over
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::back) enum Effect {
    /// Plays the sound of an audio cue
    Sound,
    /// Fades the level of a cue, and stops it afterwards when `then_stop`
    /// is set
    Fade {
        target: CueId,
        to: f32,
        then_stop: bool,
    },
    /// Stops a cue
    Stop(CueId),
    /// Takes time and does nothing else, as wait and memo cues do
    Nothing,
}

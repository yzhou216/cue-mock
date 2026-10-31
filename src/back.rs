// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Show model and audio playback, with no knowledge of any interface
//!
//! A front end reads a [`Workspace`] and changes it only by applying a
//! [`Command`], so every rule about what may change lives here.  Every
//! change consumes the workspace and returns a new one, without side
//! effects.  [`Audio`] is the one part with side effects: it reads the files
//! that cues name and makes the sound that the workspace describes.

mod audio;
mod cue;
mod items;
pub mod sample;
mod workspace;

pub use audio::{Audio, load_files};
pub use cue::{Advance, Cue, CueId, CueList, Edit, Kind, Media, Problem, SILENCE};
pub use workspace::{Active, Change, Command, Phase, Place, Turn, Workspace};

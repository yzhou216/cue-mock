// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Show model and audio playback, with no knowledge of any interface
//!
//! A front end reads a [`Workspace`] and changes it only by applying a
//! [`Command`], so every rule about what may change lives here.  Every
//! change consumes the workspace and returns a new one, without side
//! effects.  [`Audio`], which makes the sound that the workspace describes,
//! and [`load_files`], which reads the files that cues name, are the parts
//! with side effects.

mod audio;
mod cue;
mod items;
mod workspace;

pub use audio::{Audio, Load, SOUND_FILES, load_files};
pub use cue::{
    Advance, Cue, CueId, CueList, Edit, Kind, LOUDEST, Media, Problem, SILENCE, parse_level,
};
pub use workspace::{Active, Change, Command, Phase, Place, Turn, Workspace};

use cue::Clip;

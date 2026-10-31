// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the rules that a workspace applies, with the helpers that
//! build the shows they run on

mod editing;
mod history;
mod playback;

use super::Workspace;
use super::playback::PANIC_FADE;
use crate::back::{
    Advance, Command, Cue, CueId, CueList, Edit, Kind, Media, Phase, Place, Problem, SILENCE,
};

/// Returns a workspace with one list of cues, every audio file of which
/// has loaded as ten seconds of sound
fn workspace(cues: Vec<Cue>) -> Workspace {
    let workspace = Workspace::new(vec![CueList::new("Main", cues)]);
    let loads: Vec<Command> = workspace
        .unloaded()
        .map(|(cue, file)| Command::Loaded {
            cue,
            file: file.to_owned(),
            media: Media::Loaded { duration: 10.0 },
        })
        .collect();
    loads.into_iter().fold(workspace, Workspace::apply)
}

fn audio(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Audio, "Sound")
        .file("sound.ogg")
        .build()
}

/// Returns an audio cue that auto-continues, so the next cue fires with it
fn continuing(id: u32) -> Cue {
    Cue {
        advance: Advance::Continue,
        ..audio(id)
    }
}

/// Returns an audio cue that auto-continues after a post-wait
fn continuing_after(id: u32, post_wait: f32) -> Cue {
    Cue {
        post_wait: post_wait.into(),
        ..continuing(id)
    }
}

/// Returns a fade cue that fades cue 1 out over four seconds and stops it
fn fade_out(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Fade, "Fade")
        .target(CueId(1))
        .duration(4.0)
        .build()
}

fn level(workspace: &Workspace, id: u32) -> Option<f32> {
    workspace.active_of(CueId(id)).map(|active| active.level)
}

const GO: Command = Command::Go { list: 0 };

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of loading the files of audio cues, which needs no audio device

use super::{Load, load_files};
use crate::back::{Command, Cue, CueId, CueList, Kind, Media, Workspace};

#[test]
fn every_cue_that_names_a_missing_file_learns_that_it_is_not_found() {
    let named = |id| {
        Cue::new(CueId(id), id.to_string(), Kind::Audio, "Sound")
            .file("gone.ogg")
            .build()
    };
    let workspace =
        Workspace::new(vec![CueList::new("Main", vec![named(1), named(2)])]).directory("/nowhere");
    let found = load_files(Load::all(&workspace, workspace.unloaded()));
    let not_found = |id| {
        found.iter().any(|command| {
            matches!(command, Command::Loaded { cue, media: Media::Failed(reason), .. }
                if *cue == CueId(id) && reason == "file not found")
        })
    };
    assert!(not_found(1) && not_found(2), "{found:?}");
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text for back-end values, kept apart from the widgets that show it

use std::path::Path;

use itertools::Itertools;

use crate::back::{Active, Advance, Change, Cue, CueList, Edit, Kind, Media, Phase, Workspace};

/// Formats seconds as `m:ss.s`, the way the cue list shows times
pub fn clock(seconds: f64) -> String {
    let tenths = (seconds.max(0.0) * 10.0).round() as u32;
    format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

/// Formats a pre-wait or post-wait, leaving zero blank so real waits stand out
pub fn wait(seconds: f64) -> String {
    if seconds > 0.0 {
        clock(seconds)
    } else {
        String::new()
    }
}

/// Formats the length of a cue's action: the length of the file for an
/// audio cue, a question mark while that is unknown, and a dash for instant
/// cues
pub fn action(cue: &Cue) -> String {
    match (cue.kind, &cue.media) {
        (Kind::Audio, Media::Loaded { duration }) => clock((*duration).into()),
        (Kind::Audio, _) => "?".to_owned(),
        (Kind::Fade | Kind::Wait, _) => clock(cue.action().into()),
        (Kind::Stop | Kind::Memo, _) => "-".to_owned(),
    }
}

/// Formats the post-wait, leaving it blank for auto-follow, which ignores it
pub fn post_wait(cue: &Cue) -> String {
    if cue.advance == Advance::Follow {
        String::new()
    } else {
        wait(cue.post_wait.value().into())
    }
}

/// Returns the mark of the Next column, which only the standby cue has
pub fn next(list: &CueList, cue: &Cue) -> &'static str {
    if list.standby == Some(cue.id) {
        "NEXT"
    } else {
        ""
    }
}

/// Returns the short label of the Then column, blank for manual cues
pub fn advance(advance: Advance) -> &'static str {
    match advance {
        Advance::Manual => "",
        Advance::Continue => "continue",
        Advance::Follow => "follow",
    }
}

/// Describes what a cue acts on: the file name of an audio cue, or the
/// number of the cue that a fade or stop cue targets
pub fn target(workspace: &Workspace, cue: &Cue) -> String {
    match cue.kind {
        Kind::Audio => Path::new(cue.file.trim())
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
        Kind::Fade | Kind::Stop => cue
            .target
            .and_then(|id| workspace.cue(id))
            .map_or_else(String::new, |target| target.number.clone()),
        Kind::Wait | Kind::Memo => String::new(),
    }
}

/// Formats a level in dB with its sign
pub fn level(db: f32) -> String {
    format!("{db:+.1} dB")
}

/// Counts things by name, as in `1 cue` and `2 cues`
pub fn count(number: usize, noun: &str) -> String {
    let plural = if number == 1 { "" } else { "s" };
    format!("{number} {noun}{plural}")
}

/// Describes where an active cue is in its lifetime
pub fn phase(active: &Active) -> String {
    let phase = match active.phase() {
        Phase::Scheduled { starts_in } => format!("in {starts_in:.1}s"),
        Phase::PreWait { remaining } => format!("pre-wait {remaining:.1}s"),
        Phase::Running { remaining } => format!("{} left", clock(remaining)),
    };
    if active.paused {
        format!("paused, {phase}")
    } else {
        phase
    }
}

/// Describes an active cue in full for the list of running cues: where it
/// is in its lifetime, whether a fade is changing its level, and the level
/// of an audio cue
pub fn activity(active: &Active, cue: &Cue) -> String {
    let fading = if active.fade.is_some() {
        ", fading"
    } else {
        ""
    };
    let level = if cue.kind == Kind::Audio {
        format!(", {}", level(active.level))
    } else {
        String::new()
    };
    format!("{}{fading}{level}", phase(active))
}

/// Describes every cue of the chain that the next GO starts, so that
/// auto-continue and auto-follow chains are visible before they run.  The
/// count leaves out the broken cues, which GO skips.
pub fn chain(workspace: &Workspace, list: &CueList, standby: &Cue) -> String {
    let cues: Vec<(&Cue, f32, bool)> = list
        .chain(standby.id)
        .map(|(cue, delay)| (cue, delay, workspace.problem(cue).is_some()))
        .collect();
    let fired = cues.iter().filter(|(_, _, broken)| !broken).count();
    let described = cues.iter().map(|(cue, delay, broken)| {
        let skipped = if *broken { ", broken, skipped" } else { "" };
        format!("{cue} (+{delay:.1}s{skipped})")
    });
    format!(
        "GO fires {}: {}",
        count(fired, "cue"),
        described.format(", ")
    )
}

/// Describes a change to the show in few words, starting in lower case, as
/// the status bar tells what undo or redo acts on
pub fn change(workspace: &Workspace, change: &Change) -> String {
    match change {
        Change::Added(placed) => format!("added cue {}", number(&placed.cue)),
        Change::Deleted { cues, .. } => match cues.as_slice() {
            [placed] => format!("deleted cue {}", number(&placed.cue)),
            cues => format!("deleted {} cues", cues.len()),
        },
        Change::Edited { cue, redo, .. } => {
            let cue = workspace.cue(*cue).map_or_else(String::new, number);
            match redo {
                Edit::Number(_) => format!("changed the number of cue {cue}"),
                Edit::Name(_) => format!("changed the name of cue {cue}"),
                Edit::File(_) => format!("changed the file of cue {cue}"),
                Edit::Target(_) => format!("changed the target of cue {cue}"),
                Edit::Notes(_) => format!("changed the notes of cue {cue}"),
                Edit::PreWait(_) => format!("changed the pre-wait of cue {cue}"),
                Edit::Duration(_) => format!("changed the duration of cue {cue}"),
                Edit::PostWait(_) => format!("changed the post-wait of cue {cue}"),
                Edit::Advance(_) => format!("changed what follows cue {cue}"),
                Edit::Level(_) => format!("changed the level of cue {cue}"),
                Edit::StopTarget(_) => format!("changed whether cue {cue} stops its target"),
            }
        }
        Change::Standby(standby) => match standby.after.and_then(|id| workspace.cue(id)) {
            Some(cue) => format!("moved standby to cue {}", number(cue)),
            None => "moved standby past the end".to_owned(),
        },
    }
}

/// Names a cue by its number, or by its name in quotes when it has no
/// number
fn number(cue: &Cue) -> String {
    if cue.number.trim().is_empty() {
        format!("\"{}\"", cue.name)
    } else {
        cue.number.clone()
    }
}

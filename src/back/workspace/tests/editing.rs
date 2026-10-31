// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, deleting, editing, and loading cues

use super::*;

#[test]
fn show_mode_blocks_edits_but_not_playback() {
    let workspace = workspace(vec![audio(1)])
        .apply(Command::ShowMode(true))
        .apply(Command::Edit(CueId(1), Edit::Name("Renamed".to_owned())))
        .apply(GO);
    assert_eq!(workspace.cue(CueId(1)).unwrap().name, "Sound");
    assert_eq!(workspace.active().len(), 1);
}

#[test]
fn new_cue_before_another_targets_the_cue_it_follows() {
    let add = Command::Add {
        list: 0,
        kind: Kind::Fade,
        at: Place::Before(CueId(2)),
    };
    let workspace = workspace(vec![audio(1), audio(2)]).apply(add);
    let cues = &workspace.lists()[0].cues;
    let order: Vec<CueId> = cues.iter().map(|cue| cue.id).collect();
    assert_eq!(order, [CueId(1), CueId(3), CueId(2)]);
    assert_eq!(cues[1].target, Some(CueId(1)));
}

#[test]
fn loading_ignores_a_file_that_has_been_replaced() {
    let workspace = workspace(vec![audio(1)])
        .apply(Command::Edit(CueId(1), Edit::File("other.ogg".to_owned())))
        .apply(Command::Loaded {
            cue: CueId(1),
            file: "sound.ogg".to_owned(),
            media: Media::Loaded { duration: 10.0 },
        });
    let cue = workspace.cue(CueId(1)).unwrap();
    assert_eq!(workspace.problem(cue), Some(Problem::NotLoaded));
}

#[test]
fn new_cues_are_numbered_after_the_highest_number_of_any_list() {
    let lists = vec![
        CueList::new("Main", vec![audio(1), audio(7)]),
        CueList::new("Preshow", Vec::new()),
    ];
    let add = Command::Add {
        list: 1,
        kind: Kind::Memo,
        at: Place::End,
    };
    let workspace = Workspace::new(lists).apply(add);
    assert_eq!(workspace.lists()[1].cues[0].number, "8");
}

#[test]
fn numbers_too_large_to_count_up_from_do_not_overflow() {
    let endless = Cue {
        number: "inf".to_owned(),
        ..audio(1)
    };
    let add = Command::Add {
        list: 0,
        kind: Kind::Memo,
        at: Place::End,
    };
    let workspace = workspace(vec![endless]).apply(add);
    assert_eq!(workspace.lists()[0].cues[1].number, u64::MAX.to_string());
}

#[test]
fn level_edit_leaves_a_cue_that_a_panic_is_stopping() {
    let workspace = workspace(vec![audio(1)])
        .apply(GO)
        .apply(Command::Panic)
        .apply(Command::Edit(CueId(1), Edit::Level(-3.0)))
        .apply(Command::Tick(PANIC_FADE));
    assert!(workspace.active().is_empty());
}

#[test]
fn run_whose_sound_cannot_start_ends_and_its_chain_plays_on() {
    let failed = Command::Loaded {
        cue: CueId(1),
        file: "sound.ogg".to_owned(),
        media: Media::Failed("file not found".to_owned()),
    };
    let workspace = workspace(vec![continuing_after(1, 2.0), audio(2)])
        .apply(GO)
        .apply(failed);
    let left: Vec<CueId> = workspace.active().iter().map(|active| active.id).collect();
    assert_eq!(left, [CueId(2)]);
    let cue = workspace.cue(CueId(1)).unwrap();
    assert_eq!(
        workspace.problem(cue),
        Some(Problem::LoadFailed("file not found"))
    );
}

#[test]
fn failure_of_a_file_that_was_replaced_leaves_the_cue_playing() {
    let replaced = Command::Loaded {
        cue: CueId(1),
        file: "other.ogg".to_owned(),
        media: Media::Loaded { duration: 10.0 },
    };
    let stale = Command::Loaded {
        cue: CueId(1),
        file: "sound.ogg".to_owned(),
        media: Media::Failed("file not found".to_owned()),
    };
    let workspace = workspace(vec![audio(1)])
        .apply(Command::Edit(CueId(1), Edit::File("other.ogg".to_owned())))
        .apply(replaced)
        .apply(GO)
        .apply(stale);
    assert!(workspace.active_of(CueId(1)).is_some());
}

#[test]
fn file_paths_leave_out_surrounding_spaces() {
    let workspace = Workspace::new(Vec::new()).directory("/show");
    assert_eq!(
        workspace.path(" sounds/storm.ogg\n"),
        std::path::Path::new("/show/sounds/storm.ogg")
    );
}

#[test]
fn editing_a_running_cue_leaves_the_run_under_way() {
    let workspace = workspace(vec![audio(1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(Command::Edit(
            CueId(1),
            Edit::PreWait("10".to_owned().into()),
        ));
    let active = workspace.active_of(CueId(1)).unwrap();
    assert_eq!(active.phase(), Phase::Running { remaining: 7.0 });
}

#[test]
fn deleting_the_last_cue_on_standby_moves_standby_past_the_end() {
    let workspace = workspace(vec![audio(1), audio(2)])
        .apply(Command::Standby {
            list: 0,
            cue: Some(CueId(2)),
        })
        .apply(Command::Delete(vec![CueId(2)]));
    assert_eq!(workspace.lists()[0].standby, None);
}

#[test]
fn new_numbers_read_numbers_with_spaces_and_large_numbers_exactly() {
    let spaced = Cue {
        number: " 12 ".to_owned(),
        ..audio(1)
    };
    let large = Cue {
        number: "16777217".to_owned(),
        ..audio(2)
    };
    let add = Command::Add {
        list: 0,
        kind: Kind::Memo,
        at: Place::End,
    };
    let numbers = |workspace: Workspace| workspace.lists()[0].cues.last().unwrap().number.clone();
    assert_eq!(numbers(workspace(vec![spaced]).apply(add.clone())), "13");
    assert_eq!(numbers(workspace(vec![large]).apply(add)), "16777218");
}

#[test]
fn files_that_could_not_be_played_are_listed_for_another_try() {
    let failed = Command::Loaded {
        cue: CueId(1),
        file: "sound.ogg".to_owned(),
        media: Media::Failed("file not found".to_owned()),
    };
    let workspace = workspace(vec![audio(1), audio(2)]).apply(failed);
    let listed: Vec<CueId> = workspace.failed().map(|(cue, _)| cue).collect();
    assert_eq!(listed, [CueId(1)]);
}

#[test]
fn editing_a_fade_cue_in_its_pre_wait_leaves_the_fade_it_will_make() {
    let fade = Cue {
        pre_wait: 2.0.into(),
        ..fade_out(2)
    };
    let workspace = workspace(vec![continuing(1), fade])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(Command::Edit(CueId(2), Edit::Level(-10.0)))
        .apply(Command::Edit(CueId(2), Edit::StopTarget(false)))
        .apply(Command::Tick(1.0));
    let fade = workspace.active_of(CueId(1)).unwrap().fade.unwrap();
    assert_eq!((fade.to, fade.then_stop), (SILENCE, true));
}

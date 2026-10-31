// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of undo and redo, which take back and make again the changes to
//! the show and leave playback alone

use super::*;

/// Returns the identities of the cues of the first list, in order
fn ids(workspace: &Workspace) -> Vec<u32> {
    workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

fn standby(workspace: &Workspace) -> Option<u32> {
    workspace.lists()[0].standby.map(|id| id.0)
}

fn name(workspace: &Workspace, id: u32) -> Option<&str> {
    workspace.cue(CueId(id)).map(|cue| cue.name.as_str())
}

fn delete(ids: &[u32]) -> Command {
    Command::Delete(ids.iter().map(|&id| CueId(id)).collect())
}

fn rename(id: u32, name: &str) -> Command {
    Command::Edit(CueId(id), Edit::Name(name.to_owned()))
}

fn move_standby(id: u32) -> Command {
    Command::Standby {
        list: 0,
        cue: Some(CueId(id)),
    }
}

#[test]
fn deleted_cues_come_back_in_place_as_one_change() {
    let deleted = workspace(vec![audio(1), audio(2), audio(3), audio(4)]).apply(delete(&[2, 3]));
    assert_eq!(ids(&deleted), [1, 4]);
    let undone = deleted.apply(Command::Undo);
    assert_eq!(ids(&undone), [1, 2, 3, 4]);
    assert_eq!(ids(&undone.apply(Command::Redo)), [1, 4]);
}

#[test]
fn standby_goes_back_to_a_deleted_cue_unless_go_has_moved_it_since() {
    let deleted = workspace(vec![audio(1), audio(2), audio(3)]).apply(delete(&[1]));
    assert_eq!(standby(&deleted), Some(2));
    assert_eq!(standby(&deleted.clone().apply(Command::Undo)), Some(1));
    let played = deleted.apply(GO).apply(Command::Undo);
    assert_eq!(ids(&played), [1, 2, 3]);
    assert_eq!(standby(&played), Some(3));
}

#[test]
fn added_cue_goes_away_and_comes_back_with_its_identity() {
    let workspace = workspace(vec![audio(1)]);
    let id = workspace.next_cue_id().0;
    let added = workspace.apply(Command::Add {
        list: 0,
        kind: Kind::Audio,
        at: Place::End,
    });
    assert_eq!(ids(&added.clone().apply(Command::Undo)), [1]);
    assert_eq!(
        ids(&added.apply(Command::Undo).apply(Command::Redo)),
        [1, id]
    );
}

#[test]
fn keys_typed_into_one_setting_make_one_change() {
    let typed = workspace(vec![audio(1)])
        .apply(rename(1, "S"))
        .apply(rename(1, "St"))
        .apply(rename(1, "Storm"))
        .apply(Command::Edit(CueId(1), Edit::Level(-6.0)));
    let level = typed.clone().apply(Command::Undo);
    assert_eq!(level.cue(CueId(1)).map(|cue| cue.level), Some(0.0));
    assert_eq!(name(&level, 1), Some("Storm"));
    assert_eq!(name(&level.apply(Command::Undo), 1), Some("Sound"));
}

#[test]
fn edit_that_changes_nothing_leaves_nothing_to_undo() {
    let workspace = workspace(vec![audio(1)]).apply(rename(1, "Sound"));
    assert!(workspace.undoable().is_none());
}

#[test]
fn undo_takes_back_only_the_setting_it_changed() {
    let reloaded = workspace(vec![audio(1)])
        .apply(rename(1, "Storm"))
        .apply(Command::Loaded {
            cue: CueId(1),
            file: "sound.ogg".to_owned(),
            media: Media::Loaded { duration: 20.0 },
        })
        .apply(Command::Undo);
    let cue = reloaded.cue(CueId(1)).expect("the cue is still there");
    assert_eq!(cue.name, "Sound");
    assert_eq!(cue.media, Media::Loaded { duration: 20.0 });
}

#[test]
fn new_change_leaves_nothing_to_redo() {
    let workspace = workspace(vec![audio(1)])
        .apply(rename(1, "Storm"))
        .apply(Command::Undo)
        .apply(Command::Edit(CueId(1), Edit::Level(-6.0)));
    assert!(workspace.redoable().is_none());
}

#[test]
fn show_mode_leaves_only_moves_of_standby_to_undo() {
    let show = workspace(vec![audio(1), audio(2)])
        .apply(rename(1, "Storm"))
        .apply(move_standby(2))
        .apply(Command::ShowMode(true))
        .apply(Command::Undo);
    assert_eq!(standby(&show), Some(1));
    let locked = show.apply(Command::Undo);
    assert_eq!(name(&locked, 1), Some("Storm"));
    assert!(locked.undoable().is_none());
}

#[test]
fn go_forgets_the_moves_of_the_standby_it_moves_on() {
    let played = workspace(vec![audio(1), audio(2), audio(3)])
        .apply(move_standby(2))
        .apply(GO);
    assert!(played.undoable().is_none());
    assert_eq!(standby(&played.apply(Command::Undo)), Some(3));
}

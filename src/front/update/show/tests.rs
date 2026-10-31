// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening and saving shows, and of keeping their unsaved changes
//! as long as they last

use std::fs;

use tempfile::TempDir;

use super::*;
use crate::back::{CueId, Edit, Kind, Place};

/// Show file of one list of one memo cue
const SHOW: &str = "[[list]]\nname = \"Main\"\n\n[[list.cue]]\nid = 1\nnumber = \"1\"\nkind = \"memo\"\nname = \"Note\"\n";

/// File of the storm show, in `/shows`
const STORM: &str = "/shows/storm.cinq.toml";

/// Returns what reading the show in `file` found: `text` in its file, and
/// `kept` among the unsaved changes
fn found(file: Option<&str>, text: Option<&str>, kept: Option<&str>) -> Read {
    Read {
        file: file.map(PathBuf::from),
        directory: PathBuf::from("/shows"),
        text: Ok(text.map(str::to_owned)),
        kept: Ok(kept.map(str::to_owned)),
    }
}

/// Returns the state of the interface with the storm show open
fn storm() -> App {
    App::default().opened(found(Some(STORM), Some(SHOW), None))
}

/// Renames the memo cue of the storm show
fn renamed(app: App, name: &str) -> App {
    app.send(Command::Edit(CueId(1), Edit::Name(name.to_owned())))
}

/// Returns a keeper of unsaved changes in a new temporary directory, along
/// with the directory, which goes once the test drops it
fn keeper() -> (Keeper, TempDir) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    (Keeper::new(Some(directory.path().to_owned())), directory)
}

#[test]
fn show_changed_and_changed_back_has_nothing_to_save() {
    let app = storm();
    assert!(!app.show.unsaved());
    let changed = renamed(app, "Cue");
    assert!(changed.show.unsaved());
    assert!(!changed.send(Command::Undo).show.unsaved());
}

#[test]
fn opening_brings_back_kept_changes_as_one_change_that_undo_takes_back() {
    let kept = SHOW.replace("Note", "Changed");
    let app = App::default().opened(found(Some(STORM), Some(SHOW), Some(&kept)));
    assert!(app.show.unsaved());
    let name = app.workspace.cue(CueId(1)).map(|cue| cue.name.clone());
    assert_eq!(name.as_deref(), Some("Changed"));
    assert!(!app.send(Command::Undo).show.unsaved());
}

#[test]
fn opening_a_file_that_is_not_there_starts_a_new_show_that_saving_puts_there() {
    let app = App::default().opened(found(Some("/shows/new.cinq.toml"), None, None));
    assert_eq!(app.show.name(), "new.cinq.toml");
    assert_eq!(app.workspace.lists().len(), 1);
    assert!(!app.show.unsaved());
}

#[test]
fn file_that_is_not_a_show_leaves_the_show_on_screen() {
    let app = storm().opened(found(Some("/shows/broken.cinq.toml"), Some("[[list"), None));
    assert_eq!(app.show.name(), "storm.cinq.toml");
    assert!(app.file_problem.is_some());
}

#[test]
fn opening_another_show_tells_where_the_unsaved_changes_went() {
    let app =
        renamed(storm(), "Cue").opened(found(Some("/shows/calm.cinq.toml"), Some(SHOW), None));
    assert_eq!(
        app.told.as_deref(),
        Some("Kept the unsaved changes to storm.cinq.toml | Control+o: open")
    );
}

#[test]
fn saving_leaves_unsaved_only_the_changes_made_since() {
    let new = App::default()
        .opened(found(None, None, None))
        .send(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: Place::End,
        });
    let file = PathBuf::from("/shows/new.cinq.toml");
    let show = new.show.id;
    let saved = |text: String| Saved {
        show,
        file: file.clone(),
        text,
        result: Ok(()),
    };
    let text = new.workspace.show(file.parent());
    let later = renamed(new, "Later");
    let after = later.saved(saved(text));
    assert_eq!(after.show.name(), "new.cinq.toml");
    assert!(after.show.unsaved());
    let text = after.workspace.show(file.parent());
    assert!(!after.saved(saved(text)).show.unsaved());
}

#[test]
fn failed_saving_leaves_the_changes_unsaved() {
    let changed = renamed(storm(), "Cue");
    let show = changed.show.id;
    let app = changed.saved(Saved {
        show,
        file: PathBuf::from(STORM),
        text: String::new(),
        result: Err("disk full".to_owned()),
    });
    assert!(app.show.unsaved());
    assert_eq!(
        app.file_problem.as_deref(),
        Some("Cannot save /shows/storm.cinq.toml: disk full")
    );
}

#[test]
fn keeper_keeps_unsaved_changes_until_they_are_gone() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let app = storm();
    assert_eq!(keeper.keep(&app.show), None);
    assert!(!kept.exists());
    let changed = renamed(app, "Cue");
    assert_eq!(keeper.keep(&changed.show), None);
    assert_eq!(
        fs::read_to_string(&kept).ok(),
        Some(changed.show.text.clone())
    );
    assert_eq!(keeper.keep(&changed.send(Command::Undo).show), None);
    assert!(!kept.exists());
}

#[test]
fn keeper_leaves_the_kept_changes_of_a_show_that_another_takes_the_place_of() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let changed = renamed(storm(), "Cue");
    keeper.keep(&changed.show);
    let other = changed.opened(found(Some("/shows/calm.cinq.toml"), Some(SHOW), None));
    assert_eq!(keeper.keep(&other.show), None);
    assert!(kept.exists());
}

#[test]
fn kept_changes_move_along_with_their_show() {
    let (mut keeper, directory) = keeper();
    let kept = kept_file(directory.path(), Some(Path::new(STORM)));
    let moved = PathBuf::from("/elsewhere/storm.cinq.toml");
    let changed = renamed(storm(), "Cue");
    keeper.keep(&changed.show);
    let text = changed.workspace.show(moved.parent());
    let show = changed.show.id;
    let saved = renamed(changed, "Later").saved(Saved {
        show,
        file: moved.clone(),
        text,
        result: Ok(()),
    });
    assert_eq!(keeper.keep(&saved.show), None);
    assert!(!kept.exists());
    assert!(kept_file(directory.path(), Some(&moved)).exists());
}

#[test]
fn save_that_ends_after_another_show_opens_leaves_that_show_alone() {
    let changed = renamed(storm(), "Cue");
    let show = changed.show.id;
    let text = changed.workspace.show(Path::new(STORM).parent());
    let calm = changed.opened(found(Some("/shows/calm.cinq.toml"), Some(SHOW), None));
    let saved = calm.show.saved.clone();
    let after = calm.saved(Saved {
        show,
        file: PathBuf::from(STORM),
        text,
        result: Ok(()),
    });
    assert_eq!(after.show.name(), "calm.cinq.toml");
    assert_eq!(after.show.saved, saved);
    assert!(!after.show.unsaved());
}

#[test]
fn show_read_back_holds_what_was_written_along_with_its_kept_changes() {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let file = directory.path().join("storm.cinq.toml");
    let unsaved = directory.path().join("unsaved");
    write(&file, SHOW, true).expect("the show can be written");
    write(&kept_file(&unsaved, Some(&file)), "kept", false).expect("changes can be kept");
    let read = Read::new(Some(file), Some(&unsaved));
    assert_eq!(read.text, Ok(Some(SHOW.to_owned())));
    assert_eq!(read.kept, Ok(Some("kept".to_owned())));
}

#[test]
fn chosen_file_gets_the_ending_of_a_show_unless_it_is_toml() {
    assert_eq!(with_extension(Path::new("/shows/storm")), Path::new(STORM));
    assert_eq!(
        with_extension(Path::new("/shows/storm.toml")),
        Path::new("/shows/storm.toml")
    );
    assert_eq!(
        with_extension(Path::new("/shows/storm.v2")),
        Path::new("/shows/storm.v2.cinq.toml")
    );
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the update cycle: the keys that act while the help page is
//! open, the selection after undo takes the selected cue away, and a link
//! of the help page that fails to open

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::back::{CueId, Place, sample};
use crate::front::keymap::Press;
use crate::front::{App, Message};

/// Presses a key with the default bindings while the help page is open,
/// and returns whether the page stays open
fn stays_open(key: Key) -> bool {
    let press = Press::from_key(&key, Modifiers::empty()).expect("keys in the tests are named");
    let open = App {
        help: true,
        ..App::default()
    };
    open.typed(press, false).0.help
}

#[test]
fn help_page_closes_on_its_keys_and_on_go_and_panic() {
    assert!(!stays_open(Key::Character("q".into())));
    assert!(!stays_open(Key::Character("?".into())));
    assert!(!stays_open(Key::Named(Named::Space)));
    assert!(!stays_open(Key::Named(Named::Escape)));
    assert!(stays_open(Key::Character("j".into())));
}

#[test]
fn undoing_away_the_selected_cue_selects_the_cue_in_its_place() {
    let app = App {
        workspace: sample::workspace(),
        selected: Some(CueId(4)),
        ..App::default()
    };
    let (added, _) = app.handled(Message::Add(Place::After));
    let (undone, _) = added.handled(Message::Undo);
    assert_eq!(undone.selected, Some(CueId(5)));
}

#[test]
fn link_that_fails_to_open_shows_until_the_help_page_closes() {
    let failure = || Message::Opened(Err("Cannot open it".to_owned()));
    let open = App {
        help: true,
        ..App::default()
    };
    let (failed, _) = open.next(failure());
    assert_eq!(failed.unopened.as_deref(), Some("Cannot open it"));
    let (closed, _) = failed.next(Message::CloseHelp);
    assert_eq!(closed.unopened, None);
    let (unseen, _) = closed.next(failure());
    assert_eq!(unseen.unopened, None);
}

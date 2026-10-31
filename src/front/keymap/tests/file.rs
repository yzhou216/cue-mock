// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the configuration file, which chooses a keymap, replaces the
//! keys of the actions it names, and reports what it cannot use

use std::path::Path;

use super::*;

#[test]
fn file_chooses_a_keymap_and_changes_the_keys_it_names() {
    let (keymap, problems) = config::parse("keymap = \"cua\"\n[keys]\ngo = \"F5\"\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.name(), "cua");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Undo), [keys("Control+z")]);
}

#[test]
fn file_naming_no_keymap_keeps_vi() {
    let (keymap, problems) = config::parse("keymap = \"emacs\"\n");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(keymap.name(), "vi");
}

#[test]
fn file_replaces_the_keys_of_the_actions_it_names() {
    let (keymap, problems) =
        config::parse("[keys]\ngo = \"F5\"\nnext = [\"n\", \"ArrowDown\"]\ndelete = []\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Next), [keys("n"), keys("ArrowDown")]);
    assert!(keymap.keys(Action::Delete).is_empty());
    assert_eq!(
        keymap.keys(Action::Previous),
        Keymap::default().keys(Action::Previous)
    );
}

#[test]
fn file_reports_what_it_cannot_use_and_keeps_the_rest() {
    let (keymap, problems) =
        config::parse("colour = 1\n[keys]\nfly = \"f\"\nnext = [\"Foo\", \"n\"]\nlast = \"1 G\"\n");
    assert_eq!(problems.len(), 4, "{problems:?}");
    assert_eq!(keymap.keys(Action::Next), [keys("n")]);
    assert!(keymap.keys(Action::Last).is_empty());
}

#[test]
fn file_that_is_not_toml_leaves_the_defaults() {
    let (keymap, problems) = config::parse("[keys\n");
    assert_eq!(problems.len(), 1);
    assert_eq!(keymap.keys(Action::Go), Keymap::default().keys(Action::Go));
}

#[test]
fn file_reports_bindings_that_hide_others() {
    let (_, problems) = config::parse("[keys]\nhelp = \"g\"\n");
    assert_eq!(problems.len(), 3, "{problems:?}");
}

#[test]
fn directory_named_on_the_command_line_holds_the_only_file() {
    assert_eq!(
        config::candidates(Some(Path::new("/show"))),
        [Path::new("/show/config.toml")]
    );
    assert!(config::candidates(Some(Path::new("show")))[0].is_absolute());
}

#[test]
fn file_defines_keymaps_of_its_own_from_others_or_from_no_keys() {
    let (keymap, problems) = config::parse(
        "keymap = \"booth\"\n[keymaps.booth]\nfrom = \"cua\"\ngo = \"F5\"\n[keys]\nhelp = \"F12\"\n",
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.name(), "booth");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Undo), [keys("Control+z")]);
    assert_eq!(keymap.keys(Action::Help), [keys("F12")]);
    let (keymap, problems) =
        config::parse("keymap = \"mine\"\n[keymaps.mine]\ngo = \"Space\"\npanic = \"Escape\"\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.keys(Action::Go), [keys("Space")]);
    assert!(keymap.keys(Action::Next).is_empty());
}

#[test]
fn file_reports_keymaps_that_cannot_be_built() {
    let (keymap, problems) =
        config::parse("keymap = \"a\"\n[keymaps.a]\nfrom = \"b\"\n[keymaps.b]\nfrom = \"a\"\n");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(keymap.name(), "a");
    assert!(keymap.keys(Action::Go).is_empty());
    let (_, problems) = config::parse("[keymaps.vi]\ngo = \"F5\"\n");
    assert_eq!(problems.len(), 1, "{problems:?}");
    let (_, problems) =
        config::parse("keymap = \"booth\"\n[keymaps.booth]\nfrom = \"emacs\"\nfly = \"f\"\n");
    assert_eq!(problems.len(), 2, "{problems:?}");
}

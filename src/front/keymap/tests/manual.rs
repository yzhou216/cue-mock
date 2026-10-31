// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the manual page of the configuration file, whose table lists
//! the keys that each keymap binds to every action in the order of the help
//! page, since a key bound to two actions goes to the one listed first

use std::iter;

use itertools::Itertools;

use super::*;

/// Manual page of the configuration file, in roff
const MANUAL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/man/cinq.5"));

/// Writes a row of the table of keymaps, whose cells tbl separates by tabs
fn row(name: &str, cells: impl IntoIterator<Item = String>) -> String {
    iter::chain([name.to_owned()], cells).join("\t")
}

#[test]
fn manual_lists_the_keys_of_every_keymap_in_order() {
    let keymaps: Vec<Keymap> = Preset::VARIANTS.iter().copied().map(Keymap::from).collect();
    let header = row(
        "Action",
        keymaps.iter().map(|keymap| keymap.name().to_owned()),
    );
    let actions = Action::VARIANTS.iter().map(|&action| {
        row(
            action.name(),
            keymaps.iter().map(|keymap| keymap.listed(action)),
        )
    });
    let rows: Vec<String> = iter::chain([header], actions).collect();
    let lines: Vec<&str> = MANUAL.lines().collect();
    let places: Vec<Option<usize>> = rows
        .iter()
        .map(|row| lines.iter().position(|line| line == row))
        .collect();
    let missing: Vec<&String> = rows
        .iter()
        .zip(&places)
        .filter_map(|(row, place)| place.is_none().then_some(row))
        .collect();
    assert!(
        missing.is_empty(),
        "man/cinq.5 lacks these rows of its table of keymaps: {missing:#?}"
    );
    assert!(
        places.is_sorted(),
        "man/cinq.5 lists the actions out of the order of the help page"
    );
}

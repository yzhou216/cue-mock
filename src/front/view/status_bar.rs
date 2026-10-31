// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Status of the show and the keys of GO and Panic, then at the right the
//! keys of a command typed so far, the latest change to the show with the
//! key that takes it back, and the key of the help page

use iced::widget::text::Wrapping;
use iced::widget::{container, rich_text, row, span, text};
use iced::{Center, Element, Fill, never};
use itertools::Itertools;

use super::{SPACING, format, theme, warnings};
use crate::back::Turn;
use crate::front::keymap::{Action, written};
use crate::front::{App, Message};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let list = app.current();
    let keymap = &app.config.keymap;
    let transport = [(Action::Go, "GO"), (Action::Panic, "Panic")]
        .into_iter()
        .filter_map(|(action, label)| keymap.key(action).map(|keys| format!(" | {keys}: {label}")))
        .collect::<String>();
    let count = warnings(app);
    // Show mode, which locks editing, and any warnings stand out
    let status = rich_text([
        span(if app.editable() {
            "Edit mode"
        } else {
            "Show mode"
        })
        .color_maybe((!app.editable()).then_some(theme::PALETTE.primary)),
        span(format!(
            " | {} list: {} | {} active | ",
            list.name,
            format::count(list.cues.len(), "cue"),
            app.workspace.active().len(),
        )),
        span(format::count(count, "warning"))
            .color_maybe((count > 0).then_some(theme::PALETTE.warning)),
        span(transport),
    ])
    .on_link_click(never)
    .wrapping(Wrapping::None);
    let help = keymap
        .key(Action::Help)
        .map(|keys| format!("{keys}: {}", Action::Help.name()));
    let typed = (!app.keys.is_empty()).then(|| written(&app.keys));
    // Hints grow to the left, so that the keys typed so far come and go
    // without moving the rest
    let hints = typed.into_iter().chain(notice(app)).chain(help).join(" | ");
    // Status stays on one line, cut off at the hints rather than wrapping,
    // so that the bar keeps its height
    row![container(status).width(Fill).clip(true), text(hints),]
        .spacing(SPACING)
        .padding(SPACING)
        .align_y(Center)
        .into()
}

/// Tells the latest change to the show, with the key that takes it back,
/// or that makes it again after an undo, while that key can act
fn notice(app: &App) -> Option<String> {
    let workspace = &app.workspace;
    let (turn, change) = workspace.latest_change()?;
    let told = format::change(workspace, change);
    let (told, action, next) = match turn {
        Turn::Done => (capitalized(&told), Action::Undo, workspace.undoable()),
        Turn::Undone => (format!("Undid: {told}"), Action::Redo, workspace.redoable()),
        Turn::Redone => (format!("Redid: {told}"), Action::Undo, workspace.undoable()),
    };
    next.map(|_| match app.config.keymap.key(action) {
        Some(keys) => format!("{told} | {keys}: {}", action.name()),
        None => told,
    })
}

/// Returns the text with its first letter in upper case
fn capitalized(text: &str) -> String {
    text.chars()
        .take(1)
        .flat_map(char::to_uppercase)
        .chain(text.chars().skip(1))
        .collect()
}

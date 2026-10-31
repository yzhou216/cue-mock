// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Help page, which lists every key binding over the rest of the window
//!
//! The page links to the list of key values and to the configuration file
//! that the keys came from, each of which opens in the program that the
//! desktop opens it with.

use std::ffi::OsString;

use iced::widget::text::{IntoFragment, Rich, Span};
use iced::widget::{
    center, column, container, mouse_area, opaque, rich_text, scrollable, span, table, text,
};
use iced::{Element, Fill};
use strum::VariantArray;

use super::{SPACING, button, theme};
use crate::front::keymap::Action;
use crate::front::keymap::config::Source;
use crate::front::{App, Message};

/// Widest that the page grows, so that its lines stay easy to read
const WIDTH: f32 = 960.0;

/// Space above and below the text of each row of the table
const ROW_GAP: f32 = 2.0;

/// List of the key values of the W3C UI Events specification
const KEY_VALUES: &str = "https://www.w3.org/TR/uievents-key/";

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let keymap = &app.config.keymap;
    let bindings = table::table(
        [
            table::column(text("Keys"), |action: Action| text(keymap.listed(action))),
            table::column(text("Action"), |action: Action| text(action.description())).width(Fill),
            table::column(text("Name in config.toml"), |action: Action| {
                text(action.name())
            }),
        ],
        Action::VARIANTS.iter().copied(),
    )
    // Rows closer than those of the cue list, so that the page fits the
    // window without scrolling
    .padding_y(ROW_GAP);
    let key_values = paragraph([
        span(
            "The manual page cinq(5) describes config.toml, where keys go by their W3C UI Events key values, listed at ",
        ),
        link(KEY_VALUES, KEY_VALUES),
        span(", apart from the space bar, which is Space."),
    ]);
    // File that is missing has nothing to open
    let source = match &app.config.source {
        Source::File(file) => paragraph([
            span("These keys come from "),
            link(file.display().to_string(), file.clone()),
            span(", and change as it does."),
        ]),
        Source::Missing(file) => paragraph([span(format!(
            "These are the default keys, which a file at {} can change.",
            file.display()
        ))]),
        Source::NoDirectory => paragraph([span("These are the default keys.")]),
    };
    let example = keymap
        .key(Action::Next)
        .map_or_else(String::new, |keys| format!(", as in 3 {keys}"));
    let page = column![
        text(format!("Keys of the {} keymap", keymap.name())).size(24),
        bindings,
        text(format!("A count typed before a key repeats it{example}, and picks the row for the first and last cue.")),
        text("The keys of GO and Panic act at once, even halfway through another command."),
        source,
        key_values,
    ]
    .extend(app.unopened.as_deref().map(|why| text(why).into()))
    .push(button("Close").on_press(Message::CloseHelp))
    .spacing(SPACING);
    let page = container(scrollable(page).spacing(0))
        .style(container::bordered_box)
        .padding(SPACING)
        .max_width(WIDTH);
    // Clicks beside the page close it, and the page keeps its own clicks
    opaque(mouse_area(center(opaque(page))).on_press(Message::CloseHelp))
}

/// Returns a paragraph of `spans`, whose links open what they name
fn paragraph<'a>(spans: impl AsRef<[Span<'a, OsString>]> + 'a) -> Rich<'a, OsString, Message> {
    rich_text(spans).on_link_click(Message::OpenLink)
}

/// Returns `label` as a link to `target`, in the color of links and
/// underlined, as links usually are
fn link<'a>(label: impl IntoFragment<'a>, target: impl Into<OsString>) -> Span<'a, OsString> {
    span(label)
        .link(target.into())
        .color(theme::LINK)
        .underline(true)
}

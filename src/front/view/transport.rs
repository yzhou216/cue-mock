// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! GO button, standby display, and show-wide playback controls

use iced::widget::{checkbox, column, row, text};
use iced::{Element, Fill};

use super::{SPACING, button, format, theme};
use crate::front::{App, Message};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let list = app.current();
    let standby = list.standby.and_then(|id| app.workspace.cue(id));
    let active = app.workspace.active();
    let summary = standby.map_or_else(
        || column![text("Standby: end of list").size(24)],
        |cue| {
            let notes = if cue.notes.is_empty() {
                "none"
            } else {
                cue.notes.as_str()
            };
            column![
                text(format!("Standby: {cue}")).size(24),
                text(format!("Notes: {notes}")),
                text(format::chain(&app.workspace, list, cue)),
            ]
        },
    );
    let playing = active.iter().any(|cue| !cue.paused);
    let paused = active.iter().any(|cue| cue.paused);
    row![
        button(text("GO").size(40))
            .style(theme::go)
            .on_press_maybe(standby.map(|_| Message::Go)),
        summary.width(Fill),
        column![
            row![
                button("Pause all").on_press_maybe(playing.then_some(Message::PauseAll)),
                button("Resume all").on_press_maybe(paused.then_some(Message::ResumeAll)),
                button("Panic").style(theme::panic).on_press(Message::Panic),
            ]
            .spacing(SPACING),
            checkbox(app.workspace.show_mode())
                .label("Show mode")
                .on_toggle(Message::ShowMode),
        ]
        .spacing(SPACING),
    ]
    .spacing(SPACING)
    .padding(SPACING)
    .into()
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue list, one row per cue
//!
//! The radio button marks the selection, which only drives the inspector,
//! while NEXT marks the standby cue that GO fires.

use iced::widget::{container, mouse_area, radio, scrollable, table, text};
use iced::{Element, Fill, Length, Theme};

use super::{CUES, ROW_PADDING, SELECTED, format};
use crate::back::{Cue, Workspace};
use crate::front::{App, Message};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let list = app.current();
    let text_column = move |title, content: fn(&App, &Cue) -> String| {
        table::column(text(title), move |cue: &Cue| {
            cell(app, cue, text(content(app, cue)))
        })
    };
    let columns = [
        table::column(text("#"), |cue: &Cue| {
            let number = radio(cue.number.clone(), cue.id, app.selected, Message::Select);
            marked(app, cue, number)
        }),
        table::column(text("Next"), |cue: &Cue| {
            cell(app, cue, text(format::next(list, cue)).style(text::primary))
        }),
        text_column("Type", |_, cue| cue.kind.to_string()),
        text_column("Name", |_, cue| cue.name.clone()).width(Fill),
        text_column("Target", |app, cue| format::target(&app.workspace, cue)),
        text_column("Pre", |_, cue| format::wait(cue.pre_wait.value().into())),
        text_column("Action", |_, cue| format::action(cue)),
        text_column("Post", |_, cue| format::post_wait(cue)),
        text_column("Then", |_, cue| format::advance(cue.advance).to_owned()),
        table::column(text("State"), |cue: &Cue| {
            let (state, style) = state(&app.workspace, cue);
            cell(app, cue, text(state).style(style))
        })
        .width(Length::Fixed(170.0)),
    ];
    // Embedded scroll bars take their own space instead of covering text
    scrollable(table::table(columns, &list.cues).padding_y(ROW_PADDING))
        .id(CUES)
        .spacing(0)
        .height(Fill)
        .into()
}

/// Makes a table cell select its cue when clicked
fn cell<'a>(
    app: &App,
    cue: &Cue,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    marked(
        app,
        cue,
        mouse_area(content).on_press(Message::Select(cue.id)),
    )
}

/// Marks the cells of the selected cue, so that the cue list can find the
/// whole row to scroll it into view
fn marked<'a>(
    app: &App,
    cue: &Cue,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let cell = container(content);
    if app.selected == Some(cue.id) {
        cell.id(SELECTED).into()
    } else {
        cell.into()
    }
}

/// Describes a cue in the State column, in the color of what it needs: red
/// while it cannot play, yellow while it is paused, and green while it runs
fn state(workspace: &Workspace, cue: &Cue) -> (String, fn(&Theme) -> text::Style) {
    match (workspace.problem(cue), workspace.active_of(cue.id)) {
        (Some(_), _) => ("BROKEN".to_owned(), text::danger),
        (None, Some(active)) if active.paused => (format::phase(active), text::warning),
        (None, Some(active)) => (format::phase(active), text::success),
        (None, None) => (String::new(), text::default),
    }
}

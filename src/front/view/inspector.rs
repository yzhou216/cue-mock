// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Inspector, which shows every setting of the selected cue on two tabs
//!
//! Its fields are read-only in show mode, and they keep the keys typed
//! into them.

mod field;

use iced::widget::{checkbox, column, progress_bar, radio, row, rule, slider, table, text};
use iced::{Element, Fill, Length};

use super::{SPACING, format, keep_keys};
use crate::back::{Cue, Edit, Kind, SILENCE};
use crate::front::{App, Message, Tab};
use field::Field;

/// Highest level in dB that a level slider offers
const LOUDEST: f32 = 12.0;

/// Fields of the Basics tab, one column after the other
const BASICS: [&[Field]; 2] = [
    &[Field::Number, Field::Name, Field::Target, Field::Notes],
    &[
        Field::PreWait,
        Field::Duration,
        Field::PostWait,
        Field::Then,
    ],
];

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let tabs = column![
        text("Inspector"),
        radio("Basics", Tab::Basics, Some(app.tab), Message::Tab),
        radio("Levels", Tab::Levels, Some(app.tab), Message::Tab),
    ]
    .spacing(SPACING);
    let body = match app.selected.and_then(|id| app.workspace.cue(id)) {
        Some(cue) => inspected(app, cue),
        None => text("Select a cue to inspect it").into(),
    };
    keep_keys(
        row![tabs, rule::vertical(1), body]
            .spacing(SPACING)
            .padding(SPACING)
            .height(Length::Fixed(240.0)),
        &app.config.keymap,
    )
}

/// Settings of a cue on the open tab, under the name of the cue
fn inspected<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let fields = match app.tab {
        Tab::Basics => row(BASICS.map(|fields| form(app, cue, fields)))
            .spacing(SPACING)
            .into(),
        Tab::Levels => levels(cue, app.editable()),
    };
    let lock = if app.editable() {
        ""
    } else {
        " (read-only in show mode)"
    };
    column![text(format!("{cue}, {} cue{lock}", cue.kind)), fields]
        .spacing(SPACING)
        .width(Fill)
        .into()
}

/// Table of labelled inspector fields
fn form<'a>(app: &'a App, cue: &'a Cue, fields: &[Field]) -> Element<'a, Message> {
    table::table(
        [
            table::column(text("Setting"), move |field: Field| text(field.label(cue))),
            table::column(text("Value"), move |field: Field| field.input(app, cue)).width(Fill),
        ],
        fields.iter().copied(),
    )
    .into()
}

/// Level of an audio cue, or the level a fade cue fades to, which become
/// read-only in show mode
fn levels(cue: &Cue, editable: bool) -> Element<'_, Message> {
    let label = match cue.kind {
        Kind::Audio => "Level",
        Kind::Fade => "Fade target to",
        Kind::Stop | Kind::Wait | Kind::Memo => {
            return text(format!("{} cues have no level", cue.kind)).into();
        }
    };
    let id = cue.id;
    let level: Element<'_, Message> = if editable {
        slider(SILENCE..=LOUDEST, cue.level, move |db| {
            Message::Edit(id, Edit::Level(db))
        })
        .step(0.5_f32)
        .into()
    } else {
        progress_bar(SILENCE..=LOUDEST, cue.level).into()
    };
    let stop = (cue.kind == Kind::Fade).then(|| {
        checkbox(cue.stop_target)
            .label("Stop target when done")
            .on_toggle_maybe(
                editable.then_some(move |stop| Message::Edit(id, Edit::StopTarget(stop))),
            )
    });
    column![
        // Padding keeps the value one width, so the slider holds its length
        row![
            text(label),
            level,
            text(format!("{:>8}", format::level(cue.level)))
        ]
        .spacing(SPACING),
        stop,
    ]
    .spacing(SPACING)
    .into()
}

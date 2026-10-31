// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editing actions above the cue list

use iced::Element;
use iced::widget::{row, space};
use strum::VariantArray;

use super::{SPACING, button, dropdown};
use crate::back::{Kind, Place};
use crate::front::{App, Message};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let selected_here = app.selected_here().is_some();
    row![
        button("Standby selected")
            .on_press_maybe(selected_here.then_some(Message::StandbySelected)),
        space::horizontal(),
        dropdown(
            Kind::VARIANTS,
            Some(app.new_kind),
            Message::NewKind,
            &app.config.keymap
        ),
        button("Add cue").on_press_maybe(app.editable().then_some(Message::Add(Place::After))),
        button("Delete cue")
            .on_press_maybe((app.editable() && selected_here).then_some(Message::Delete(1))),
    ]
    .spacing(SPACING)
    .padding(SPACING)
    .into()
}

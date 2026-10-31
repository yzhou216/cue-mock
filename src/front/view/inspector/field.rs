// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fields of the inspector, each a labelled input for one setting

use iced::Element;
use iced::widget::text_input;
use strum::VariantArray;

use crate::back::{Advance, Cue, Edit, Kind};
use crate::front::view::{NAME, dropdown, format};
use crate::front::{App, Message};

/// Inspector field, one per row of a form
#[derive(Debug, Clone, Copy)]
pub(super) enum Field {
    Number,
    Name,
    Target,
    Notes,
    PreWait,
    Duration,
    PostWait,
    Then,
}

impl Field {
    pub(super) fn label(self, cue: &Cue) -> &'static str {
        match self {
            Field::Number => "Number",
            Field::Name => "Name",
            Field::Target if cue.kind == Kind::Audio => "File",
            Field::Target => "Target",
            Field::Notes => "Notes",
            Field::PreWait => "Pre-wait",
            Field::Duration => "Duration",
            Field::PostWait => "Post-wait",
            Field::Then => "Then",
        }
    }

    /// Returns the input for this field.  It is disabled when editing is
    /// locked or when the value does not apply to the cue, such as the
    /// post-wait of an auto-follow cue.  Values that come from elsewhere,
    /// such as the length of an audio file, show in a disabled field too.
    pub(super) fn input<'a>(self, app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
        let id = cue.id;
        let editable = app.editable();
        let field =
            |value: &str, applies: bool, edit: fn(String) -> Edit| -> Element<'a, Message> {
                let input = text_input(self.placeholder(cue), value).on_input_maybe(
                    (editable && applies).then_some(move |text| Message::Edit(id, edit(text))),
                );
                match self {
                    Field::Name => input.id(NAME).into(),
                    _ => input.into(),
                }
            };
        match self {
            Field::Number => field(&cue.number, true, Edit::Number),
            Field::Name => field(&cue.name, true, Edit::Name),
            Field::Target => match cue.kind {
                Kind::Audio => field(&cue.file, true, Edit::File),
                Kind::Fade | Kind::Stop if editable => target_menu(app, cue),
                Kind::Fade | Kind::Stop => shown(
                    &cue.target
                        .and_then(|target| app.workspace.cue(target))
                        .map_or_else(String::new, Cue::to_string),
                ),
                Kind::Wait | Kind::Memo => shown(""),
            },
            Field::Notes => field(&cue.notes, true, Edit::Notes),
            Field::PreWait => field(cue.pre_wait.text(), true, |text| Edit::PreWait(text.into())),
            Field::Duration if cue.kind.has_duration() => {
                field(cue.duration.text(), true, |text| {
                    Edit::Duration(text.into())
                })
            }
            Field::Duration => shown(&format::action(cue)),
            Field::PostWait => field(
                cue.post_wait.text(),
                cue.advance != Advance::Follow,
                |text| Edit::PostWait(text.into()),
            ),
            Field::Then if editable => dropdown(
                Advance::VARIANTS,
                Some(cue.advance),
                move |advance| Message::Edit(id, Edit::Advance(advance)),
                &app.config.keymap,
            )
            .into(),
            Field::Then => shown(&cue.advance.to_string()),
        }
    }

    /// Returns the hint an empty field shows
    fn placeholder(self, cue: &Cue) -> &'static str {
        match self {
            Field::Target if cue.kind == Kind::Audio => "path of a sound file",
            Field::PreWait | Field::Duration | Field::PostWait => "0",
            _ => "",
        }
    }
}

/// Shows a value in a disabled field, so it lines up with editable ones
fn shown<'a>(value: &str) -> Element<'a, Message> {
    text_input("", value).into()
}

/// Menu of the cues that a fade or stop cue can act on, each shown by its
/// number and name
fn target_menu<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let id = cue.id;
    let targets: Vec<&Cue> = app
        .workspace
        .cues()
        .filter(|other| cue.can_target(other))
        .collect();
    let selected = targets
        .iter()
        .find(|target| Some(target.id) == cue.target)
        .copied();
    dropdown(
        targets,
        selected,
        move |target: &Cue| Message::Edit(id, Edit::Target(Some(target.id))),
        &app.config.keymap,
    )
    .placeholder("choose a cue")
    .into()
}

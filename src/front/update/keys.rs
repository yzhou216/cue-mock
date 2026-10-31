// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys and the keyboard focus
//!
//! Keys typed outside any text field go to the key bindings in
//! [`crate::front::keymap`], and the keys of a command typed halfway wait
//! in the interface state until the command is complete.  The inspector
//! keeps the keys typed into its fields, apart from the keys of GO and
//! Panic and the keys that move between fields.  iced moves the keyboard
//! focus only on clicks, so the keys that edit fields move it themselves.

use iced::Task;
use iced::advanced::widget::{operate, operation::focusable};
use iced::keyboard;
use iced::widget::operation;

use crate::front::keymap::Press;
use crate::front::{App, Focus, Message, Tab, view};

impl App {
    /// Reads a key together with the keys typed before it, and carries out
    /// the command they complete.  A key held down repeats, apart from the
    /// keys of GO and Panic, which act once a press, so that holding the
    /// key of Panic does not cut a panic short.  While the help page is
    /// open, only GO, Panic, and the keys that show or hide the page act.
    pub(super) fn typed(self, press: Press, repeat: bool) -> (Self, Task<Message>) {
        let outcome = self.config.keymap.parse(&self.keys, press);
        let app = Self {
            keys: outcome.pending,
            ..self
        };
        match outcome.action {
            Some((action, _)) if repeat && action.acts_at_once() => (app, Task::none()),
            Some((action, count)) if !app.help || action.acts_over_help() => {
                app.handled(action.message(count))
            }
            _ => (app, Task::none()),
        }
    }

    /// Moves the keyboard focus.  Editing the name shows the Basics tab,
    /// which holds the Name field, and does nothing in show mode or without
    /// a selection.
    pub(super) fn focused(self, focus: Focus) -> (Self, Task<Message>) {
        match focus {
            Focus::Name if self.editable() && self.selected.is_some() => (
                Self {
                    tab: Tab::Basics,
                    ..self
                },
                operation::focus(view::NAME),
            ),
            Focus::Name => (self, Task::none()),
            Focus::Next => (self, operation::focus_next()),
            Focus::Previous => (self, operation::focus_previous()),
            Focus::Cues => (self, operate(focusable::unfocus())),
        }
    }
}

/// Reads a key that no widget captured, with Shift applied, so that Shift
/// and `g` make `G`
pub(super) fn key_press(event: keyboard::Event) -> Option<Message> {
    let keyboard::Event::KeyPressed {
        modified_key,
        modifiers,
        repeat,
        ..
    } = event
    else {
        return None;
    };
    Press::from_key(&modified_key, modifiers).map(|press| Message::Typed { press, repeat })
}

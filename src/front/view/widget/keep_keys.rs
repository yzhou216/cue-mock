// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region of text fields that keeps the keys typed into them
//!
//! iced's text fields let some keys through, such as the arrows, and a
//! read-only field lets every key through.  Those keys would reach the key
//! bindings and act on the cue list while the operator types.  While one of
//! the fields has the keyboard focus, this region keeps the keys that the
//! field let through, apart from the keys that act in fields, which are
//! those of GO and Panic and those that move between fields.  Enter leaves
//! the field.  The region decides as each key arrives, so keys act in the
//! order the operator typed them.

use iced::advanced::Shell;
use iced::advanced::layout::Layout;
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Id, Operation, Tree};
use iced::keyboard::{self, Key, key::Named};
use iced::{Element, Event, Rectangle, Renderer};

use super::wrap::{Intercept, Wrap};
use crate::front::keymap::{Keymap, Press};
use crate::front::{Focus, Message};

/// Keeps the keys typed into the fields of `content`, apart from the keys
/// that `keymap` lets act in fields
pub fn keep_keys<'a>(
    content: impl Into<Element<'a, Message>>,
    keymap: &'a Keymap,
) -> Element<'a, Message> {
    Wrap {
        content: content.into(),
        intercept: Keep(keymap),
    }
    .into()
}

/// Keeps the keys that a focused field let through
struct Keep<'a>(&'a Keymap);

impl Intercept for Keep<'_> {
    fn after(
        &self,
        content: &mut Element<'_, Message>,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modified_key,
            modifiers,
            ..
        }) = event
        else {
            return;
        };
        if shell.is_event_captured() {
            return;
        }
        let mut focus = HasFocus(false);
        content
            .as_widget_mut()
            .operate(tree, layout, renderer, &mut focus);
        if !focus.0 {
            return;
        }
        let acts = Press::from_key(modified_key, *modifiers)
            .is_some_and(|press| self.0.acts_in_fields(press));
        match key {
            _ if acts => {}
            Key::Named(Named::Enter) => {
                shell.publish(Message::Focus(Focus::Cues));
                shell.capture_event();
            }
            _ => shell.capture_event(),
        }
    }
}

/// Operation that finds out whether a text field has the keyboard focus,
/// read-only fields included
struct HasFocus(bool);

impl Operation for HasFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0 |= state.is_focused();
    }
}

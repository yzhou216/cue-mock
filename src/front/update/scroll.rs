// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the cue list so that the selected cue stays in view
//!
//! The list measures the selected row as it is laid out, so a row that a
//! long name wraps onto two lines scrolls into view as surely as any other.
//! Measuring and scrolling happen in one operation, which iced carries out
//! before it draws the next frame.

use iced::advanced::widget::operation::{Operation, Outcome, Scrollable, scrollable};
use iced::advanced::widget::{Id, operate};
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};
use iced::{Rectangle, Task, Vector};

use crate::front::{App, Message, view};

impl App {
    /// Returns the state along with the operation that scrolls the selected
    /// cue into view
    pub(super) fn revealed(self) -> (Self, Task<Message>) {
        let task = self.reveal();
        (self, task)
    }

    /// Scrolls the cue list as little as it takes to show the selected cue.
    /// The first cue scrolls the list to the top, so that the header shows
    /// along with it.
    fn reveal(&self) -> Task<Message> {
        match self.selected.and_then(|id| self.current().position(id)) {
            Some(0) => operation::snap_to(view::CUES, RelativeOffset::START),
            Some(_) => operate(Measure::default()).discard(),
            None => Task::none(),
        }
    }
}

/// Operation that finds the selected cue in the cue list, then scrolls the
/// list to show it, if it is out of view
#[derive(Default)]
struct Measure {
    /// Top of the list content, how far the list has scrolled, and the
    /// height of the part in view
    list: Option<(f32, f32, f32)>,
    /// Area that the cells of the selected cue cover, which leaves out the
    /// padding of the row
    row: Option<Rectangle>,
}

impl Operation for Measure {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id == Some(&Id::new(view::CUES)) {
            self.list = Some((content_bounds.y, translation.y, bounds.height));
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&Id::new(view::SELECTED)) {
            self.row = Some(self.row.map_or(bounds, |row| row.union(&bounds)));
        }
    }

    fn finish(&self) -> Outcome<()> {
        let (Some((origin, scrolled, height)), Some(row)) = (self.list, self.row) else {
            return Outcome::None;
        };
        let top = row.y - view::ROW_PADDING - origin;
        let bottom = row.y + row.height + view::ROW_PADDING - origin;
        let offset = if top < scrolled {
            top
        } else if bottom > scrolled + height {
            bottom - height
        } else {
            return Outcome::None;
        };
        let offset = AbsoluteOffset {
            x: None,
            y: Some(offset),
        };
        Outcome::Chain(Box::new(scrollable::scroll_to(Id::new(view::CUES), offset)))
    }
}

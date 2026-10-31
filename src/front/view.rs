// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! View functions, which keep iced's default look for every widget and only
//! space the controls apart
//!
//! Each part of the window has a module of its own, with a `view` function
//! that draws it.  The text of back-end values comes from
//! [`format`](mod@format), the colors from [`theme`], and the widgets that
//! iced lacks from [`widget`].

mod cue_table;
mod format;
mod help;
mod inspector;
mod next_keys;
mod sidebar;
mod status_bar;
pub(super) mod theme;
mod toolbar;
mod transport;
mod widget;

use iced::widget::{column, row, rule, stack};
use iced::{Element, Fill};

use super::{App, Message};
use widget::{button, dropdown, keep_keys};

/// Identity of the cue list, through which the selection scrolls into view
pub const CUES: &str = "cues";

/// Identity of the cells of the selected cue, through which the cue list
/// finds where the selected cue is
pub const SELECTED: &str = "selected";

/// Space above and below the cells of each row of the cue list, which is
/// iced's own padding for tables
pub const ROW_PADDING: f32 = 5.0;

/// Identity of the Name field, through which `i` gives it the keyboard focus
pub const NAME: &str = "name";

/// Gap between neighbouring controls and around each part of the window,
/// so that every control stands apart from the next
const SPACING: f32 = 10.0;

/// Draws the window, its parts stacked with rules between them, the keys
/// that can come next over them just above the status bar, and the help
/// page over everything while it is open.  Each layer keeps its place
/// whether or not it shows, since iced matches the state of each widget to
/// the widget in the same place, so that the cue list stays scrolled where
/// it was, and the fields keep what they hold, as the layers come and go.
pub fn view(app: &App) -> Element<'_, Message> {
    let parts = column![
        transport::view(app),
        rule::horizontal(1),
        toolbar::view(app),
        rule::horizontal(1),
        row![cue_table::view(app), rule::vertical(1), sidebar::view(app)].height(Fill),
        rule::horizontal(1),
        inspector::view(app),
    ];
    let window = column![
        stack![parts, next_keys::view(app)],
        rule::horizontal(1),
        status_bar::view(app),
    ];
    stack![window, app.help.then(|| help::view(app))].into()
}

/// Counts the cues that cannot play, a missing audio device, and the
/// problems with the configuration file
fn warnings(app: &App) -> usize {
    app.workspace.problems().count()
        + usize::from(app.audio.error().is_some())
        + app.config.problems.len()
}

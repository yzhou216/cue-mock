// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! iced interface for a workspace
//!
//! The interface keeps only its own state, such as the selection and the
//! open inspector tab.  It reads the show from the back end and changes it
//! only by sending back-end commands.  Operator input arrives as a
//! [`Message`], [`update`] applies it, and [`view`] draws the result.  Keys
//! become messages through [`keymap`], whose bindings a configuration file
//! can change.

mod keymap;
mod message;
mod update;
mod view;

use std::path::PathBuf;
use std::time::Instant;

use iced::{Font, Size, window};

use crate::back::{Audio, CueId, CueList, Kind, Workspace};
use keymap::Press;
use keymap::config::Config;
use message::{Focus, Message, Tab};

/// Opens the window and runs the interface until the window closes.  The
/// key bindings come from `config.toml` in `config_dir`, when it is given,
/// and otherwise from the configuration directories.
pub fn run(config_dir: Option<PathBuf>) -> iced::Result {
    let files = keymap::config::candidates(config_dir.as_deref());
    iced::application(move || App::new(&files), App::update, view::view)
        .title("cinq")
        .theme(view::theme::modus_vivendi())
        .default_font(Font::MONOSPACE)
        .subscription(App::subscription)
        .window(window::Settings {
            size: Size::new(1440.0, 900.0),
            // Application ID for window manager rules, since some window
            // managers float a window whose ID iced has left empty
            #[cfg(target_os = "linux")]
            platform_specific: window::settings::PlatformSpecific {
                application_id: "cinq".to_owned(),
                ..window::settings::PlatformSpecific::default()
            },
            ..window::Settings::default()
        })
        .run()
}

/// State of the interface, around the workspace that it shows
#[derive(Default)]
struct App {
    workspace: Workspace,
    /// Sound output, which plays what the workspace describes
    audio: Audio,
    /// Index of the cue list shown in the main table
    list: usize,
    /// Cue shown in the inspector
    selected: Option<CueId>,
    tab: Tab,
    /// Kind of cue that Add cue creates
    new_kind: Kind,
    /// Time of the last clock tick, or `None` while the clock is stopped
    last_tick: Option<Instant>,
    /// Key bindings, and where they came from
    config: Config,
    /// Keys of a command typed so far, such as a count or the `g` of `g g`
    keys: Vec<Press>,
    /// Whether the help page with every key binding is open
    help: bool,
    /// Why the link that the help page opened last failed to open, which
    /// the page shows while it stays open
    unopened: Option<String>,
    /// Whether the files that could not be played are being read again
    rechecking: bool,
}

impl App {
    fn current(&self) -> &CueList {
        &self.workspace.lists()[self.list]
    }

    /// Returns the selection if it belongs to the list on screen
    fn selected_here(&self) -> Option<CueId> {
        self.selected
            .filter(|&id| self.current().position(id).is_some())
    }

    fn editable(&self) -> bool {
        !self.workspace.show_mode()
    }
}

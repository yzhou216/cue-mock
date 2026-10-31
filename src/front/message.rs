// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Operator intent, which the view and the keys send and the update applies

use std::ffi::OsString;
use std::time::Instant;

use strum::VariantArray;

use super::keymap::Press;
use super::keymap::config::Config;
use crate::back::{Command, CueId, Edit, Kind, Place};

/// Page of the inspector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, VariantArray)]
pub enum Tab {
    #[default]
    Basics,
    Levels,
}

impl Tab {
    /// Returns the tab `offset` places to the right, wrapping around at
    /// either end
    pub fn stepped(self, offset: isize) -> Self {
        let index = Self::VARIANTS
            .iter()
            .position(|&tab| tab == self)
            .unwrap_or(0);
        let count = Self::VARIANTS.len() as isize;
        Self::VARIANTS[(index as isize + offset).rem_euclid(count) as usize]
    }
}

/// Text field that keys move the keyboard focus to, since iced itself
/// moves it only on clicks
#[derive(Debug, Clone, Copy)]
pub enum Focus {
    /// Name field of the inspector, for editing the selected cue
    Name,
    /// Next field, or the first one when no field has focus
    Next,
    Previous,
    /// No field, which hands the keys back to the cue list
    Cues,
}

/// Operator intent, which becomes a back-end command or an interface change
#[derive(Debug, Clone)]
pub enum Message {
    Go,
    Panic,
    PauseAll,
    ResumeAll,
    Stop(CueId),
    Tick(Instant),
    Select(CueId),
    /// Moves the selection by a number of cues, downwards when positive
    Step(isize),
    /// Selects the cue on a row counted from one, or the last cue for a row
    /// past the end
    Row(usize),
    StandbySelected,
    SwitchList(usize),
    /// Shows the cue list a number of lists further down the sidebar
    StepList(isize),
    ShowMode(bool),
    Tab(Tab),
    /// Shows the inspector tab a number of tabs to the right
    StepTab(isize),
    NewKind(Kind),
    /// Adds a cue next to the selection, on the side that the function
    /// picks
    Add(fn(CueId) -> Place),
    /// Deletes a number of cues from the selection down
    Delete(usize),
    Edit(CueId, Edit),
    /// Takes back the latest change to the cues
    Undo,
    /// Makes again the change that undo took back last
    Redo,
    Focus(Focus),
    /// Shows the help page with every key binding, or hides it
    ToggleHelp,
    CloseHelp,
    /// Key typed outside any text field, and whether it is a repeat of a
    /// key held down
    Typed {
        press: Press,
        repeat: bool,
    },
    /// Time to load again the files that could not be played
    Recheck,
    /// What loading again the files that could not be played found
    Rechecked(Vec<Command>),
    /// Configuration read again after one of its files changed
    Reconfigured(Config),
    /// Opens a web page or a file in the program that the desktop opens it
    /// with
    Open(OsString),
    /// Why opening a link failed, if it did
    Opened(Result<(), String>),
    /// Mouse button pressed anywhere in the window
    Clicked,
    /// Window changed size
    Resized,
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Trying again the files that could not be played
//!
//! A file can fail because its drive has gone, and reading such a file can
//! block for a long time, so the files are read again off the interface
//! thread, where GO and Panic must never wait.  One recheck runs at a time,
//! so a read that hangs never piles up more reads behind it.

use iced::Task;

use crate::back::{Command, load_files};
use crate::front::{App, Message};

impl App {
    /// Starts reading again the files that could not be played, unless a
    /// recheck is still under way
    pub(super) fn recheck(self) -> (Self, Task<Message>) {
        if self.rechecking {
            return (self, Task::none());
        }
        let files: Vec<_> = self
            .workspace
            .failed()
            .map(|(cue, file)| (cue, file.to_owned(), self.workspace.path(file)))
            .collect();
        if files.is_empty() {
            return (self, Task::none());
        }
        // Executor runs the blocking read on one of its worker threads
        let read = Task::perform(async move { load_files(files) }, Message::Rechecked);
        (
            Self {
                rechecking: true,
                ..self
            },
            read,
        )
    }

    /// Records what reading the files again found
    pub(super) fn rechecked(self, loads: Vec<Command>) -> Self {
        let app = loads.into_iter().fold(self, Self::send);
        Self {
            rechecking: false,
            ..app
        }
    }
}

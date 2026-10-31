// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Update cycle, which applies each message to the interface state
//!
//! Each concern of the update has a module of its own: [`navigation`]
//! moves the selection, [`editing`] adds and deletes cues, [`keys`] reads
//! keys and moves the keyboard focus, [`scroll`] keeps the selected cue in
//! view whenever the selection moves, and [`recheck`] tries again the files
//! that could not be played.  The key bindings follow the configuration
//! file as it changes, through [`keymap::config::watch`].

mod editing;
mod keys;
mod navigation;
mod recheck;
mod scroll;
#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;

use iced::{Event, Subscription, Task, event, keyboard, mouse, time, window};

use super::{App, Message, keymap};
use crate::back::{Audio, Command, sample};

/// Interval of the playback clock while any cue is running
const TICK: Duration = Duration::from_millis(25);

/// Interval at which the files that could not be played are tried again
const RECHECK: Duration = Duration::from_secs(2);

impl App {
    /// Opens the audio device and the sample show, and reads the key
    /// bindings from the first of `files` that exists
    pub(super) fn new(files: &[PathBuf]) -> Self {
        let mut audio = Audio::open();
        let workspace = audio.sync(sample::workspace());
        Self {
            workspace,
            audio,
            selected: Some(sample::FIRST_SELECTED),
            config: keymap::config::load(files),
            ..Self::default()
        }
    }

    /// Applies one message, then lets the audio output catch up with the
    /// result.  This is the only place where state is replaced, and the
    /// audio output is the only part it calls that has side effects;
    /// everything else builds new values.  Taking the state out of `self`
    /// instead of cloning it keeps updates free of copies.
    pub(super) fn update(&mut self, message: Message) -> Task<Message> {
        let (app, task) = std::mem::take(self).next(message);
        *self = app;
        self.workspace = self.audio.sync(std::mem::take(&mut self.workspace));
        task
    }

    /// Listens for keys, clicks, changes of the window size, and changes of
    /// the configuration, for clock ticks while any cue is running, and for
    /// the time to try again the files that could not be played
    pub(super) fn subscription(&self) -> Subscription<Message> {
        let ticks = self.running().then(|| time::every(TICK).map(Message::Tick));
        let rechecks = (!self.rechecking && self.workspace.failed().next().is_some())
            .then(|| time::every(RECHECK).map(|_| Message::Recheck));
        let always = [
            keyboard::listen().filter_map(keys::key_press),
            event::listen_with(clicked),
            window::resize_events().map(|_| Message::Resized),
            Subscription::run_with(self.config.files.clone(), |files| {
                keymap::config::watch(files.clone())
            })
            .map(Message::Reconfigured),
        ];
        Subscription::batch(always.into_iter().chain(ticks).chain(rechecks))
    }

    /// Applies one message to the state, along with the widget operations
    /// it asks for
    pub(super) fn handled(self, message: Message) -> (Self, Task<Message>) {
        let app = match message {
            Message::Typed { press, repeat } => return self.typed(press, repeat),
            Message::Focus(focus) => return self.focused(focus),
            // Selection that moves, or rows that move around it, scroll into view
            Message::Select(id) => return self.with_selection(id).revealed(),
            Message::Step(offset) => return self.stepped(offset).revealed(),
            Message::Row(row) => return self.at_row(row).revealed(),
            Message::SwitchList(list) => return self.switched_to(list).revealed(),
            Message::StepList(offset) => return self.stepped_list(offset).revealed(),
            Message::Add(place) => return self.with_new_cue(place).revealed(),
            Message::Delete(count) => return self.without_selected(count).revealed(),
            Message::Undo => return self.undone().revealed(),
            Message::Redo => return self.redone().revealed(),
            Message::Resized => return self.revealed(),
            Message::Recheck => return self.recheck(),
            // Opener can take a moment to tell that it failed, so it runs
            // off the interface thread, where GO and Panic must never wait
            Message::Open(target) => {
                let open = Task::perform(async move { open_link(&target) }, Message::Opened);
                return (
                    Self {
                        unopened: None,
                        ..self
                    },
                    open,
                );
            }
            // GO and Panic close the help page, so that the show is in view
            // as it runs or stops
            Message::Go => {
                let go = Command::Go { list: self.list };
                Self {
                    help: false,
                    ..self.send(go)
                }
            }
            Message::Panic => Self {
                help: false,
                ..self.send(Command::Panic)
            },
            Message::PauseAll => self.send(Command::Pause(true)),
            Message::ResumeAll => self.send(Command::Pause(false)),
            Message::Stop(id) => self.send(Command::Stop(id)),
            Message::Tick(now) => {
                let elapsed = self.last_tick.map_or(TICK, |last| now - last);
                Self {
                    last_tick: Some(now),
                    ..self.send(Command::Tick(elapsed.as_secs_f64()))
                }
            }
            Message::StandbySelected => {
                let standby = Command::Standby {
                    list: self.list,
                    cue: self.selected_here(),
                };
                self.send(standby)
            }
            Message::ShowMode(show_mode) => self.send(Command::ShowMode(show_mode)),
            Message::Rechecked(loads) => self.rechecked(loads),
            // Keys typed halfway went by the bindings before, so the update
            // drops them, as it does for most messages
            Message::Reconfigured(config) => Self { config, ..self },
            Message::Opened(result) => Self {
                unopened: result.err(),
                ..self
            },
            Message::Clicked => self,
            Message::Edit(id, edit) => self.send(Command::Edit(id, edit)),
            Message::ToggleHelp => Self {
                help: !self.help,
                ..self
            },
            Message::CloseHelp => Self {
                help: false,
                ..self
            },
            Message::Tab(tab) => Self { tab, ..self },
            Message::StepTab(offset) => Self {
                tab: self.tab.stepped(offset),
                ..self
            },
            Message::NewKind(new_kind) => Self { new_kind, ..self },
        };
        (app, Task::none())
    }

    /// Hands a command to the back end
    pub(super) fn send(self, command: Command) -> Self {
        Self {
            workspace: self.workspace.apply(command),
            ..self
        }
    }

    /// Returns the state after one message, with the widget operations it
    /// asks for.  Anything the operator does other than typing drops a
    /// command typed halfway, so that a stray `d` cannot wait through a
    /// click and an edit to delete a cue.  The clock stops with the last
    /// running cue, so that time spent idle or paused never reaches the
    /// next tick, and a link that failed to open is forgotten once the help
    /// page that holds it closes.
    fn next(self, message: Message) -> (Self, Task<Message>) {
        let app = match message {
            Message::Typed { .. }
            | Message::Tick(_)
            | Message::Recheck
            | Message::Rechecked(_)
            | Message::Opened(_)
            | Message::Resized => self,
            _ => Self {
                keys: Vec::new(),
                ..self
            },
        };
        let (app, task) = app.handled(message);
        let last_tick = app.last_tick.filter(|_| app.running());
        let unopened = app.unopened.filter(|_| app.help);
        (
            Self {
                last_tick,
                unopened,
                ..app
            },
            task,
        )
    }

    /// Returns whether any fired cue counts time, which a paused cue does
    /// not
    fn running(&self) -> bool {
        self.workspace.active().iter().any(|active| !active.paused)
    }
}

/// Reports a press of any mouse button, captured by a widget or not, so
/// that a click drops a command typed halfway
fn clicked(event: Event, _status: event::Status, _window: window::Id) -> Option<Message> {
    matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))).then_some(Message::Clicked)
}

/// Opens a web page or a file in the program that the desktop opens it
/// with: through xdg-open on desktops that follow the XDG specifications,
/// and through the system's own opener elsewhere.  Returns why it failed,
/// if it did.
fn open_link(target: &OsStr) -> Result<(), String> {
    open::that_detached(target)
        .map_err(|error| format!("Cannot open {}: {error}", target.to_string_lossy()))
}

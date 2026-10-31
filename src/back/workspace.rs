// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Workspace, which holds the cue lists and the cues playing from them
//!
//! Playback here is bookkeeping without side effects: fired cues count
//! time, fades move levels, and stop cues end other cues.  The audio output
//! in [`super::Audio`] turns the result into sound.  A [`Command`] asks for
//! each change, and an [`Active`] records each cue that GO has fired.  The
//! rules of playback live in [`playback`] and [`stopping`], those that
//! change the cue lists in [`editing`], and undo and redo in [`history`].

mod active;
mod command;
mod editing;
mod history;
mod playback;
mod stopping;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

pub use active::{Active, Phase};
pub use command::{Command, Place};
pub use history::{Change, Turn};

use crate::back::{Cue, CueId, CueList, Kind, Media, Problem};
use history::History;

/// Show as a whole, with its cue lists, running cues, and editing lock
///
/// Front ends read a workspace freely but change it only through
/// [`Workspace::apply`], which keeps every rule about changes in one place.
#[derive(Debug, Clone, Default)]
pub struct Workspace {
    lists: Vec<CueList>,
    active: Vec<Active>,
    /// Lock on editing during a performance, which leaves playback alone
    show_mode: bool,
    /// Identity the next added cue receives
    next_id: u32,
    /// Serial number the next fired cue receives
    next_run: u32,
    /// Directory that relative file paths start from
    directory: PathBuf,
    /// Changes to the cues that undo and redo act on
    history: History,
}

impl Workspace {
    /// Creates a workspace in edit mode with nothing playing
    pub fn new(lists: Vec<CueList>) -> Self {
        let next_id = lists
            .iter()
            .flat_map(|list| &list.cues)
            .map(|cue| cue.id.0)
            .max()
            .unwrap_or(0)
            + 1;
        Self {
            lists,
            active: Vec::new(),
            show_mode: false,
            next_id,
            next_run: 0,
            directory: PathBuf::new(),
            history: History::default(),
        }
    }

    /// Returns the workspace with relative file paths starting from
    /// `directory`, the way QLab finds files next to its workspace file.
    /// Without one, they start from the current directory.
    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            ..self
        }
    }

    /// Returns where a file named by a cue is.  Spaces around the name do
    /// not count, as the check for a missing file has it, so that a pasted
    /// path with a trailing newline still loads.
    pub fn path(&self, file: &str) -> PathBuf {
        self.directory.join(file.trim())
    }

    pub fn lists(&self) -> &[CueList] {
        &self.lists
    }

    /// Returns the cues that GO has fired and that have not finished yet
    pub fn active(&self) -> &[Active] {
        &self.active
    }

    /// Returns whether editing is locked for a performance
    pub fn show_mode(&self) -> bool {
        self.show_mode
    }

    /// Returns the identity the next added cue will receive
    pub fn next_cue_id(&self) -> CueId {
        CueId(self.next_id)
    }

    pub fn cues(&self) -> impl Iterator<Item = &Cue> {
        self.lists.iter().flat_map(|list| &list.cues)
    }

    pub fn cue(&self, id: CueId) -> Option<&Cue> {
        self.cues().find(|cue| cue.id == id)
    }

    pub fn list_of(&self, id: CueId) -> Option<usize> {
        self.lists
            .iter()
            .position(|list| list.position(id).is_some())
    }

    pub fn active_of(&self, id: CueId) -> Option<&Active> {
        self.active.iter().find(|active| active.id == id)
    }

    /// Returns the reason a cue cannot play, if there is one
    pub fn problem<'a>(&self, cue: &'a Cue) -> Option<Problem<'a>> {
        match (cue.kind, &cue.media) {
            (Kind::Audio, _) if cue.file.trim().is_empty() => Some(Problem::NoFile),
            (Kind::Audio, Media::Unloaded) => Some(Problem::NotLoaded),
            (Kind::Audio, Media::Failed(reason)) => Some(Problem::LoadFailed(reason)),
            (Kind::Fade | Kind::Stop, _) => match cue.target.and_then(|id| self.cue(id)) {
                Some(target) if cue.can_target(target) => None,
                _ => Some(Problem::NoTarget),
            },
            _ => None,
        }
    }

    /// Returns every cue that cannot play, with the reason
    pub fn problems(&self) -> impl Iterator<Item = (&Cue, Problem<'_>)> {
        self.cues()
            .filter_map(|cue| self.problem(cue).map(|problem| (cue, problem)))
    }

    /// Returns the audio cues whose file has not been loaded, with the file
    pub fn unloaded(&self) -> impl Iterator<Item = (CueId, &str)> {
        self.files_where(|media| *media == Media::Unloaded)
    }

    /// Returns the audio cues whose file could not be played, with the file
    pub fn failed(&self) -> impl Iterator<Item = (CueId, &str)> {
        self.files_where(|media| matches!(media, Media::Failed(_)))
    }

    /// Returns the audio cues that name a file and whose media passes `test`,
    /// with the file
    fn files_where(&self, test: fn(&Media) -> bool) -> impl Iterator<Item = (CueId, &str)> {
        self.cues()
            .filter(move |cue| {
                cue.kind == Kind::Audio && test(&cue.media) && !cue.file.trim().is_empty()
            })
            .map(|cue| (cue.id, cue.file.as_str()))
    }

    /// Returns the workspace with a command applied.  Commands that change
    /// cues do nothing in show mode, which only allows playback and moves
    /// of standby, and undo and redo there take back and make again only
    /// moves of standby.
    pub fn apply(self, command: Command) -> Self {
        match command {
            Command::Go { list } => self.go(list),
            Command::Panic => self.panic(),
            Command::Pause(paused) => self.paused(paused),
            Command::Stop(id) => self.stopped(id),
            Command::Tick(seconds) => self.tick(seconds),
            Command::Standby { list, cue } => self.moved_standby(list, cue),
            Command::ShowMode(show_mode) => Self { show_mode, ..self },
            Command::Loaded { cue, file, media } => self.loaded(cue, &file, media),
            Command::Undo => self.undone(),
            Command::Redo => self.redone(),
            Command::Add { .. } | Command::Delete(_) | Command::Edit(..) if self.show_mode => self,
            Command::Add { list, kind, at } => self.with_new_cue(list, kind, at),
            Command::Delete(ids) => self.without(&ids),
            Command::Edit(id, edit) => self.edited(id, edit),
        }
    }
}

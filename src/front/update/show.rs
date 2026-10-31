// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shows that the operator opens and saves, and the unsaved changes that
//! cinq keeps for them
//!
//! A show lives in a file of its own, which the operator names on the
//! command line or picks in the desktop's file chooser.  Opening reads the
//! file and saving writes it, both on a thread of their own, as any reading
//! of files is.  Changes that are not saved never go away unasked: as they
//! happen, the [`Keeper`] keeps them in a file in the state directory of
//! cinq, which the next opening of the show reads, so that neither closing
//! the window, nor opening another show, nor a crash loses them.  The kept
//! changes come back as one change, which undo takes back to the show as
//! saved, so that nothing has to ask first.

#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use iced::Task;

use super::on_own_thread;
use crate::back::{Command, Workspace};
use crate::front::keymap::Action;
use crate::front::view::LINE;
use crate::front::{App, Message, read};

/// Ending of the name of a show file, which is TOML
const EXTENSION: &str = "cinq.toml";

/// Show on screen, with the file that holds it
#[derive(Debug, Clone, Default)]
pub struct Show {
    /// Serial number of the opening that brought the show, which tells a
    /// show that moved to another file apart from another show
    id: u64,
    /// File that holds the show, or none for a show not saved yet
    file: Option<PathBuf>,
    /// Show as its file holds it
    saved: String,
    /// Show as its file would hold it now
    text: String,
    /// Count of the changes to the show or to its file
    version: u64,
}

impl Show {
    /// Returns whether the show holds changes that its file does not
    pub fn unsaved(&self) -> bool {
        self.text != self.saved
    }

    /// Returns the name that the show goes by: the name of its file, or
    /// `untitled` while it has none
    pub fn name(&self) -> Cow<'_, str> {
        self.file
            .as_deref()
            .and_then(Path::file_name)
            .map_or(Cow::Borrowed("untitled"), |name| name.to_string_lossy())
    }

    /// Returns the serial number of the opening that brought the show
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the directory of the file of the show, which the relative
    /// paths of its sound files go from, or none while it has no file
    fn directory(&self) -> Option<&Path> {
        self.file.as_deref().and_then(Path::parent)
    }
}

/// What reading a show found, along with the unsaved changes kept for it
#[derive(Debug, Clone)]
pub struct Read {
    /// File of the show, or none for a new show without one
    file: Option<PathBuf>,
    /// Directory that the relative paths of its sound files go from
    directory: PathBuf,
    /// Text of the file, or none when there is no such file yet, or why it
    /// cannot be read
    text: Result<Option<String>, String>,
    /// Unsaved changes kept for the show, if there are any, or why they
    /// cannot be read
    kept: Result<Option<String>, String>,
}

impl Read {
    /// Reads the show in `file`, or a new show without one, along with the
    /// unsaved changes kept for it in `unsaved`.  A new show finds its sound
    /// files from the directory that cinq started in.
    fn new(file: Option<PathBuf>, unsaved: Option<&Path>) -> Self {
        let directory = file
            .as_deref()
            .and_then(Path::parent)
            .map_or_else(|| env::current_dir().unwrap_or_default(), Path::to_owned);
        let text = file.as_deref().map_or(Ok(None), read);
        let kept = unsaved.map_or(Ok(None), |unsaved| {
            read(&kept_file(unsaved, file.as_deref()))
        });
        Self {
            file,
            directory,
            text,
            kept,
        }
    }
}

/// What saving the show found
#[derive(Debug, Clone)]
pub struct Saved {
    /// Identity of the show that was saved, so that a show opened since is
    /// left alone
    show: u64,
    /// File that the show went to
    file: PathBuf,
    /// Show as the file holds it now
    text: String,
    /// Why saving failed, if it did
    result: Result<(), String>,
}

/// Returns the directory that keeps unsaved changes: `cinq/unsaved` in the
/// state directory, or in the directory of local data on platforms without
/// one
pub(super) fn unsaved_directory() -> Option<PathBuf> {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .map(|directory| directory.join("cinq").join("unsaved"))
}

/// Starts reading the show in `file`, or a new show without one, along with
/// the unsaved changes kept for it in `unsaved`
pub(super) fn open(file: Option<PathBuf>, unsaved: Option<PathBuf>) -> Task<Message> {
    on_own_thread(move || Read::new(file, unsaved.as_deref()))
        .map(|read| Message::Opened(Box::new(read)))
}

impl App {
    /// Asks which show to open, starting in the directory of the show on
    /// screen, and starts reading the show chosen
    pub(super) fn choose_show(&self) -> Task<Message> {
        let chooser = chooser(self.show.directory()).set_title("Open show");
        let unsaved = self.unsaved.clone();
        Task::future(chooser.pick_file())
            .and_then(move |chosen| open(Some(chosen.path().to_owned()), unsaved.clone()))
    }

    /// Asks where to save the show, and saves it there
    pub(super) fn choose_file(&self) -> Task<Message> {
        let chooser = chooser(self.show.directory())
            .set_title("Save show")
            .set_file_name(match &self.show.file {
                Some(_) => self.show.name().into_owned(),
                None => format!("untitled.{EXTENSION}"),
            });
        Task::future(chooser.save_file())
            .and_then(|chosen| Task::done(Message::SaveTo(with_extension(chosen.path()))))
    }

    /// Saves the show in its file, or asks where to save a show without one
    pub(super) fn save(&self) -> Task<Message> {
        match self.show.file.clone() {
            Some(file) => self.save_to(file),
            None => self.choose_file(),
        }
    }

    /// Starts saving the show in `file`, with the relative paths of its
    /// sound files going from the directory of the file
    pub(super) fn save_to(&self, file: PathBuf) -> Task<Message> {
        let show = self.show.id;
        let text = self.workspace.show(file.parent());
        on_own_thread(move || {
            let result = write(&file, &text, true).map_err(|error| error.to_string());
            Saved {
                show,
                file,
                text,
                result,
            }
        })
        .map(Message::Saved)
    }

    /// Records what saving the show found.  The show then goes by the file
    /// it went to, and any change made while it was being saved stays
    /// unsaved.  A save that ends after another show has opened leaves that
    /// show alone.
    pub(super) fn saved(self, saved: Saved) -> Self {
        match saved.result {
            Ok(()) if saved.show != self.show.id => self,
            Ok(()) => {
                let show = Show {
                    text: self.workspace.show(saved.file.parent()),
                    file: Some(saved.file),
                    saved: saved.text,
                    version: self.show.version + 1,
                    ..self.show
                };
                Self {
                    told: Some(format!("Saved {}", show.name())),
                    file_problem: None,
                    show,
                    ..self
                }
            }
            Err(problem) => Self {
                file_problem: Some(format!("Cannot save {}: {problem}", saved.file.display())),
                ..self
            },
        }
    }

    /// Puts the show that `read` found on screen in place of the one
    /// there, and brings back the unsaved changes kept for it as one
    /// change.  The show that was on screen keeps its own unsaved changes,
    /// which the keeper has written as they happened.
    pub(super) fn opened(self, read: Read) -> Self {
        let name = read.file.as_deref().map_or_else(
            || "a new show".to_owned(),
            |file| file.display().to_string(),
        );
        let workspace = read.text.and_then(|text| match text {
            Some(text) => Workspace::from_show(&text, &read.directory),
            None => Ok(Workspace::default().directory(&read.directory)),
        });
        let workspace = match workspace {
            Ok(workspace) => workspace,
            Err(problem) => {
                return Self {
                    file_problem: Some(format!("Cannot open {name}: {problem}")),
                    ..self
                };
            }
        };
        let base = read.file.as_deref().and_then(Path::parent);
        let saved = workspace.show(base);
        // Show that goes keeps its unsaved changes, which opening it again
        // brings back
        let told = self.show.unsaved().then(|| {
            let kept = format!("Kept the unsaved changes to {}", self.show.name());
            match self.config.keymap.hint(Action::Open, Action::Open.name()) {
                Some(hint) => format!("{kept}{LINE}{hint}"),
                None => kept,
            }
        });
        let opened = Self {
            selected: workspace.lists().first().and_then(|cues| cues.standby),
            workspace,
            show: Show {
                id: self.show.id + 1,
                file: read.file.clone(),
                saved: saved.clone(),
                text: saved,
                version: self.show.version + 1,
            },
            list: 0,
            file_problem: None,
            told,
            ..self
        };
        let kept = read.kept.and_then(|kept| {
            kept.map(|kept| Workspace::from_show(&kept, &read.directory))
                .transpose()
        });
        match kept {
            Ok(Some(kept)) if kept.show(base) != opened.show.saved => {
                opened.send(Command::Restore(kept.lists().to_vec()))
            }
            Ok(_) => opened,
            Err(problem) => Self {
                file_problem: Some(format!(
                    "Cannot bring back the unsaved changes to {name}: {problem}"
                )),
                ..opened
            },
        }
    }

    /// Returns the state with the show as its file would hold it brought up
    /// to date, as after a change to the cues
    pub(super) fn with_text(self) -> Self {
        let text = self.workspace.show(self.show.directory());
        if text == self.show.text {
            return self;
        }
        Self {
            show: Show {
                text,
                version: self.show.version + 1,
                ..self.show
            },
            told: None,
            ..self
        }
    }
}

/// Keeper of unsaved changes, which writes them to the kept file of their
/// show as they happen, and removes the file once they are gone
///
/// The keeper writes at once rather than on a thread of its own, so that
/// its writes stay in order and none is left to make when the window
/// closes.  The files are small and the state directory is local, which
/// keeps the wait short.
#[derive(Debug, Default)]
pub struct Keeper {
    /// Directory that keeps unsaved changes, if the platform has one
    unsaved: Option<PathBuf>,
    /// Identity and version of the show last seen
    seen: Option<(u64, u64)>,
    /// Kept file written last, with the identity of its show
    kept: Option<(u64, PathBuf)>,
}

impl Keeper {
    /// Returns a keeper of unsaved changes in `unsaved`
    pub fn new(unsaved: Option<PathBuf>) -> Self {
        Self {
            unsaved,
            ..Self::default()
        }
    }

    /// Keeps the unsaved changes of `show` in its kept file, or removes the
    /// file once the show has none, as well as the kept file that the show
    /// had before it moved to another file.  The kept file of another show
    /// stays, for that show to bring back.  Returns why keeping failed, if
    /// it did.
    pub fn keep(&mut self, show: &Show) -> Option<String> {
        if self.seen == Some((show.id, show.version)) {
            return None;
        }
        self.seen = Some((show.id, show.version));
        let wanted = self
            .unsaved
            .as_deref()
            .filter(|_| show.unsaved())
            .map(|unsaved| kept_file(unsaved, show.file.as_deref()));
        let gone = self
            .kept
            .take()
            .filter(|(id, kept)| *id == show.id && wanted.as_ref() != Some(kept))
            .map(|(_, kept)| kept);
        let removed = gone.map_or(Ok(()), |gone| remove(&gone));
        let written = wanted
            .as_ref()
            .map_or(Ok(()), |wanted| write(wanted, &show.text, false));
        self.kept = wanted.map(|wanted| (show.id, wanted));
        match (removed.and(written), &self.kept) {
            (Err(error), _) => Some(format!("Cannot keep the unsaved changes: {error}")),
            (Ok(()), None) if show.unsaved() => {
                Some("Cannot keep the unsaved changes: there is no state directory".to_owned())
            }
            (Ok(()), _) => None,
        }
    }
}

/// Returns the desktop's file chooser for shows, which starts in
/// `directory`, or wherever the desktop starts without one
fn chooser(directory: Option<&Path>) -> rfd::AsyncFileDialog {
    let chooser = rfd::AsyncFileDialog::new().add_filter("Shows", &[EXTENSION]);
    match directory {
        Some(directory) => chooser.set_directory(directory),
        None => chooser,
    }
}

/// Returns `file` with the ending of a show file, unless it ends in
/// `.toml` already
fn with_extension(file: &Path) -> PathBuf {
    if file
        .extension()
        .is_some_and(|extension| extension == "toml")
    {
        file.to_owned()
    } else {
        file.with_added_extension(EXTENSION)
    }
}

/// Returns the file in `unsaved` that keeps the unsaved changes of the
/// show in `file`, or of a new show without one: by the path of the show's
/// file under `shows`, or as `untitled.cinq.toml` for a new show
fn kept_file(unsaved: &Path, file: Option<&Path>) -> PathBuf {
    match file {
        Some(file) => unsaved.join("shows").join(
            file.components()
                .filter_map(|component| match component {
                    Component::Normal(name) => Some(name),
                    _ => None,
                })
                .collect::<PathBuf>(),
        ),
        None => unsaved.join(format!("untitled.{EXTENSION}")),
    }
}

/// Writes `text` to a file next to `file`, which then takes its place, so
/// that a failure halfway leaves the file as it was.  Waiting for the disk,
/// as saving a show does, makes the text outlast a power cut as well.
fn write(file: &Path, text: &str, wait_for_disk: bool) -> io::Result<()> {
    let directory = file.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(text.as_bytes())?;
    if wait_for_disk {
        temporary.as_file().sync_all()?;
    }
    temporary
        .persist(file)
        .map(drop)
        .map_err(|error| error.error)
}

/// Removes a file, which is no failure when it is gone already
fn remove(file: &Path) -> io::Result<()> {
    match fs::remove_file(file) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

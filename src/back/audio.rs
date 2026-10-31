// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound output, the one part of the program with side effects
//!
//! [`Audio::sync`] runs after every change to the workspace.  It loads the
//! files that audio cues name, then makes the sound match the workspace: it
//! starts the audio cues that the workspace has sounding, follows their
//! level and pause state, and stops the sounds whose cue has stopped.  The
//! workspace stays the only record of what should be heard, so a sound
//! that cannot start goes back to it as a file that cannot be played.

mod voice;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use itertools::{Either, Itertools};
use kira::sound::FromFileError;
use kira::sound::streaming::StreamingSoundData;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{Active, Command, Cue, CueId, Media, Workspace};
use voice::Voice;

/// Sound output for a workspace
pub struct Audio {
    /// Audio device, or the reason it could not be opened
    output: Result<AudioManager, String>,
    /// Sounds of the audio cues that are sounding
    voices: HashMap<CueId, Voice>,
}

impl Default for Audio {
    /// Returns an output without a device, which plays nothing
    fn default() -> Self {
        Self {
            output: Err("no audio device opened".to_owned()),
            voices: HashMap::new(),
        }
    }
}

impl Audio {
    /// Opens the default audio device.  Without one, the workspace still
    /// runs and files still load, but nothing is heard.
    pub fn open() -> Self {
        Self {
            output: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .map_err(|error| error.to_string()),
            ..Self::default()
        }
    }

    /// Returns why the audio device could not be opened, if it could not
    pub fn error(&self) -> Option<&str> {
        self.output.as_ref().err().map(String::as_str)
    }

    /// Loads the files that the workspace has not loaded yet, then makes the
    /// sound match the workspace.  Returns the workspace with what loading
    /// found, including the files whose sound could not start.
    pub fn sync(&mut self, workspace: Workspace) -> Workspace {
        let unloaded = workspace
            .unloaded()
            .map(|(cue, file)| (cue, file.to_owned(), workspace.path(file)))
            .collect();
        let workspace = load_files(unloaded)
            .into_iter()
            .fold(workspace, Workspace::apply);
        let failures = self.play(&workspace);
        failures.into_iter().fold(workspace, Workspace::apply)
    }

    /// Starts, follows, and stops sounds, so that each sounding audio cue
    /// plays at the level and pause state of its run, and nothing else
    /// plays.  Returns the commands that record the sounds that could not
    /// start.
    fn play(&mut self, workspace: &Workspace) -> Vec<Command> {
        let Ok(manager) = &mut self.output else {
            return Vec::new();
        };
        let sounding: HashMap<CueId, (&Cue, &Active)> = workspace
            .active()
            .iter()
            .filter(|active| active.sounding())
            .filter_map(|active| workspace.cue(active.id).map(|cue| (cue.id, (cue, active))))
            .collect();
        // Sounds whose cue has stopped, or which GO has fired again
        let (mut kept, ended): (HashMap<_, _>, HashMap<_, _>) = std::mem::take(&mut self.voices)
            .into_iter()
            .partition(|(id, voice)| {
                sounding
                    .get(id)
                    .is_some_and(|(_, active)| active.run == voice.run)
            });
        ended.into_values().for_each(Voice::stop);
        let (voices, failures) = sounding.into_iter().partition_map(|(id, (cue, active))| {
            let voice = kept.remove(&id).map_or_else(
                || Voice::start(manager, &workspace.path(&cue.file), active),
                Ok,
            );
            match voice {
                Ok(voice) => Either::Left((id, voice.followed(active))),
                Err(reason) => Either::Right(Command::Loaded {
                    cue: id,
                    file: cue.file.clone(),
                    media: Media::Failed(reason),
                }),
            }
        });
        self.voices = voices;
        failures
    }
}

/// Reads the files of cues, each one named as its cue names it and paired
/// with where it is, and returns the commands that record what reading
/// found.  Each distinct file is read once, and the files are read in
/// parallel, which shortens opening a large show whose files are not cached
/// yet.  Reading can block on a slow drive.
pub fn load_files(files: Vec<(CueId, String, PathBuf)>) -> Vec<Command> {
    // Collected first, so that rayon reads no file or a single one on this
    // thread instead of waking its pool
    let paths: Vec<&Path> = files
        .iter()
        .map(|(_, _, path)| path.as_path())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let found: HashMap<&Path, Media> = paths
        .into_par_iter()
        .map(|path| (path, load(path)))
        .collect();
    files
        .iter()
        .map(|(cue, file, path)| Command::Loaded {
            cue: *cue,
            file: file.clone(),
            media: found[path.as_path()].clone(),
        })
        .collect()
}

/// Reads how long a sound file plays, or why it cannot be played.  This
/// opens the file and reads its format without decoding the sound.
fn load(file: &Path) -> Media {
    match StreamingSoundData::from_file(file) {
        Ok(data) => Media::Loaded {
            duration: data.duration().as_secs_f32(),
        },
        Err(error) => Media::Failed(reason(&error)),
    }
}

/// Describes why a sound file cannot be played
fn reason(error: &FromFileError) -> String {
    match error {
        FromFileError::IoError(error) if error.kind() == io::ErrorKind::NotFound => {
            "file not found".to_owned()
        }
        error => error.to_string(),
    }
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watching the files that may hold the configuration, so that the keys
//! follow the file while the program runs
//!
//! Editors often save a file by writing another and renaming it over the
//! first, which ends any watch on the file itself, so the watcher watches
//! the directory that holds each file instead.  While that directory is
//! missing, it watches the one above, so that making the directory and
//! then the file comes through as well.  Each burst of changes reads the
//! files once, after it settles.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use super::{Config, load};

/// Time that changes must settle for before the files are read again,
/// which lets an editor finish saving
const SETTLE: Duration = Duration::from_millis(200);

/// Watcher of the directories, which stops watching once it is dropped
type Watcher = Debouncer<RecommendedWatcher, RecommendedCache>;

/// Returns the configuration that `files` hold once the watches are in
/// place, so that no change made before goes unseen, and again after each
/// change to the files.  Without a watcher, the configuration comes once,
/// with a problem that tells why the changes do not apply.
pub fn watch(files: Vec<PathBuf>) -> impl Stream<Item = Config> {
    stream::channel(1, async move |mut output| {
        let (sender, mut events) = mpsc::unbounded();
        let watcher = new_debouncer(SETTLE, None, move |result| {
            // Channel closes only once the stream has gone
            let _ = sender.unbounded_send(result);
        });
        let mut watcher = match watcher {
            Ok(watcher) => watcher,
            Err(error) => {
                let config = load(&files);
                let problem = format!("is not watched, so changes apply after a restart: {error}");
                let problems = [config.problems, vec![problem]].concat();
                let _ = output.send(Config { problems, ..config }).await;
                return;
            }
        };
        let mut watched = rewatch(&mut watcher, BTreeSet::new(), &files);
        let _ = output.send(load(&files)).await;
        while let Some(result) = events.next().await {
            if touches(&result, &files) {
                watched = rewatch(&mut watcher, watched, &files);
                let _ = output.send(load(&files)).await;
            }
        }
    })
}

/// Returns whether a burst of changes touches one of `files`, or the
/// directory that holds it.  A watch that failed may have missed a change,
/// so it counts as touching them all.
fn touches(result: &DebounceEventResult, files: &[PathBuf]) -> bool {
    match result {
        Ok(events) => events.iter().flat_map(|event| &event.paths).any(|path| {
            files
                .iter()
                .any(|file| file == path || file.parent() == Some(path))
        }),
        Err(_) => true,
    }
}

/// Watches the [`directory`] of each of `files`, and stops watching the
/// directories of `watched` that none of them needs any more.  Returns the
/// directories watched, which leave out any that cannot be watched, for
/// the next change to try again.
fn rewatch(
    watcher: &mut Watcher,
    watched: BTreeSet<PathBuf>,
    files: &[PathBuf],
) -> BTreeSet<PathBuf> {
    let wanted: BTreeSet<PathBuf> = files.iter().filter_map(|file| directory(file)).collect();
    // Directory that has gone took its watch with it, so a failure is moot
    watched.difference(&wanted).for_each(|gone| {
        let _ = watcher.unwatch(gone);
    });
    wanted
        .into_iter()
        .filter(|directory| {
            watched.contains(directory)
                || watcher
                    .watch(directory, RecursiveMode::NonRecursive)
                    .is_ok()
        })
        .collect()
}

/// Returns the directory to watch for `file`: the one that holds it, or
/// the one above while that is missing
fn directory(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .take(2)
        .find(|directory| directory.is_dir())
        .map(Path::to_owned)
}

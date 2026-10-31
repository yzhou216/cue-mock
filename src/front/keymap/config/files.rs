// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the configuration file is, and reading it

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Returns the directories that may hold the configuration, most important
/// first: the `cinq` directory of each configuration directory of the XDG
/// Base Directory specification
#[cfg(unix)]
pub(super) fn directories() -> Vec<PathBuf> {
    let base = xdg::BaseDirectories::with_prefix("cinq");
    base.get_config_home()
        .into_iter()
        .chain(base.get_config_dirs())
        .collect()
}

/// Returns the directories that may hold the configuration: the `cinq`
/// directory of the platform's own configuration directory
#[cfg(not(unix))]
pub(super) fn directories() -> Vec<PathBuf> {
    dirs::config_dir()
        .into_iter()
        .map(|directory| directory.join("cinq"))
        .collect()
}

/// Reads a file, or returns `None` when there is none
pub(super) fn read(file: &Path) -> Option<Result<String, String>> {
    match fs::read_to_string(file) {
        Ok(text) => Some(Ok(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => Some(Err(format!("cannot be read: {error}"))),
    }
}

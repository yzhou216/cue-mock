// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Command line, which can name the directory that the key bindings come
//! from

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use clap::{Parser, ValueHint};

/// Version with the copyright and license notice that `--version` prints
/// after it, as the GNU Coding Standards set out
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nCopyright (C) 2026 Yiyu Zhou",
    "\nLicense GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.",
    "\nThis is free software: you are free to change and redistribute it.",
    "\nThere is NO WARRANTY, to the extent permitted by law.",
);

// Name, version, and description come from the manifest of the crate
#[derive(Debug, Parser)]
#[command(version, long_version = LONG_VERSION, about)]
pub struct Cli {
    /// Read config.toml from DIRECTORY instead of the configuration
    /// directories
    #[arg(long, value_name = "DIRECTORY", value_hint = ValueHint::DirPath)]
    pub config_dir: Option<PathBuf>,
}

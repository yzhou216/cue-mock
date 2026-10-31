// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the command line, and of the desktop entry that runs it, with
//! its icon

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser};

use super::Cli;

/// Desktop entry, through which application launchers list cinq
const DESKTOP_ENTRY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/share/applications/cinq.desktop"
));

#[test]
fn command_line_is_well_formed() {
    Cli::command().debug_assert();
}

#[test]
fn desktop_entry_names_describes_and_runs_cinq_as_its_manifest_does() {
    let value = |key: &str| {
        DESKTOP_ENTRY
            .lines()
            .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
    };
    assert_eq!(value("Name"), Some(env!("CARGO_PKG_NAME")));
    assert_eq!(value("Icon"), Some(env!("CARGO_PKG_NAME")));
    assert_eq!(value("Comment"), Some(env!("CARGO_PKG_DESCRIPTION")));
    assert_eq!(value("Exec"), Some(concat!(env!("CARGO_PKG_NAME"), " %f")));
}

#[test]
fn icon_that_the_desktop_entry_names_is_one_that_the_build_draws() {
    let icon = Path::new(env!("CINQ_ICON_DIR")).join(concat!(env!("CARGO_PKG_NAME"), ".svg"));
    assert!(icon.is_file(), "{}", icon.display());
}

#[test]
fn config_dir_takes_its_directory_after_an_equals_sign_or_a_space() {
    let config_dir = |args: &[&str]| Cli::try_parse_from(args).map(|cli| cli.config_dir).ok();
    let directory = Some(Some(PathBuf::from("/config")));
    assert_eq!(config_dir(&["cinq"]), Some(None));
    assert_eq!(config_dir(&["cinq", "--config-dir=/config"]), directory);
    assert_eq!(config_dir(&["cinq", "--config-dir", "/config"]), directory);
    assert_eq!(config_dir(&["cinq", "--config-dir"]), None);
}

#[test]
fn version_tells_the_copyright_and_license() {
    let version = Cli::command().render_long_version();
    assert!(
        version.contains("Copyright (C) 2026 Yiyu Zhou"),
        "{version}"
    );
    assert!(version.contains("License GPLv3+"), "{version}");
}

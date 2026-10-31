// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the command line

use std::path::PathBuf;

use clap::{CommandFactory, Parser};

use super::Cli;

#[test]
fn command_line_is_well_formed() {
    Cli::command().debug_assert();
}

#[test]
fn config_dir_takes_its_directory_after_an_equals_sign_or_a_space() {
    let config_dir = |args: &[&str]| Cli::try_parse_from(args).map(|cli| cli.config_dir).ok();
    let show = Some(Some(PathBuf::from("/show")));
    assert_eq!(config_dir(&["cinq"]), Some(None));
    assert_eq!(config_dir(&["cinq", "--config-dir=/show"]), show);
    assert_eq!(config_dir(&["cinq", "--config-dir", "/show"]), show);
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

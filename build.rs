// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Renders the manual pages for the platform of the build, when
//! `CINQ_MAN_DIR` names a directory to render them into: cinq(1) from the
//! command line, and cinq(5) and cinq-show(5) from their scdoc sources by
//! scdoc, which has to be installed then.  The pages are dated by
//! `SOURCE_DATE_EPOCH`, which reproducible builds set, or else by the day of
//! the build.  Draws the logo as the icon of the desktop entry as well, when
//! `CINQ_ICON_DIR` names a directory to draw it into.  A relative directory
//! goes from the root of the package.

// Modules of the program that the build compiles as well, which keep their
// own modules where the program finds them
#[path = "src"]
mod src {
    // Command line, of which the build reads only the definition
    #[allow(dead_code)]
    mod cli;
    pub mod logo;
    pub mod man;
}

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

use time::OffsetDateTime;

use src::logo;
use src::man::{self, Platform};

fn main() {
    [
        "src/cli.rs",
        "src/logo.rs",
        "src/man.rs",
        "man/cinq.5.scd",
        "man/cinq-show.5.scd",
    ]
    .iter()
    .for_each(|file| println!("cargo::rerun-if-changed={file}"));
    ["CINQ_ICON_DIR", "CINQ_MAN_DIR", "SOURCE_DATE_EPOCH"]
        .iter()
        .for_each(|variable| println!("cargo::rerun-if-env-changed={variable}"));
    let rendered = env::var_os("CINQ_MAN_DIR").map(|directory| render(Path::new(&directory)));
    if let Some(Err(error)) = rendered {
        println!("cargo::error=cannot render the manual pages: {error}");
    }
    let drawn = env::var_os("CINQ_ICON_DIR").map(|directory| draw(Path::new(&directory)));
    if let Some(Err(error)) = drawn {
        println!("cargo::error=cannot draw the icon: {error}");
    }
}

/// Draws the logo into `directory` as the icon of the desktop entry
fn draw(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    fs::write(directory.join("cinq.svg"), logo::svg())
}

/// Renders the manual pages for the platform of the build into `directory`
fn render(directory: &Path) -> io::Result<()> {
    let target = env::var("CARGO_CFG_TARGET_OS").expect("cargo names the target");
    let platform = Platform::of(&target);
    fs::create_dir_all(directory)?;
    fs::write(
        directory.join("cinq.1"),
        man::command_page(platform, &date()),
    )?;
    man::SCDOC.iter().try_for_each(|(name, source)| {
        scdoc(&man::for_platform(source, platform), &directory.join(name))
    })
}

/// Returns the date of the build, written YYYY-MM-DD: that of
/// `SOURCE_DATE_EPOCH`, in seconds since 1970, when it is set, or else today
fn date() -> String {
    let time = env::var_os("SOURCE_DATE_EPOCH").map_or_else(OffsetDateTime::now_utc, |seconds| {
        seconds
            .to_str()
            .and_then(|seconds| seconds.parse().ok())
            .and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds).ok())
            .expect("SOURCE_DATE_EPOCH holds a time in seconds since 1970")
    });
    time.date().to_string()
}

/// Renders the scdoc `source` into the manual page `page` with scdoc, which
/// dates it as [`date`] does.  The source reaches scdoc through a file in
/// the output directory of the build, and the page is written only once
/// scdoc has rendered all of it, so that a failure leaves no page behind.
fn scdoc(source: &str, page: &Path) -> io::Result<()> {
    let name = page
        .file_name()
        .expect("a page is a file")
        .to_string_lossy();
    let input = Path::new(&env::var_os("OUT_DIR").expect("cargo names the output directory"))
        .join(format!("{name}.scd"));
    fs::write(&input, source)?;
    let output = Command::new("scdoc")
        .stdin(File::open(&input)?)
        .output()
        .map_err(|error| io::Error::new(error.kind(), format!("cannot run scdoc: {error}")))?;
    if output.status.success() {
        fs::write(page, output.stdout)
    } else {
        let reason = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::other(format!("scdoc failed: {}", reason.trim())))
    }
}

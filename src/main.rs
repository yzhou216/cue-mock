// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue-based show controller in the spirit of QLab
//!
//! Audio cues play sound files, fade cues change the level of a playing
//! audio cue, stop cues end another cue, and wait and memo cues pace and
//! annotate the show.  GO fires the standby cue together with every cue that
//! auto-continue or auto-follow chains to it.  Selection and standby are
//! independent: inspecting a cue never changes what the next GO fires.
//!
//! The back end in [`back`] holds the show, every rule for changing it, and
//! the audio output, and it does not depend on iced.  The front end in
//! [`front`] draws the show with iced and turns operator input into
//! back-end commands.  The command line in [`cli`] can name the directory
//! that the key bindings come from.

mod back;
mod cli;
mod front;

use clap::Parser;

fn main() -> iced::Result {
    let cli = cli::Cli::parse();
    front::run(cli.config_dir)
}

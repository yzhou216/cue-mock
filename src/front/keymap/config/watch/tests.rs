// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of watching the configuration files, whose keys follow the file
//! as it comes, as an editor saves it, and as it goes

use std::sync::mpsc;
use std::time::Duration;
use std::{env, fs, io, iter, process, thread};

use iced::futures::{StreamExt, executor, future};

use super::watch;
use crate::front::keymap::Preset;
use crate::front::keymap::config::{Config, Source};

/// Time that the watcher has to see a change before the test fails
const PATIENCE: Duration = Duration::from_secs(10);

/// Waits for a configuration from the watcher that `wanted` accepts
fn until(configs: &mpsc::Receiver<Config>, wanted: impl Fn(&Config) -> bool) {
    let found = iter::from_fn(|| configs.recv_timeout(PATIENCE).ok()).any(|config| wanted(&config));
    assert!(found, "the watcher sent no such configuration");
}

/// Accepts a configuration read from a file that starts from `preset`
fn from_file(preset: Preset) -> impl Fn(&Config) -> bool {
    move |config| matches!(config.source, Source::File(_)) && config.keymap.name() == preset.name()
}

/// Accepts a configuration without a file
fn defaults(config: &Config) -> bool {
    matches!(config.source, Source::Defaults(_))
}

#[test]
fn keys_follow_the_file_as_it_comes_changes_and_goes() -> io::Result<()> {
    let home = env::temp_dir().join(format!("cinq-watch-{}", process::id()));
    let directory = home.join("cinq");
    let file = directory.join("config.toml");
    fs::create_dir_all(&home)?;
    let (sender, configs) = mpsc::channel();
    let files = vec![file.clone()];
    thread::spawn(move || {
        executor::block_on(watch(files).for_each(|config| {
            let _ = sender.send(config);
            future::ready(())
        }));
    });
    until(&configs, defaults);
    fs::create_dir(&directory)?;
    fs::write(&file, "keymap = \"cua\"\n")?;
    until(&configs, from_file(Preset::Cua));
    // Saved as editors often save, by renaming another file over it
    let draft = directory.join("config.toml.draft");
    fs::write(&draft, "keymap = \"vi\"\n")?;
    fs::rename(&draft, &file)?;
    until(&configs, from_file(Preset::Vi));
    fs::remove_file(&file)?;
    until(&configs, defaults);
    fs::remove_dir_all(&home)
}

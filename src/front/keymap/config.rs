// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration file, which chooses a keymap and binds actions to keys
//!
//! The file is `cinq/config.toml` in a configuration directory, unless the
//! command line names a directory of its own to read `config.toml` from.
//! On every
//! POSIX system, the directories follow the XDG Base Directory
//! specification: `$XDG_CONFIG_HOME`, or `~/.config` unless it is set,
//! comes first, then each directory of `$XDG_CONFIG_DIRS`, or `/etc/xdg`
//! unless it is set, and the first file found applies.  Elsewhere the
//! directory is the platform's own, as the dirs crate finds it.  [`watch`]
//! reads the files again whenever one of them changes, so that the keys
//! follow the file while the program runs.
//!
//! `keymap` names the keymap that the keys start from, which is vi unless
//! the file names another: a shipped one, or one of the `[keymaps]` table,
//! which starts from the keymap that its `from` names, or from no keys at
//! all without one.  A keymap of the file's own and the `[keys]` table bind
//! each action they name to a key sequence or a list of them, in place of
//! the keys that the keymap they build on gives it:
//!
//! ```toml
//! keymap = "booth"
//!
//! [keymaps.booth]
//! from = "cua"
//! go = ["Space", "F5"]
//!
//! [keys]
//! delete = []
//! ```

mod files;
mod watch;

use std::path::{self, Path, PathBuf};

use itertools::Itertools;
use toml::{Table, Value};

use super::{Action, Keymap, Preset, Press, press};
pub use watch::watch;

/// Settings that the file can hold
const SETTINGS: [&str; 3] = ["keymap", "keymaps", "keys"];

/// Key bindings, along with where they came from and what was wrong with
/// the file they came from
#[derive(Debug, Clone)]
pub struct Config {
    pub keymap: Keymap,
    pub source: Source,
    /// Problems with the file, each of which leaves the keys of the keymap
    /// in place, and each written to follow the name of the file
    pub problems: Vec<String>,
    /// Files that may hold the configuration, most important first, which
    /// the keys follow as they change
    pub files: Vec<PathBuf>,
}

/// Entry of the `[keys]` table: an action and the key sequences bound to it
type Entry = (Action, Vec<Vec<Press>>);

/// Where the key bindings came from
#[derive(Debug, Clone)]
pub enum Source {
    /// Configuration file that was read
    File(PathBuf),
    /// Defaults, along with the file that would change them
    Defaults(Option<PathBuf>),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keymap: Keymap::default(),
            source: Source::Defaults(None),
            problems: Vec::new(),
            files: Vec::new(),
        }
    }
}

/// Returns the files that may hold the configuration, most important
/// first: `config.toml` in `directory` alone, when one is given, or else
/// `cinq/config.toml` in each configuration directory.  The directory goes
/// by its absolute path, which the watcher reports changes by.  This is
/// the only place that reads the environment for it.
pub fn candidates(directory: Option<&Path>) -> Vec<PathBuf> {
    match directory {
        Some(directory) => {
            let directory = path::absolute(directory).unwrap_or_else(|_| directory.to_owned());
            vec![directory.join("config.toml")]
        }
        None => files::directories()
            .into_iter()
            .map(|directory| directory.join("config.toml"))
            .collect(),
    }
}

/// Reads the configuration from the first of `files` that exists
pub fn load(files: &[PathBuf]) -> Config {
    let files = files.to_vec();
    match files
        .iter()
        .find_map(|file| files::read(file).map(|text| (file.clone(), text)))
    {
        Some((file, Ok(text))) => {
            let (keymap, problems) = parse(&text);
            Config {
                keymap,
                source: Source::File(file),
                problems,
                files,
            }
        }
        Some((file, Err(problem))) => Config {
            source: Source::File(file),
            problems: vec![problem],
            files,
            ..Config::default()
        },
        None => Config {
            source: Source::Defaults(files.first().cloned()),
            files,
            ..Config::default()
        },
    }
}

/// Reads the text of a configuration file into key bindings, along with
/// what was wrong with it
pub fn parse(text: &str) -> (Keymap, Vec<String>) {
    let table = match text.parse::<Table>() {
        Ok(table) => table,
        Err(error) => {
            let line = error.span().map_or_else(String::new, |span| {
                format!(" on line {}", text[..span.start].matches('\n').count() + 1)
            });
            let problem = format!("is not TOML{line}: {}", error.message());
            return (Keymap::default(), vec![problem]);
        }
    };
    let unknown = table
        .keys()
        .filter(|name| !SETTINGS.contains(&name.as_str()))
        .map(|name| format!("no setting is named `{name}`"));
    let none = Table::new();
    let (keymaps, malformed) = match table.get("keymaps") {
        Some(Value::Table(keymaps)) => (keymaps, Vec::new()),
        Some(_) => (&none, vec!["keymaps should be a table".to_owned()]),
        None => (&none, Vec::new()),
    };
    let shadowed = keymaps
        .keys()
        .filter(|name| name.parse::<Preset>().is_ok())
        .map(|name| format!("keymaps.{name} goes unused, since {name} is a shipped keymap"));
    let (name, unnamed) = match table.get("keymap") {
        Some(Value::String(name)) => (name.as_str(), None),
        Some(_) => (
            Preset::default().name(),
            Some("keymap should be the name of a keymap"),
        ),
        None => (Preset::default().name(), None),
    };
    let (chosen, built) =
        build(name, keymaps, &[]).unwrap_or_else(|problem| (Keymap::default(), vec![problem]));
    let (keymap, unbound) = match table.get("keys") {
        Some(Value::Table(keys)) => bind(chosen, keys, ""),
        Some(_) => (chosen, vec!["keys should be a table".to_owned()]),
        None => (chosen, Vec::new()),
    };
    let conflicts = keymap.conflicts();
    let problems = unknown
        .chain(malformed)
        .chain(shadowed)
        .chain(unnamed.map(str::to_owned))
        .chain(built)
        .chain(unbound)
        .chain(conflicts)
        .collect();
    (keymap, problems)
}

/// Builds the keymap named `name`: a shipped one, or one of `keymaps`,
/// which starts from the keymap that its `from` names, or from no keys at
/// all without one, and binds the keys of its other entries.  `chain`
/// holds the keymaps that start from this one, so that a keymap that comes
/// back around to itself starts from no keys instead.
fn build(name: &str, keymaps: &Table, chain: &[&str]) -> Result<(Keymap, Vec<String>), String> {
    if let Ok(preset) = name.parse::<Preset>() {
        return Ok((preset.into(), Vec::new()));
    }
    let entries = match keymaps.get(name) {
        Some(Value::Table(entries)) => entries,
        Some(_) => return Err(format!("keymaps.{name} should be a table")),
        None => return Err(format!("no keymap is named `{name}`")),
    };
    if chain.contains(&name) {
        return Err(format!("keymap {name} starts from itself"));
    }
    let chain = [chain, &[name]].concat();
    let (start, problems) = match entries.get("from") {
        Some(Value::String(from)) => build(from, keymaps, &chain)
            .unwrap_or_else(|problem| (Keymap::empty(name), vec![problem])),
        Some(_) => (
            Keymap::empty(name),
            vec![format!(
                "keymaps.{name}.from should be the name of a keymap"
            )],
        ),
        None => (Keymap::empty(name), Vec::new()),
    };
    let bindings = entries.iter().filter(|(key, _)| key.as_str() != "from");
    let (keymap, unbound) = bind(start.named(name), bindings, &format!("keymaps.{name}: "));
    Ok((keymap, [problems, unbound].concat()))
}

/// Binds the keys of each entry of `entries` to its action in place of the
/// keys that `keymap` gives it, along with the problems of the entries,
/// each written after `place`
fn bind<'a>(
    keymap: Keymap,
    entries: impl IntoIterator<Item = (&'a String, &'a Value)>,
    place: &str,
) -> (Keymap, Vec<String>) {
    entries
        .into_iter()
        .fold((keymap, Vec::new()), |(keymap, problems), (name, value)| {
            let (entry, found) = entry(name, value);
            let keymap = match entry {
                Some((action, keys)) => keymap.with(action, keys),
                None => keymap,
            };
            let found = found.into_iter().map(|problem| format!("{place}{problem}"));
            (keymap, problems.into_iter().chain(found).collect())
        })
}

/// Reads the keys that one entry of the `[keys]` table or of a keymap binds
/// to its action, along with the problems that leave some of them out
fn entry(name: &str, value: &Value) -> (Option<Entry>, Vec<String>) {
    let Ok(action) = name.parse::<Action>() else {
        return (None, vec![format!("no action is named `{name}`")]);
    };
    let values: Vec<&Value> = match value {
        Value::Array(values) => values.iter().collect(),
        value => vec![value],
    };
    let (sequences, problems): (Vec<_>, Vec<_>) = values
        .into_iter()
        .map(|value| match value {
            Value::String(text) => press::sequence(text)
                .and_then(|keys| match keys.first().and_then(|key| key.digit()) {
                    Some(_) => Err(format!("`{text}` starts with a digit, which counts")),
                    None => Ok(keys),
                })
                .map_err(|problem| format!("{name}: {problem}")),
            _ => Err(format!("{name} should be keys or a list of keys")),
        })
        .partition_result();
    (Some((action, sequences)), problems)
}

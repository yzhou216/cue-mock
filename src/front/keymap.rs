// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Key bindings, which turn key presses into the messages that buttons send
//!
//! Every keymap in [`preset`] binds keys to every action.  The configuration
//! file in [`config`] chooses one and can replace the keys of any action,
//! and [`press`] names keys as the W3C UI Events specification does.
//! [`typing`] reads a command of several keys, such as a count followed by a
//! key, once it is complete, and the status bar shows the keys typed so far.
//! The keys bound to GO and Panic act at once, whatever was typed before
//! them and whatever modifiers are held, so that nothing holds them back.

mod action;
pub mod config;
mod preset;
mod press;
#[cfg(test)]
mod tests;
mod typing;

use std::collections::HashMap;

use itertools::Itertools;
use strum::VariantArray;

pub use action::Action;
pub use preset::Preset;
pub use press::{Press, written};

/// Keys bound to each action
#[derive(Debug, Clone)]
pub struct Keymap {
    /// Name of the keymap, a shipped one or one that the configuration
    /// file defines
    name: String,
    /// Key sequences bound to each action, which may be none
    bindings: HashMap<Action, Vec<Vec<Press>>>,
}

impl From<Preset> for Keymap {
    fn from(preset: Preset) -> Self {
        let bindings = Action::VARIANTS
            .iter()
            .map(|&action| {
                let keys = preset
                    .keys(action)
                    .iter()
                    .filter_map(|keys| press::sequence(keys).ok())
                    .collect();
                (action, keys)
            })
            .collect();
        Self {
            name: preset.name().to_owned(),
            bindings,
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Preset::default().into()
    }
}

impl Keymap {
    /// Returns a keymap named `name` that binds no keys at all
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            bindings: HashMap::new(),
        }
    }

    /// Returns the keymap under the name `name`, with the same keys
    pub fn named(self, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..self
        }
    }

    /// Returns the keymap with the keys of `action` replaced by `keys`
    pub fn with(self, action: Action, keys: Vec<Vec<Press>>) -> Self {
        let mut bindings = self.bindings;
        bindings.insert(action, keys);
        Self { bindings, ..self }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the key sequences bound to an action
    pub fn keys(&self, action: Action) -> &[Vec<Press>] {
        self.bindings.get(&action).map_or(&[], Vec::as_slice)
    }

    /// Writes the first key sequence bound to an action, which stands for
    /// the rest where there is room for one
    pub fn key(&self, action: Action) -> Option<String> {
        self.keys(action).first().map(|keys| written(keys))
    }

    /// Writes the key sequences bound to an action as a list, or `none`
    pub fn listed(&self, action: Action) -> String {
        let keys = self
            .keys(action)
            .iter()
            .map(|keys| written(keys))
            .join(", ");
        if keys.is_empty() {
            "none".to_owned()
        } else {
            keys
        }
    }

    /// Describes the bindings that hide others: a sequence bound to two
    /// actions, which only the first gets, a sequence that begins a longer
    /// one, which can then never be typed, and a key of
    /// [`Action::AT_ONCE`], which keeps every other sequence that holds it
    /// from being typed
    pub fn conflicts(&self) -> Vec<String> {
        let bound: Vec<Binding> = self.bound().collect();
        let hidden = bound.iter().enumerate().flat_map(|(index, &earlier)| {
            bound[index + 1..]
                .iter()
                .filter_map(move |&later| conflict(earlier, later))
        });
        let cut_short = bound.iter().filter_map(|&(_, keys)| self.cut_short(keys));
        hidden.chain(cut_short).collect()
    }

    /// Describes how a key of [`Action::AT_ONCE`] keeps `keys` from being
    /// typed, if one does, by acting halfway through them or when held with
    /// other modifiers than it is bound with.  Keys that begin with the
    /// very key bound are the binding of the key itself, or one that
    /// [`conflict`] describes.
    fn cut_short(&self, keys: &[Press]) -> Option<String> {
        let (&first, _) = keys.split_first()?;
        if self
            .at_once_bound(first)
            .is_some_and(|(_, key)| key == first)
        {
            return None;
        }
        keys.iter().find_map(|&press| {
            let (action, key) = self.at_once_bound(press)?;
            Some(format!(
                "`{}` acts at once as {}, so `{}` can never be typed",
                written(&[key]),
                action.name(),
                written(keys)
            ))
        })
    }

    /// Returns every binding, in the order of [`Action::VARIANTS`]
    fn bound(&self) -> impl Iterator<Item = Binding<'_>> {
        Action::VARIANTS.iter().flat_map(move |&action| {
            self.keys(action)
                .iter()
                .map(move |keys| (action, keys.as_slice()))
        })
    }
}

/// Action with one key sequence bound to it
type Binding<'a> = (Action, &'a [Press]);

/// Describes how a binding hides one that comes after it, if it does
fn conflict((earlier, keys): Binding, (later, other): Binding) -> Option<String> {
    if keys == other {
        let (earlier, later) = (earlier.name(), later.name());
        return Some(format!(
            "`{}` is bound to both {earlier} and {later}, so {later} goes without",
            written(keys)
        ));
    }
    let (short, long) = if keys.len() <= other.len() {
        (keys, other)
    } else {
        (other, keys)
    };
    long.starts_with(short).then(|| {
        format!(
            "`{}` comes first, so `{}` can never be typed",
            written(short),
            written(long)
        )
    })
}

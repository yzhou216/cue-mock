// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Times that the operator types

#[cfg(test)]
mod tests;

/// Time in seconds as the operator typed it, along with its value
///
/// Keeping the text lets the operator type intermediate input such as `1.`
/// without the field rewriting it.  The value is parsed once, when the text
/// is set, so playback and drawing read it without parsing again.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Seconds {
    text: String,
    value: f32,
}

impl Seconds {
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the parsed value, which is zero for empty or invalid text
    pub fn value(&self) -> f32 {
        self.value
    }
}

impl From<String> for Seconds {
    /// Parses the text, counting what is not a finite number of seconds,
    /// such as `inf` or `1e40`, as zero, so that no cue waits forever
    fn from(text: String) -> Self {
        let value = text
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .map_or(0.0, |value| value.max(0.0));
        Self { text, value }
    }
}

impl From<f32> for Seconds {
    fn from(value: f32) -> Self {
        Self::from(value.to_string())
    }
}

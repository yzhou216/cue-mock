// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Widgets that iced lacks
//!
//! [`button`](mod@button) acts on the press rather than the release,
//! [`dropdown`](mod@dropdown) lets keys move through its menu, and
//! [`keep_keys`](mod@keep_keys) keeps the keys typed into its text fields
//! from reaching the key bindings.  The first and the last handle events
//! around an iced widget with [`wrap`], while the drop-down draws itself in
//! the look of iced's pick list.

mod button;
mod dropdown;
mod keep_keys;
mod wrap;

pub use button::button;
pub use dropdown::dropdown;
pub use keep_keys::keep_keys;

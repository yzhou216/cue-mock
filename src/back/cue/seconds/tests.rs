// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how typed seconds read

use super::Seconds;

fn seconds(text: &str) -> Seconds {
    Seconds::from(text.to_owned())
}

#[test]
fn text_stays_as_typed() {
    assert_eq!(seconds("1.").text(), "1.");
    assert_eq!(seconds("1.").value(), 1.0);
}

#[test]
fn value_is_a_finite_number_of_seconds_or_zero() {
    assert_eq!(seconds(" 2.5 ").value(), 2.5);
    assert_eq!(seconds("-2").value(), 0.0);
    assert_eq!(seconds("1e40").value(), 0.0);
    assert_eq!(seconds("inf").value(), 0.0);
    assert_eq!(seconds("soon").value(), 0.0);
}

// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving the selection, and switching the cue list on screen

use crate::back::CueId;
use crate::front::App;

impl App {
    /// Selects a cue and shows the list that holds it
    pub(super) fn with_selection(self, id: CueId) -> Self {
        Self {
            list: self.workspace.list_of(id).unwrap_or(self.list),
            selected: Some(id),
            ..self
        }
    }

    /// Moves the selection by a number of cues, downwards when positive
    pub(super) fn stepped(self, offset: isize) -> Self {
        Self {
            selected: self.current().step(self.selected, offset),
            ..self
        }
    }

    /// Selects the cue on a row counted from one, or the last cue for a row
    /// past the end
    pub(super) fn at_row(self, row: usize) -> Self {
        let cues = &self.current().cues;
        let selected = cues
            .get(row.saturating_sub(1))
            .or(cues.last())
            .map(|cue| cue.id);
        Self { selected, ..self }
    }

    /// Shows a cue list with its standby cue selected, or its first cue
    /// when standby is past the end
    pub(super) fn switched_to(self, list: usize) -> Self {
        Self {
            selected: self
                .workspace
                .lists()
                .get(list)
                .and_then(|cues| cues.standby.or_else(|| cues.cues.first().map(|cue| cue.id))),
            list,
            ..self
        }
    }

    /// Shows the cue list a number of lists further down the sidebar, and
    /// stays put past either end
    pub(super) fn stepped_list(self, offset: isize) -> Self {
        let count = self.workspace.lists().len();
        let list = self
            .list
            .saturating_add_signed(offset)
            .min(count.saturating_sub(1));
        if list == self.list {
            self
        } else {
            self.switched_to(list)
        }
    }
}

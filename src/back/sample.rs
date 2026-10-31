// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sample show, loosely based on the opening of The Tempest
//!
//! Its sounds are short synthesized stand-ins in the `sounds` directory of
//! this crate, which `sounds/make.sh` generates.  The workspace finds them
//! in [`DATA`], wherever the program runs.  The voice-over file is missing
//! on purpose, so the show has a broken cue to warn about.

use super::{Advance, Cue, CueId, CueList, Kind, Workspace};

/// Directory that holds the `sounds` directory: the one a package installs
/// them in, which `CINQ_DATA_DIR` names at build time, or else this crate
const DATA: &str = match option_env!("CINQ_DATA_DIR") {
    Some(directory) => directory,
    None => env!("CARGO_MANIFEST_DIR"),
};

/// Cue to select at startup, chosen to differ from standby so the two are
/// visibly independent
pub const FIRST_SELECTED: CueId = CueId(3);

pub fn workspace() -> Workspace {
    let main = CueList::new(
        "Main",
        vec![
            Cue::new(CueId(1), "1", Kind::Fade, "Preshow music out")
                .target(CueId(20))
                .advance(Advance::Continue)
                .post_wait(2.0)
                .notes("GO when stage management calls house to half")
                .build(),
            Cue::new(CueId(2), "2", Kind::Audio, "Storm build")
                .file("sounds/storm.ogg")
                .level(-4.0)
                .advance(Advance::Continue)
                .post_wait(8.0)
                .build(),
            Cue::new(CueId(3), "2.1", Kind::Audio, "Thunder crack")
                .file("sounds/thunder.ogg")
                .pre_wait(0.5)
                .build(),
            Cue::new(CueId(4), "3", Kind::Audio, "Ship splits")
                .file("sounds/split.ogg")
                .advance(Advance::Follow)
                .notes("GO on \"We split, we split!\"")
                .build(),
            Cue::new(CueId(5), "3.1", Kind::Fade, "Storm out")
                .target(CueId(2))
                .duration(8.0)
                .build(),
            Cue::new(CueId(6), "4", Kind::Memo, "Scene 2: Prospero's cell").build(),
            Cue::new(CueId(7), "5", Kind::Audio, "Ariel's theme")
                .file("sounds/ariel.ogg")
                .level(-3.0)
                .build(),
            Cue::new(CueId(8), "6", Kind::Audio, "Ariel voice-over")
                .file("sounds/ariel_vo.ogg")
                .build(),
            Cue::new(CueId(9), "7", Kind::Fade, "Theme under speech")
                .target(CueId(7))
                .level(-18.0)
                .duration(3.0)
                .stop_target(false)
                .build(),
            Cue::new(CueId(10), "8", Kind::Stop, "Theme stop")
                .target(CueId(7))
                .build(),
            Cue::new(CueId(11), "9", Kind::Wait, "Hold for applause")
                .duration(3.0)
                .advance(Advance::Follow)
                .build(),
            Cue::new(CueId(12), "10", Kind::Audio, "Curtain call")
                .file("sounds/curtain.ogg")
                .build(),
        ],
    );
    let preshow = CueList::new(
        "Preshow",
        vec![
            Cue::new(CueId(20), "P1", Kind::Audio, "Preshow music")
                .file("sounds/preshow.ogg")
                .level(-6.0)
                .build(),
            Cue::new(CueId(21), "P2", Kind::Audio, "Five-minute chime")
                .file("sounds/chime.ogg")
                .notes("GO at five minutes to curtain")
                .build(),
        ],
    );
    Workspace::new(vec![main, preshow]).directory(DATA)
}

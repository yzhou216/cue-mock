// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound of one run of an audio cue

use std::path::Path;
use std::time::Duration;

use kira::sound::FromFileError;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::{AudioManager, Tween};

use crate::back::Active;

/// Time a sound takes to reach a level the workspace asks for.  It is a
/// little longer than a tick of the playback clock, so fades move smoothly
/// from one tick to the next.
const GLIDE: Duration = Duration::from_millis(50);

/// Sound of one run of an audio cue
pub(super) struct Voice {
    pub(super) run: u32,
    sound: StreamingSoundHandle<FromFileError>,
    /// Level and pause state last sent to the sound
    level: f32,
    paused: bool,
}

impl Voice {
    /// Starts the sound of a run at its level, as far into the file as the
    /// run has played, since the clock may have passed the start by part of
    /// a tick.  The sound streams from the file, so long files start as
    /// quickly as short ones.  Returns why the sound cannot start, as when
    /// its file has gone since it loaded or the output plays as many sounds
    /// as it can.
    pub(super) fn start(
        manager: &mut AudioManager,
        file: &Path,
        active: &Active,
    ) -> Result<Self, String> {
        let data = StreamingSoundData::from_file(file).map_err(|error| super::reason(&error))?;
        let data = data.volume(active.level).start_position(active.played());
        let sound = manager.play(data).map_err(|error| error.to_string())?;
        Ok(Self {
            run: active.run,
            sound,
            level: active.level,
            paused: false,
        })
    }

    /// Sends the sound any change in the level or pause state of its run
    pub(super) fn followed(mut self, active: &Active) -> Self {
        match (self.paused, active.paused) {
            (false, true) => self.sound.pause(Tween::default()),
            (true, false) => self.sound.resume(Tween::default()),
            _ => {}
        }
        if self.level != active.level {
            let glide = Tween {
                duration: GLIDE,
                ..Tween::default()
            };
            self.sound.set_volume(active.level, glide);
        }
        Self {
            level: active.level,
            paused: active.paused,
            ..self
        }
    }

    /// Stops the sound with kira's default ramp, which is short enough to
    /// sound immediate and long enough to avoid a click
    pub(super) fn stop(mut self) {
        self.sound.stop(Tween::default());
    }
}

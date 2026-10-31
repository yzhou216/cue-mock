// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound of one run of an audio cue

use std::path::Path;
use std::time::Duration;

use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::{FromFileError, Region};
use kira::{AudioManager, Tween};

use crate::back::{Active, Clip};

/// Time a sound takes to reach a level the workspace asks for.  It lasts
/// two ticks of the playback clock, so fades move smoothly from one tick to
/// the next.
const GLIDE: Duration = Duration::from_millis(50);

/// Sound of one run of an audio cue
pub(super) struct Voice {
    pub(super) run: u32,
    sound: StreamingSoundHandle<FromFileError>,
    /// Level last sent to the sound
    level: f32,
    /// Pause state last sent to the sound
    paused: bool,
}

impl Voice {
    /// Starts the sound of a run at its level, as far into its clip as the
    /// run has played, since the clock may have passed the start by part of
    /// a tick.  The sound ends at the end of the clip, or goes back to the
    /// start of the clip there when the clip loops.  It streams from the
    /// file, so long files start as quickly as short ones.  Returns why the
    /// sound cannot start, as when its file has gone since it loaded or the
    /// output plays as many sounds as it can.
    pub(super) fn start(
        manager: &mut AudioManager,
        file: &Path,
        active: &Active,
        clip: Clip,
    ) -> Result<Self, String> {
        let data = StreamingSoundData::from_file(file).map_err(|error| super::reason(&error))?;
        // Cut from the start of the file, so that times in the cut are
        // times in the file, which kira seeks to directly.  A cut is left
        // out where it would end past the last frame of the file.
        let cut = (clip.end < data.duration().as_secs_f64()).then(|| Region::from(0.0..clip.end));
        let looped = clip.looping.then(|| Region::from(clip.start..));
        let data = data
            .slice(cut)
            .loop_region(looped)
            .start_position(clip.position(active.played()))
            .volume(active.level);
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

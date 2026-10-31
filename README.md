<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# cinq (CINQ Is Not QLab)

Minimal cue-based show controller, the
[free](https://www.fsf.org/about/what-is-free-software) alternative to QLab

## Features

- Audio cues play a sound file (WAV, FLAC, Ogg Vorbis, or MP3) at a level in
  dB, which can change while the cue plays.
- Fade cues fade a playing audio cue to a level over time, and can stop it
  once the fade is over.
- Stop cues stop another cue.  Wait cues take time, and memo cues do nothing.
- GO fires the standby cue together with every cue that auto-continue or
  auto-follow chains to it, honouring pre-waits and post-waits.  Broken cues
  in the chain are skipped, and editing a running cue changes its next run.
- Pause all, Resume all, and Stop for each running cue.  Stopping a cue also
  cancels the cues of its chain that it has yet to start.  Panic fades
  everything out over one second, and a second Panic stops at once.
- Several cue lists, an inspector for every setting, a list of cues that
  cannot play, and a show mode that locks editing during a performance.
  Files that cannot be played are tried again every two seconds, so a file
  that comes back plays again.
- Undo and redo for every change to the show: adding and deleting cues,
  editing any of their settings, and moving standby.  The status bar names
  the latest change and the key that takes it back.  Playback cannot be
  undone, and show mode leaves only moves of standby to undo.
- Keyboard control, with a help page that lists the keys.

## Building

### Requirements

- Cargo
- pkg-config
- ALSA (GNU/Linux-only)

## Sample show

The sample show is loosely based on the opening of The Tempest.  Its sounds
in `sounds/` are synthesized stand-ins, and `sounds/make.sh` makes them
again with ffmpeg.  The voice-over file is missing on purpose, so one cue
shows as broken.

## Configuration

See [cinq(5)](man/cinq.5).

## License

cinq is free software under the GNU General Public License, version 3
or later.  Every file states its copyright and license as the
[REUSE specification](https://reuse.software/) sets out, and the text of
the license is in [`LICENSES`](LICENSES/GPL-3.0-or-later.txt).

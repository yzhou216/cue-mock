<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# cinq (CINQ Is Not QLab)

Minimal cue-based show controller, the
[free](https://www.fsf.org/about/what-is-free-software) alternative to QLab

## Features

- Audio cues play a sound file (WAV, FLAC, Ogg Vorbis, or MP3) at a level in
  dB, which can change while the cue plays.  They can play only the part of
  the file between a start and an end time, and loop it until something stops
  them.  A sound file chosen in the file chooser of the desktop names a new cue
  after itself.
- Fade cues fade a playing audio cue to a level over time, and can stop it
  once the fade is over.
- Stop cues stop another cue.  Wait cues take time, and memo cues do nothing.
- GO fires the standby cue together with every cue that auto-continue or
  auto-follow chains to it, honouring pre-waits and post-waits.  Broken cues
  in the chain are skipped, and editing a running cue changes its next run.
- Pause all, Resume all, and Stop for each running cue.  Stopping a cue also
  cancels the cues of its chain that it has yet to start.  Panic fades
  everything out over one second, and a second Panic stops at once.
- Several cue lists, set out in the show file, an inspector for every
  setting, a list of cues that cannot play, and a show mode that locks
  editing during a performance.
  Files that cannot be played are tried again every two seconds, so a file
  that comes back plays again.
- Undo and redo for every change to the show: adding, moving, and deleting
  cues, editing any of their settings, and moving standby.  The status bar
  names the latest change and the key that takes it back.  Playback cannot
  be undone, and show mode leaves only moves of standby to undo.
- Shows saved as readable TOML files, which `cinq SHOW` or the file chooser
  of the desktop opens.  Nothing ever asks before acting: changes that are
  not saved yet are kept as they happen, through closing the window,
  opening another show, or a crash, and come back the next time the show
  opens as one change that undo takes back.
- Keyboard control, with a help page that lists the keys.

## Building

### Requirements

#### General

- Cargo
- [scdoc](https://git.sr.ht/~sircmpwn/scdoc)

#### GNU/Linux and the BSDs

- pkg-config
- ALSA
- Wayland
- libxkbcommon
- D-Bus
- XDG desktop portal with a file chooser

### Desktop entry

On GNU/Linux and the BSDs, `share` holds a desktop entry, through which
application launchers list cinq, and the type of show files, through which
file managers open them in cinq.  Every build draws the icon of the entry
into `target/icons`.  The Nix package installs all three.  Elsewhere, copy
what `share` holds into `~/.local/share` and the icon into
`~/.local/share/icons/hicolor/scalable/apps`, then run
`update-mime-database ~/.local/share/mime`.

## Configuration

See [cinq(5)](man/cinq.5.scd).

## Show files

See [cinq-show(5)](man/cinq-show.5.scd).

## License

cinq is free software under the GNU General Public License, version 3
or later, whose text is in [`LICENSES`](LICENSES/GPL-3.0-or-later.txt).

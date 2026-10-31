#!/bin/sh

# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Synthesizes the stand-in sounds of the sample show with ffmpeg.  Every
# sound is made from sine waves and noise, so nothing recorded goes into
# the files.  The voice-over file stays missing on purpose.
set -eu
cd "$(dirname "$0")"

# Renders an aevalsrc expression or a filter graph to a stereo Ogg file at
# a moderate loudness.  Bitexact output leaves out the random stream serial
# number and the version of ffmpeg, so that the files change only when the
# sounds do.
render() {
	name=$1
	graph=$2
	ffmpeg -hide_banner -loglevel error -y -filter_complex "$graph,loudnorm=I=-18:TP=-2" \
		-ar 44100 -ac 2 -c:a libvorbis -q:a 4 -fflags +bitexact -flags:a +bitexact "$name"
}

# Expression for one plucked note per step, on the chord tones of four
# chords that change every eight steps.  The arguments are the step length
# in seconds, the four chord roots as MIDI notes, and the chord that is
# minor (or 9 for none).  Each note rises from and falls to silence, so the
# pitch changes without clicks, and a quiet pad swells under each chord.
arpeggio() {
	step=$1 r0=$2 r1=$3 r2=$4 r3=$5 minor=$6
	bar=$(awk "BEGIN { print $step * 8 }")
	cycle=$(awk "BEGIN { print $step * 32 }")
	printf '%s' "st(0, floor(mod(t, $cycle) / $bar));
st(1, floor(mod(t, $bar) / $step));
st(2, if(lt(ld(1), 4), ld(1), if(eq(mod(ld(1), 2), 0), 2, 1)));
st(3, if(eq(ld(0), 0), $r0, if(eq(ld(0), 1), $r1, if(eq(ld(0), 2), $r2, $r3))));
st(4, 440 * pow(2, (ld(3) + if(eq(ld(2), 0), 0, if(eq(ld(2), 1), if(eq(ld(0), $minor), 3, 4), if(eq(ld(2), 2), 7, 12))) - 69) / 12));
st(5, mod(t, $step));
st(6, min(1, ld(5) / 0.004) * exp(-6 * ld(5)) * (1 - ld(5) / $step));
st(7, 440 * pow(2, (ld(3) - 81) / 12));
0.5 * ld(6) * (0.6 * sin(2 * PI * ld(4) * t) + 0.25 * sin(4 * PI * ld(4) * t) + 0.1 * sin(6 * PI * ld(4) * t))
+ 0.12 * sin(PI * mod(t, $bar) / $bar) * (sin(2 * PI * ld(7) * t) + 0.5 * sin(3 * PI * ld(7) * t))"
}

# Preshow music: C, A minor, F, and G, in gentle eighth notes
render preshow.ogg "aevalsrc='$(arpeggio 0.25 60 57 53 55 1)':s=44100:d=75,aecho=0.8:0.6:250|500:0.35|0.2,afade=t=out:st=72:d=3"

# Curtain call: D, G, A, and D, in brighter sixteenth notes
render curtain.ogg "aevalsrc='$(arpeggio 0.125 62 67 69 62 9)':s=44100:d=30,aecho=0.8:0.5:180|360:0.3|0.15,afade=t=out:st=27:d=3"

# Five-minute chime: two bell strokes, E then C
render chime.ogg "aevalsrc='
st(0, t - 0.6);
if(gte(t, 0), min(1, t / 0.002) * exp(-2.5 * t) * (sin(2 * PI * 659.25 * t) + 0.5 * sin(2 * PI * 1819.5 * t) + 0.25 * sin(2 * PI * 3560 * t)), 0)
+ if(gte(ld(0), 0), min(1, ld(0) / 0.002) * exp(-2 * ld(0)) * (sin(2 * PI * 523.25 * ld(0)) + 0.5 * sin(2 * PI * 1444.2 * ld(0)) + 0.25 * sin(2 * PI * 2825.6 * ld(0))), 0)
':s=44100:d=4.5,aecho=0.8:0.7:300|600:0.3|0.2"

# Storm build: rain over gusting wind, swelling in from silence
render storm.ogg "anoisesrc=d=45:c=pink:a=0.5:seed=1,highpass=f=800,lowpass=f=6000[rl];
anoisesrc=d=45:c=pink:a=0.5:seed=2,highpass=f=800,lowpass=f=6000[rr];
[rl][rr]amerge=inputs=2[rain];
anoisesrc=d=45:c=brown:a=0.9:seed=3,lowpass=f=400,tremolo=f=0.2:d=0.7[wl];
anoisesrc=d=45:c=brown:a=0.9:seed=4,lowpass=f=400,tremolo=f=0.13:d=0.7[wr];
[wl][wr]amerge=inputs=2[wind];
[rain][wind]amix=inputs=2:weights=0.5 1:normalize=0,afade=t=in:d=12,afade=t=out:st=42:d=3"

# Thunder crack: a sharp crack, then rolling rumble
render thunder.ogg "anoisesrc=d=7:c=white:a=0.9:seed=5,highpass=f=300,afade=t=out:st=0:d=0.5:curve=exp[crack];
anoisesrc=d=7:c=brown:a=1:seed=6,lowpass=f=160,tremolo=f=5:d=0.4,afade=t=in:d=0.15,afade=t=out:st=0.3:d=6.7:curve=qsin[rumble];
[crack][rumble]amix=inputs=2:weights=0.6 1:normalize=0"

# Ship splits: three cracks of timber, then a falling groan
render split.ogg "aevalsrc='
(2 * random(0) - 1) * (exp(-25 * t) + if(gte(t, 0.35), 0.8 * exp(-25 * (t - 0.35)), 0) + if(gte(t, 0.62), exp(-18 * (t - 0.62)), 0))
+ if(gte(t, 0.5), min(1, (t - 0.5) / 0.3) * exp(-0.6 * (t - 0.5)) * (1 + 0.5 * sin(2 * PI * 7 * t)) * 0.5 * (sin(2 * PI * (90 * (t - 0.5) - 5.5 * (t - 0.5) * (t - 0.5))) + 0.4 * sin(4 * PI * (90 * (t - 0.5) - 5.5 * (t - 0.5) * (t - 0.5)))), 0)
':s=44100:d=5,highpass=f=40,aecho=0.8:0.5:120|240:0.3|0.2"

# Ariel's theme: a high pentatonic line with vibrato over a soft drone
render ariel.ogg "aevalsrc='
st(0, floor(t / 0.75));
st(1, mod(ld(0) * 3, 5));
st(2, 440 * pow(2, (81 + if(eq(ld(1), 0), 0, if(eq(ld(1), 1), 3, if(eq(ld(1), 2), 5, if(eq(ld(1), 3), 7, 10)))) - 69) / 12));
st(3, mod(t, 0.75) / 0.75);
0.4 * sin(PI * ld(3)) * sin(2 * PI * ld(2) * t)
+ 0.1 * (sin(2 * PI * 220 * t) + 0.6 * sin(2 * PI * 329.63 * t)) * min(1, t / 3)
':s=44100:d=36,vibrato=f=5:d=0.15,aecho=0.8:0.7:400|800:0.4|0.25,afade=t=out:st=33:d=3"

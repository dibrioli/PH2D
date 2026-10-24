#!/usr/bin/env bash
# A/B do Inspector estreito: $1 = binário, $2 = foto de saída, $3 = :display. Cena da câmera, Camera seleccionada,
# Game Camera aberta, dock arrastado de 300 para ~220 px.
set -uo pipefail
APP=$1; OUT=$2; DSP=$3
T=$(mktemp -d /tmp/ab.XXXX); mkdir -p "$T/home"
Xvfb $DSP -screen 0 1600x900x24 -nolisten tcp >/dev/null 2>&1 & XV=$!
AP=; fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }; trap fim EXIT
sleep 1; export DISPLAY=$DSP
(cd "$(dirname "$APP")/../.." && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_GAME_CAMERA_SMOKE=1 timeout 90 "$APP" >"$T/app.log" 2>&1) & AP=$!
for _ in $(seq 1 60); do W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 0.5; done
sleep 3; xdotool windowsize "$W" 1600 900; xdotool windowmove "$W" 0 0; sleep 8
xdotool mousemove 74 881; sleep 0.2; xdotool click 1; sleep 1.5
xdotool mousemove 1385 369; sleep 0.2; xdotool click 1; sleep 1.5
xdotool mousemove 1298 450; sleep 0.3; xdotool mousedown 1; sleep 0.3
for x in 1320 1350 1380 1400 $4; do xdotool mousemove $x 450; sleep 0.1; done
xdotool mouseup 1; sleep 2
import -window root "$OUT"

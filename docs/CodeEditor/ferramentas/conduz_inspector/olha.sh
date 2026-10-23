#!/usr/bin/env bash
# Abre a cena 9 numa Xvfb e fotografa (para achar onde estão as linhas). Uso: bash olha.sh <saida.png> [espera]
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
T=$(mktemp -d /tmp/olha9.XXXX); mkdir -p "$T/home"
DSP=:82
Xvfb $DSP -screen 0 1600x900x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_CODE_SMOKE=9 \
  timeout 120 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 60); do W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 0.5; done
sleep 3
xdotool windowsize "$W" 1600 900; xdotool windowmove "$W" 0 0
sleep "${2:-8}"
[ -n "${3:-}" ] && { xdotool mousemove ${3% *} ${3#* } click 1; sleep 1.5; }
import -window root "$1"
grep -E "code-smoke|panic" "$T/app.log" | head -5

#!/usr/bin/env bash
# Abre o binário de smoke da linha com UMA variável de cena numa Xvfb 1600x900, faz cliques reais e fotografa
# depois de cada um. Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/olha_cena.sh VAR=valor <prefixo> [x,y ...]
# Grava <prefixo>_0.png (ao abrir) e <prefixo>_N.png depois do N-ésimo clique.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
CENA=$1; PRE=$2; shift 2
T=$(mktemp -d /tmp/olha_cena.XXXX); mkdir -p "$T/home"
DSP=:85
Xvfb $DSP -screen 0 1600x900x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 "$CENA" \
  timeout 120 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 60); do W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 0.5; done
sleep 3
xdotool windowsize "$W" 1600 900; xdotool windowmove "$W" 0 0
sleep 8
import -window root "${PRE}_0.png"
n=0
for p in "$@"; do
  n=$((n + 1))
  xdotool mousemove "${p%,*}" "${p#*,}"; sleep 0.2; xdotool click 1; sleep 1.5
  import -window root "${PRE}_$n.png"
done

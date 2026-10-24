#!/usr/bin/env bash
# Roteiro CONDUZIDO do aviso «ph2d.object no topo fora de ph2d.property» (doc CodeEditor 10) sobre a cena 9:
# o binário de smoke com `PH2D_CODE_SMOKE=9` numa Xvfb 1600x900, teclas e cliques REAIS por `xdotool`.
# Uso (da raiz da worktree, com o binário de smoke feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/conduz_objeto_no_topo/corre.sh
# As fotos ficam ao lado (foto_NN_*.png).
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/conduz_objeto.XXXX); mkdir -p "$T/home"
DSP=:84
Xvfb $DSP -screen 0 1600x900x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_CODE_SMOKE=9 \
  timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 60); do W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 0.5; done
sleep 3
xdotool windowsize "$W" 1600 900; xdotool windowmove "$W" 0 0
sleep 6
xdotool windowfocus --sync "$W"
foto() { import -window root "$AQUI/$1.png" 2>/dev/null; }
clica() { xdotool mousemove "$1" "$2"; sleep 0.2; xdotool click 1; sleep 0.6; }
L7=145       # a linha vazia do topo
# ① na linha vazia do topo escrever a linha errada; parar de escrever
clica 640 $L7
xdotool type --delay 40 'local porta = ph2d.object("Door'
sleep 0.4
xdotool key End
sleep 2.5
foto foto_01_sublinhado
# ② abrir a lista de problemas
clica 652 645
sleep 0.8
foto foto_02_lista_de_problemas
# ③ desfazer: o aviso some na pausa seguinte
xdotool key ctrl+z; sleep 0.3; xdotool key ctrl+z; sleep 2.5
foto foto_03_desfeito

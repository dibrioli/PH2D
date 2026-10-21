#!/usr/bin/env bash
# Roteiro CONDUZIDO do item 8 do CodeEditor (arrastar para o código): o binário de smoke na cena 7
# (`PH2D_CODE_SMOKE=7`) numa Xvfb 1280x800 (a janela abre a 1024x768 em 0,0), arrastos REAIS por `xdotool`:
# ① Ctrl + arrastar a linha «Door» da Hierarquia para a linha vazia 10 do script; ② arrastar «Gold Coin» sem
# modificador; ③ Ctrl+Z. As fotos ficam ao lado (a meio do arrasto: o fantasma com o nome, o cursor fino e o
# balão das teclas). Uso (da raiz da worktree, com o binário de smoke feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/conduz_arrastar/corre.sh
# Medido 21/10: ① escreve `  local door = ph2d.object("Door")` na linha 10, a Hierarquia não muda (antes da cura
# do `hierarchy_drag_in_own_panel` soltar fora dela reparentava); ② escreve `ph2d.object("Gold Coin")` onde o
# rato larga; ③ desfaz a ② inteira.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/conduz_arrastar.XXXX); mkdir -p "$T/home"
DSP=:81
Xvfb $DSP -screen 0 1280x800x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_CODE_SMOKE=7 \
  timeout 240 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 60); do W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 0.5; done
sleep 6
xdotool windowfocus --sync "$W"
foto() { import -window root "$AQUI/$1.png" 2>/dev/null; }
arrasta() { # x0 y0 x1 y1 modificador|- foto_a_meio
  xdotool mousemove "$1" "$2"; sleep 0.3; xdotool mousedown 1; sleep 0.2
  for k in 1 2 3 4 5 6 7 8 9 10 11 12; do
    xdotool mousemove $(($1 + ($3 - $1) * k / 12)) $(($2 + ($4 - $2) * k / 12)); sleep 0.05
  done
  sleep 0.5
  [ "$5" != - ] && { xdotool keydown "$5"; sleep 0.4; }
  foto "$6"
  xdotool mouseup 1; sleep 0.3
  [ "$5" != - ] && xdotool keyup "$5"
  sleep 1
}
foto foto_00_cena7
arrasta 66 214 560 202 ctrl foto_01_a_meio_ctrl
foto foto_02_depois_ctrl
arrasta 80 237 560 278 - foto_03_meio_sem_ctrl
foto foto_03_soltar_sem_ctrl
xdotool key ctrl+z; sleep 1
foto foto_04_ctrl_z

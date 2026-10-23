#!/usr/bin/env bash
# Roteiro CONDUZIDO da cena 9 do CodeEditor (8b + 8c: arrastar do Inspector, Alt = propriedade de objecto):
# o binário de smoke com `PH2D_CODE_SMOKE=9` numa Xvfb 1600x900 (a janela é esticada a 1600x900 em 0,0),
# teclas e arrastos REAIS por `xdotool`. Uso (da raiz da worktree, com o binário de smoke feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/conduz_inspector/corre.sh [ate_o_passo]
# As fotos ficam ao lado (foto_NN_*.png); o texto final do script sai em saida.txt.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
ATE=${1:-99}
T=$(mktemp -d /tmp/conduz_inspector.XXXX); mkdir -p "$T/home"
DSP=:83
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
escreve() { xdotool type --delay 40 "$1"; sleep 0.6; }
arrasta() { # x0 y0 x1 y1 modificador|- foto_a_meio
  xdotool mousemove "$1" "$2"; sleep 0.3; xdotool mousedown 1; sleep 0.2
  for k in 1 2 3 4 5 6 7 8 9 10 11 12; do
    xdotool mousemove $(($1 + ($3 - $1) * k / 12)) $(($2 + ($4 - $2) * k / 12)); sleep 0.05
  done
  sleep 0.5
  [ "$5" != - ] && { xdotool keydown "$5"; sleep 0.4; }
  xdotool mousemove "$3" $(($4 + 1)); sleep 0.4
  foto "$6"
  xdotool mouseup 1; sleep 0.3
  [ "$5" != - ] && xdotool keyup "$5"
  sleep 1
}
L15=316      # a linha vazia do update (①/②)
L6=145       # a linha vazia do topo (③)
foto foto_00_cena9
# ① escrever o começo (o editor fecha o parêntese) e arrastar «Rotation» do cartão Transform para antes do «)»
clica 640 $L15
escreve "  ph2d.set(self.id, "
arrasta 1380 246 736 $L15 - foto_01_a_meio_rotation
foto foto_02_depois_rotation
[ "$ATE" -le 1 ] && exit 0
# ② escrever «, tempo * », abrir o cartão Script e arrastar «giro» para antes do «)»
escreve ", tempo * "
clica 1356 369
foto foto_03_cartao_script
arrasta 1393 466 892 $L15 - foto_04_a_meio_giro
foto foto_05_depois_giro
xdotool key ctrl+s; sleep 2
clica 268 10; sleep 2
foto foto_06_cena_a_nave_gira
clica 330 10; sleep 1
[ "$ATE" -le 2 ] && exit 0
# ③ Alt + arrastar «Door» da Hierarquia para a linha vazia do topo: o campo «door» aparece SEM Ctrl+S (1b)
arrasta 66 214 640 $L6 alt foto_07_a_meio_alt_door
foto foto_08_depois_alt
clica 1000 375
sleep 2.5
foto foto_09_campo_door_sem_salvar
xdotool key ctrl+s; sleep 2
[ "$ATE" -le 3 ] && exit 0
clica 268 10; sleep 1.5
foto foto_10_cena_a_porta_sobe
clica 330 10; sleep 1
# ④ largar a «Lamp» da Hierarquia no campo «door» do cartão Script
arrasta 68 237 1485 495 - foto_11_a_meio_lamp_no_campo
foto foto_12_campo_lamp
clica 268 10; sleep 1.5
foto foto_13_cena_a_lampada_sobe

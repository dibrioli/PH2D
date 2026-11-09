#!/bin/bash
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_rato_xvfb.sh [binario] [rotulo] [perf]
# (09/11) O rabisco PH2D_COMPOSITE_RABISCO entra pela ponte e NÃO passa pela entrada da shell; no kwin --virtual o
# xdotool não chega ao app. Aqui o rato real (XTEST) chega. Saída: target/mede-rato/xr_<rotulo>.log (+ .perf, .png).
# Xvfb puro + rato real (XTEST) sobre a tela Textured Paper no Wet; grava o PAINT_PERF e um perf plano da linha principal.
R="$(cd "$(dirname "$0")/../../.." && pwd)"
S=${SAIDA:-$R/target/mede-rato}; mkdir -p "$S"
BIN=${1:-$R/target/smoke/ph2d-host-desktop}; TAG=${2:-x}
D=:93
Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
sleep 1
cd "$R"
env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$S/home_x PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_PAINEL=wet \
  PH2D_COMPOSITE_PAPEL=1 PH2D_PAINT_PERF=1 PH2D_EXIT_AFTER_FRAMES=100000 "$BIN" > $S/xr_$TAG.log 2>&1 & APP=$!
sleep 12
WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
echo "[x] app $APP win $WIN"
DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 2>/dev/null; sleep 2
[ "$3" = perf ] && (perf record -F 499 -p $APP -o $S/xr_$TAG.perf -- sleep 20 > /dev/null 2>&1 &)
for t in 1 2 3; do
  cmd="mousemove 800 480 mousedown 1"
  for i in $(seq 0 500); do
    x=$(awk -v i=$i 'BEGIN{printf "%d", 800+260*sin(i*0.031*1.3)}'); y=$(awk -v i=$i 'BEGIN{printf "%d", 480+220*cos(i*0.031*0.9)}')
    cmd="$cmd mousemove $x $y sleep 0.004"
  done
  t0=$(date +%s%N); DISPLAY=$D xdotool $cmd mouseup 1; t1=$(date +%s%N)
  echo "[x] traco $t: $(( (t1-t0)/1000000 )) ms"
done
sleep 4
DISPLAY=$D import -window root $S/xr_$TAG.png 2>/dev/null
kill $APP; sleep 1; kill $XV

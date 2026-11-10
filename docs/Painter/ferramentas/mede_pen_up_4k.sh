#!/bin/bash
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh [binario] [rotulo] [WxH] [painel]
# (10/11) O dono: «trava no mouse-up numa imagem 4K, mesmo num traço pequeno». Xvfb + rato real (XTEST, como o
# mede_rato_xvfb.sh) sobre a tela Textured Paper de WxH (PH2D_COMPOSITE_TELA); 6 traços CURTOS, cada um seguido de
# 2 s parado. perf com relógio MONOTÓNICO (-k mono) e o instante de cada mouse-up no mesmo relógio: o resumo
# (mede_pen_up_4k.py) olha só o segundo depois de cada Up. Saída: target/mede-pen-up/<rotulo>.{log,perf,ups,png}.
R="$(cd "$(dirname "$0")/../../.." && pwd)"
S=${SAIDA:-$R/target/mede-pen-up}; mkdir -p "$S"
BIN=${1:-$R/target/smoke/ph2d-host-desktop}; TAG=${2:-x}; TELA=${3:-3840x2160}; PAINEL=${4:-wet}
D=:94
mono() { python3 -c 'import time;print(time.monotonic_ns())'; }
Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
sleep 1
cd "$R"
# O perf ABRE o app (o ptrace_scope do Yama recusa o `perf record -p` num processo que não é filho). Sem `-g`: com o
# perf_event_paranoid 2 a cadeia de chamadas não abre (medido 10/11) — a amostra é a função FOLHA.
env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$S/home_$TAG PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 \
  PH2D_COMPOSITE_PAINEL=$PAINEL PH2D_COMPOSITE_PAPEL=1 PH2D_COMPOSITE_TELA=$TELA PH2D_PAINT_PERF=1 \
  PH2D_QUADRO_LENTO_MS=${LENTO_MS:-40} PH2D_EXIT_AFTER_FRAMES=100000 perf record -k CLOCK_MONOTONIC -F 997 -o $S/$TAG.perf -- "$BIN" \
  > $S/$TAG.log 2>&1 & PERF=$!
sleep 15
APP=$(pgrep -P $PERF | head -1)
WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
echo "[pen-up] app $APP win $WIN tela $TELA painel $PAINEL"
DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 2>/dev/null; sleep 3
: > $S/$TAG.ups; : > $S/$TAG.ups_parede
for t in 1 2 3 4 5 6; do
  cmd="mousemove 760 460 mousedown 1"
  for i in $(seq 0 40); do cmd="$cmd mousemove $((760 + i * 2)) $((460 + (i % 7))) sleep 0.008"; done
  DISPLAY=$D xdotool $cmd
  mono >> $S/$TAG.ups; date +%s%N >> $S/$TAG.ups_parede
  DISPLAY=$D xdotool mouseup 1
  sleep 2.5
done
DISPLAY=$D import -window root $S/$TAG.png 2>/dev/null
kill -INT $APP; wait $PERF; kill $XV
echo "[pen-up] feito: $S/$TAG.{log,perf,ups,png}"

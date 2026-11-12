#!/usr/bin/env bash
# Fotos da cena `CENA` (omissão 14; plano 14 Q3/Q4): o binário de smoke numa Xvfb (o XTest chega; ver
# ../../conduz_posse/corre.sh), conduzido até ao passo k (`PH2D_CODE_SMOKE_PASSO`), e uma foto da
# janela. Com `RATO="x y"` o rato real (xdotool) pára ali antes da foto (a dica sobre um traço); com
# `TECLAS="ctrl+z"` teclas reais (xdotool key) chegam à app depois do rato e antes da foto. Com
# `ARRASTA="x1 y1 x2 y2"` um arrasto real (carregar em x1 y1, mover a x2 y2 em passos, largar) corre
# depois do rato e antes das teclas (o divisor da coluna da esquerda, 10/11). Com `DUPLO="x y"` um
# duplo-clique real (dois cliques a 80 ms, sem mover) corre depois do arrasto (o divisor volta à
# largura de fábrica). Com `CLIQUES="x1 y1;x2 y2;…"` cliques reais em sequência (mover, 1 s, clicar,
# 1 s), depois do `RATO` e antes do arrasto: abrir um menu e escolher num submenu (Settings ▸ Text
# rendering ▸ …, 10/11).
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   [CENA=13] bash docs/CodeEditor/ferramentas/smoke/cena14/fotos_q3.sh <passo> <saida.png> [espera_s]
set -uo pipefail
PASSO=${1:?passo}
SAIDA=${2:?saida.png}
ESPERA=${3:-9}
CENA=${CENA:-14}
RAIZ=$(git rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/fotos_q3.XXXX)
mkdir -p "$T/home"
AP= XV=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; [ -n "$XV" ] && kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
# Ecrã livre escolhido pelo Xvfb (corridas em paralelo e outras sessões não se cruzam).
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1930x1040x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_CODE_SMOKE="$CENA" PH2D_CODE_SMOKE_PASSO="$PASSO" timeout 120 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 2
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] && xdotool windowsize "$W" 1930 1040 windowmove "$W" 0 0 2>/dev/null
sleep "$ESPERA"
if [ -n "${RATO:-}" ]; then
  # shellcheck disable=SC2086
  xdotool mousemove $RATO
  sleep 1
  [ -n "${CLIQUE:-}" ] && { xdotool click 1; sleep 1; }
  # `RODA=n`: n voltas da roda para baixo onde o rato está (a grelha da cena 15 rola até ao fim).
  [ -n "${RODA:-}" ] && { xdotool click --repeat "$RODA" --delay 120 5; sleep 1; }
fi
if [ -n "${CLIQUES:-}" ]; then
  IFS=';' read -ra PONTOS <<<"$CLIQUES"
  for P in "${PONTOS[@]}"; do
    # shellcheck disable=SC2086
    xdotool mousemove $P; sleep 1
    xdotool click 1; sleep 1
  done
fi
if [ -n "${CAMINHO:-}" ]; then
  # `CAMINHO=<ficheiro>`: o rato sem botão por cada «x y» do ficheiro a 8 ms (a mão real, em passos de
  # 1–3 px com tremor), depois dos cliques: a diagonal até ao submenu (smoke do dono 11/11, `ax_diagonal`).
  while read -r PX PY; do xdotool mousemove "$PX" "$PY"; sleep 0.008; done < "$CAMINHO"
  sleep 0.5
fi
if [ -n "${ARRASTA:-}" ]; then
  read -r AX1 AY1 AX2 AY2 <<<"$ARRASTA"
  xdotool mousemove "$AX1" "$AY1"; sleep 0.5
  xdotool mousedown 1; sleep 0.3
  for i in 1 2 3 4; do
    xdotool mousemove $((AX1 + (AX2 - AX1) * i / 4)) $((AY1 + (AY2 - AY1) * i / 4)); sleep 0.2
  done
  xdotool mouseup 1; sleep 1
fi
if [ -n "${RATO_DEPOIS:-}" ]; then
  # `RATO_DEPOIS="x1 y1;x2 y2;…"`: o rato SEM botão por estes pontos depois do arrasto (o largar fora da
  # aba, 11/11: o 1.º movimento sem botão acaba o arrasto; antes o divisor seguia o rato).
  IFS=';' read -ra DEPOIS <<<"$RATO_DEPOIS"
  for P in "${DEPOIS[@]}"; do
    # shellcheck disable=SC2086
    xdotool mousemove $P; sleep 0.5
  done
fi
if [ -n "${DUPLO:-}" ]; then
  # shellcheck disable=SC2086
  xdotool mousemove $DUPLO; sleep 0.5
  xdotool click --repeat 2 --delay 80 1; sleep 1
fi
if [ -n "${RODA_DEPOIS:-}" ]; then
  # `RODA_DEPOIS="x y n"`: n voltas da roda para baixo em x y DEPOIS do arrasto (a coluna do lado do
  # quadro baixado pelo puxador, 10/11).
  read -r RX RY RN <<<"$RODA_DEPOIS"
  xdotool mousemove "$RX" "$RY"; sleep 0.5
  xdotool click --repeat "$RN" --delay 120 5; sleep 1
fi
if [ -n "${TECLAS:-}" ]; then
  # Teclas reais (XTest) na janela da app, depois do passo e antes da foto: na Xvfb chegam (medido 10/11,
  # `PH2D_UNDO_LOG=1 TECLAS=ctrl+z` ⇒ «RECEBIDA pela janela» no log). Foco por `windowfocus`: sem gestor de
  # janelas o `windowactivate` falha calado e a tecla não entra. Na sessão `kwin_wayland --virtual` NÃO chegam.
  # `LOG=<ficheiro>` guarda o log da app.
  [ -n "$W" ] && { xdotool windowfocus --sync "$W"; echo "foco: $(xdotool getwindowfocus) janela: $W"; }
  # shellcheck disable=SC2086
  xdotool key --delay 120 $TECLAS
  sleep 1
fi
[ -n "${LOG:-}" ] && cp "$T/app.log" "$LOG"
import -window root "$SAIDA"
echo "foto: $SAIDA"

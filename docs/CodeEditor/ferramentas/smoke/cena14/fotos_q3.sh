#!/usr/bin/env bash
# Fotos da cena `CENA` (omissão 14; plano 14 Q3/Q4): o binário de smoke numa Xvfb (o XTest chega; ver
# ../../conduz_posse/corre.sh), conduzido até ao passo k (`PH2D_CODE_SMOKE_PASSO`), e uma foto da
# janela. Com `RATO="x y"` o rato real (xdotool) pára ali antes da foto (a dica sobre um traço); com
# `TECLAS="ctrl+z"` teclas reais (xdotool key) chegam à app depois do rato e antes da foto.
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
DSP=:$((60 + CENA + PASSO))  # uma tela por cena e passo: corridas em paralelo não se cruzam
Xvfb "$DSP" -screen 0 1930x1040x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
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

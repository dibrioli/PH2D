#!/usr/bin/env bash
# Fotos de um GESTO REAL na aba Joint Editor (15/11): o binário de smoke numa Xvfb (lá o XTest chega, ao
# contrário do `kwin_wayland --virtual` do `fotografa_cena.sh`), a cena `PH2D_JOINT_EDITOR_SMOKE=$CENA` sobre
# a 123, e um roteiro de ações do `xdotool` com fotos A MEIO (botão ainda premido, número a ser escrito).
# Uso (da raiz da worktree, `target/smoke` feito):
#   CENA=1 bash docs/JointEditor/ferramentas/fotos_gesto.sh <pasta_saida> "<ações>" [espera_s]
# Ações separadas por `;`: `move X Y` · `down` · `up` · `type TEXTO` · `key TECLA` · `foto NOME` (NOME.png).
set -uo pipefail
OUT=${1:?pasta}
ACOES=${2:?ações}
ESPERA=${3:-9}
CENA=${CENA:-1}
RAIZ=$(git rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$OUT"
T=$(mktemp -d /tmp/fotos_gesto.XXXX)
mkdir -p "$T/home"
AP= XV=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; [ -n "$XV" ] && kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1930x1040x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY="$DSP" HOME="$T/home" XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}" \
  XDG_SESSION_TYPE=x11 PH2D_PHYSICS_SMOKE=123 PH2D_JOINT_EDITOR_SMOKE="$CENA" timeout 180 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 2
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] && xdotool windowsize "$W" 1930 1040 windowmove "$W" 0 0 2>/dev/null
sleep "$ESPERA"
[ -n "$W" ] && xdotool windowfocus --sync "$W"
IFS=';' read -ra PASSOS <<<"$ACOES"
for P in "${PASSOS[@]}"; do
  read -r CMD ARG <<<"$P"
  case "$CMD" in
    move) # shellcheck disable=SC2086
          xdotool mousemove $ARG; sleep 0.3 ;;
    down) xdotool mousedown 1; sleep 0.3 ;;
    up) xdotool mouseup 1; sleep 0.6 ;;
    type) xdotool type --delay 120 -- "$ARG"; sleep 0.6 ;;
    key) xdotool key "$ARG"; sleep 0.6 ;;
    foto) sleep 0.4; import -window root -crop 1930x1040+0+0 +repage "$OUT/$ARG.png" && echo "foto: $OUT/$ARG.png" ;;
    *) echo "ação desconhecida: $P" >&2; exit 2 ;;
  esac
done

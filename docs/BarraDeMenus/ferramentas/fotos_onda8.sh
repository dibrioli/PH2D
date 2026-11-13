#!/usr/bin/env bash
# Fotos da onda 8 (os menus como a maquete C): o binário de smoke numa Xvfb PRÓPRIA a 1930×1040, HOME de faz
# de conta, cena PH2D_GAME_SMOKE=2, UI de cartões, tema `TEMA` (omissão: o do app). Abre o «Menu», pousa em
# File e em Export (a cascata com o caminho aceso), Window ▸ Pixel, Help, Edit, View e a seta do Play;
# fotografa cada um e amplia 3× (vizinho mais próximo) o File ▸ Export.
# CARTOES=0: a UI de antes (PH2D_CARDS=0).
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/BarraDeMenus/ferramentas/fotos_onda8.sh [prefixo]
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
P=${1:-onda8}
SAIDA=${SAIDA:-$RAIZ/docs/BarraDeMenus/fotos}
JW=1930 JH=1040
APP="${APP_BIN:-$RAIZ/target/smoke/ph2d-host-desktop}"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$SAIDA"
T=$(mktemp -d /tmp/fotos_onda8.XXXX)
mkdir -p "$T/home/.ph2d"
[ -n "${TEMA_TOML:-}" ] && cp "$TEMA_TOML" "$T/home/.ph2d/"
AP= XV=
fim() {
  cp "$T/app.log" "$SAIDA/${P}_app.log" 2>/dev/null
  [ -n "$AP" ] && kill "$AP" 2>/dev/null
  [ -n "$XV" ] && kill "$XV" 2>/dev/null
  rm -rf "$T"
}
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre ${JW}x${JH}x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY -u PH2D_CARDS ${CARTOES:+PH2D_CARDS=$CARTOES} DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_GAME_SMOKE="${CENA:-2}" timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 3
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] || { echo "a janela não apareceu (ver $SAIDA/${P}_app.log)" >&2; exit 2; }
xdotool windowsize "$W" "$JW" "$JH" windowmove "$W" 0 0 2>/dev/null
sleep "${ESPERA:-12}"
xdotool windowfocus --sync "$W"
PX=965 PY=900
xdotool mousemove $PX $PY
# O rato anda aos passos (a mira da cascata lê o rasto), nunca salta.
vai() { local x=$1 y=$2 n=${3:-6} i; for i in $(seq 1 "$n"); do
  xdotool mousemove $((PX + (x - PX) * i / n)) $((PY + (y - PY) * i / n)); sleep 0.08; done; PX=$x PY=$y; sleep 0.6; }
clica() { vai "$1" "$2" 4; xdotool click 1; sleep 1; }
tecla() { xdotool key --delay 150 "$@"; sleep 0.8; }
foto() { import -window root -crop "$2" +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
amplia() { magick "$SAIDA/$1.png" -crop "$2" +repage -filter point -resize 300% "$SAIDA/$3.png"; echo "foto: $SAIDA/$3.png"; }
fecha() { tecla Escape Escape Escape; vai 965 900 3; }
# A posição de cada linha lê-se do índice de acerto que a cena despeja (PH2D_MENU_RECTS) — o fallback é a régua.
fecha
tecla alt; sleep 0.5
foto "${P}_1_raiz" 700x420+0+0
# A raiz: File é a 1.ª linha; anda por ela, depois ao Export (a cascata acende o caminho).
tecla Return; sleep 0.6
foto "${P}_2_file" 900x520+0+0
for _ in 1 2 3 4 5 6 7; do tecla Down; done
tecla Right; tecla Down; tecla Down; sleep 0.6
foto "${P}_3_file_export" 1100x520+0+0
amplia "${P}_3_file_export" 700x330+0+0 "${P}_3_file_export_x3"
fecha
for m in 2 3 4 5; do
  tecla alt; sleep 0.4
  for _ in $(seq 2 $m); do tecla Down; done
  tecla Right; sleep 0.6
  foto "${P}_4_raiz_${m}" 1000x600+0+0
  fecha
done
# Window ▸ Pixel (o submenu com o caminho aceso) e Edit ▸ Preferences.
tecla alt; sleep 0.4; for _ in 1 2 3; do tecla Down; done; tecla Right
for _ in 1 2 3 4 5 6 7 8 9; do tecla Down; done; tecla Right; sleep 0.6
foto "${P}_6_window_pixel" 1000x600+0+0
fecha
tecla alt; sleep 0.4; tecla Down; tecla Right; tecla Down; tecla Down; tecla Right; sleep 0.6
foto "${P}_7_edit_preferences" 1000x600+0+0
fecha
# A seta do Play.
clica 1722 18
foto "${P}_5_play_seta" 520x300+1410+0
fecha

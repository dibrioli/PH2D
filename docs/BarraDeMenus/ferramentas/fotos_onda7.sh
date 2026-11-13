#!/usr/bin/env bash
# Fotos da onda 7 (polimento da faixa, da fila fina e da busca): o binário de smoke numa Xvfb PRÓPRIA a
# 1930×1040 (o mesmo tamanho dos oráculos de `oraculos_onda7.sh`), HOME de faz de conta, cena
# PH2D_GAME_SMOKE=2, UI de cartões. Fotos: o topo em repouso, a busca «jump», e as ampliações 4× (vizinho
# mais próximo) do canto esquerdo, do campo e do grupo do Play — onde se vê o alinhamento ao pixel.
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/BarraDeMenus/ferramentas/fotos_onda7.sh [prefixo]
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
P=${1:-onda7}
SAIDA=${SAIDA:-$RAIZ/docs/BarraDeMenus/fotos}
JW=1930 JH=1040
APP="${APP_BIN:-$RAIZ/target/smoke/ph2d-host-desktop}"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$SAIDA"
T=$(mktemp -d /tmp/fotos_onda7.XXXX)
mkdir -p "$T/home/.ph2d"
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
(cd "$RAIZ" && env -u WAYLAND_DISPLAY -u PH2D_CARDS DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_GAME_SMOKE="${CENA:-2}" timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 3
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] || { echo "a janela não apareceu (ver $SAIDA/${P}_app.log)" >&2; exit 2; }
xdotool windowsize "$W" "$JW" "$JH" windowmove "$W" 0 0 2>/dev/null
sleep "${ESPERA:-12}"
xdotool windowfocus --sync "$W"
xdotool mousemove 965 900
tecla() { xdotool key --delay 150 "$@"; sleep 0.8; }
foto() { import -window root -crop "$2" +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
amplia() { magick "$SAIDA/$1.png" -crop "$2" +repage -filter point -resize 400% "$SAIDA/$3.png"; echo "foto: $SAIDA/$3.png"; }
tecla Escape Escape Escape
foto "${P}_topo" 1930x110+0+0
amplia "${P}_topo" 330x62+0+0 "${P}_x4_esquerda"
amplia "${P}_topo" 330x62+800+0 "${P}_x4_campo"
amplia "${P}_topo" 330x62+1600+0 "${P}_x4_direita"
tecla ctrl+k; xdotool type --delay 120 jump; sleep 1.2
foto "${P}_busca" 900x330+515+0
tecla Escape Escape Escape

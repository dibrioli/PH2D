#!/usr/bin/env bash
# Fotos da onda 6 da barra de menus («a busca muitíssimo mais bela», maquete C · 2 «Tudo numa
# busca»): o binário de smoke numa Xvfb PRÓPRIA (nunca o ecrã do dono), HOME de faz de conta, cena
# PH2D_GAME_SMOKE=2, `Ctrl+K` e «jump» por teclas reais (xdotool), `import -window root`. Molde:
# fotos_onda3.sh. Depois, lado a lado com o recorte da maquete (MAQUETE=<png 2x da versão C>).
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   bash docs/BarraDeMenus/ferramentas/fotos_onda6.sh [pasta_de_saida]
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/BarraDeMenus/fotos}
JW=1930 JH=1040
APP="${APP_BIN:-$RAIZ/target/smoke/ph2d-host-desktop}"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$SAIDA"
T=$(mktemp -d /tmp/fotos_onda6.XXXX)
mkdir -p "$T/home/.ph2d"
AP= XV=
fim() {
  cp "$T/app.log" "$SAIDA/onda6_app.log" 2>/dev/null
  [ -n "$AP" ] && kill "$AP" 2>/dev/null
  [ -n "$XV" ] && kill "$XV" 2>/dev/null
  rm -rf "$T"
}
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1930x1040x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY -u PH2D_CARDS DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_GAME_SMOKE="${CENA:-2}" timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 3
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] || { echo "a janela não apareceu (ver $SAIDA/onda6_app.log)" >&2; exit 2; }
xdotool windowsize "$W" "$JW" "$JH" windowmove "$W" 0 0 2>/dev/null
sleep "${ESPERA:-12}"
xdotool windowfocus --sync "$W"
xdotool mousemove 965 900
tecla() { xdotool key --delay 150 "$@"; sleep 0.8; }
busca() { tecla Escape Escape Escape; tecla ctrl+k; xdotool type --delay 120 "$1"; sleep 1.2; }
foto() { import -window root -crop "$2" +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
busca jump
foto onda6_busca "${CORTE:-1930x420+0+0}"
foto onda6_busca_perto "${PERTO:-760x300+585+10}"
busca play
foto onda6_busca_play "${PERTO:-760x300+585+10}"
tecla Down
foto onda6_busca_play_seta "${PERTO:-760x300+585+10}"
if [ -n "${MAQUETE:-}" ]; then
  convert "$MAQUETE" -crop "${MAQ_CORTE:-900x450+762+955}" +repage -resize x300 "$T/maq.png"
  convert "$SAIDA/onda6_busca_perto.png" -resize x300 "$T/app.png"
  convert "$T/maq.png" "$T/app.png" -background gray50 -splice 12x0 +append "$SAIDA/onda6_busca_lado_a_lado.png"
  echo "foto: $SAIDA/onda6_busca_lado_a_lado.png"
fi

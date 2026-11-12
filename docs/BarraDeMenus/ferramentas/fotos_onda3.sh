#!/usr/bin/env bash
# Fotos da onda 3 da barra de menus («a busca que responde»): o binário de smoke numa Xvfb PRÓPRIA
# (nunca o ecrã do dono), com HOME de faz de conta (a memória da faixa nasce do ficheiro semeado aqui
# e nunca toca a do dono), na cena PH2D_GAME_SMOKE=2 (Player e cenário), conduzido por teclas e rato
# reais (xdotool/XTest) e fotografado por `import -window root`. Molde: fotos_onda2.sh.
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   bash docs/BarraDeMenus/ferramentas/fotos_onda3.sh [pasta_de_saida]
# O log da app fica em <pasta>/onda3_app.log (procure `[hero] unhandled` e `panicked`).
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/BarraDeMenus/fotos}
JW=${JW:-1930}
JH=${JH:-1040}
CORTE=${CORTE:-700}
APP="${APP_BIN:-$RAIZ/target/smoke/ph2d-host-desktop}"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$SAIDA"

T=$(mktemp -d /tmp/fotos_onda3.XXXX)
mkdir -p "$T/home/.ph2d" "$T/home/Games/Forest"
# Dois projetos lembrados (o Open Recent); o primeiro existe.
: >"$T/home/Games/Forest/forest_level.ph2dproj"
cat >"$T/home/.ph2d/menu-memory.txt" <<EOF
# PH2D menu memory
project=$T/home/Games/Forest/forest_level.ph2dproj
project=$T/home/Games/cave_boss.ph2dproj
EOF
AP= XV=
fim() {
  cp "$T/app.log" "$SAIDA/onda3_app.log" 2>/dev/null
  cp "$T/home/.ph2d/menu-memory.txt" "$SAIDA/onda3_menu-memory.txt" 2>/dev/null
  [ -n "$AP" ] && kill "$AP" 2>/dev/null
  [ -n "$XV" ] && kill "$XV" 2>/dev/null
  rm -rf "$T"
}
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1930x1040x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY -u PH2D_CARDS DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_GAME_SMOKE="${CENA:-2}" timeout 600 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 3
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] || { echo "a janela não apareceu (ver $SAIDA/onda3_app.log)" >&2; exit 2; }
xdotool windowsize "$W" "$JW" "$JH" windowmove "$W" 0 0 2>/dev/null
sleep "${ESPERA:-12}"
xdotool windowfocus --sync "$W"
xdotool mousemove 965 600

foto() { # foto <nome> [largura] [altura]
  import -window root -crop "${2:-$JW}x${3:-$CORTE}+0+0" +repage "$SAIDA/$1.png"
  echo "foto: $SAIDA/$1.png"
}
tecla() { xdotool key --delay 150 "$@"; sleep 0.8; }
escreve() { xdotool type --delay 120 "$1"; sleep 1; }
fecha() { tecla Escape Escape Escape Escape; }
busca() { fecha; tecla ctrl+k; [ -n "${1:-}" ] && escreve "$1"; }

# 1 — a busca vazia: os menus primeiro
busca
foto onda3_1_busca_vazia "$JW" "$JH"

# 2 — uma ação do Input Map, com as teclas
escreve jump
foto onda3_2_busca_jump

# 2b — o CLIQUE na linha «jump» (não só o Enter): a aba abre na List, com jump destacado
xdotool mousemove 640 103; sleep 0.4; xdotool click 1; sleep 1.5
foto onda3_2b_clique_jump_abre_a_list "$JW" "$JH"

# 3 — um objeto da cena; Enter seleciona-o
busca player
foto onda3_3_busca_player
tecla Return
sleep 1
foto onda3_3b_player_selecionado "$JW" "$JH"

# 4 — os campos dos cartões do objeto selecionado; Enter dá o teclado ao campo e pisca-o
busca "air j"
foto onda3_4_busca_campo
tecla Return
sleep 0.2
foto onda3_4b_campo_aceso "$JW" "$JH"

# 5 — Alt+Enter: mostra onde o comando mora (File ▸ Export ▸ Export SVG)
busca "export svg"
foto onda3_5a_busca_export_svg
tecla alt+Return
sleep 2
foto onda3_5_onde_mora

# 6 — File ▸ Open Recent (pelo teclado: Alt, → abre o File, ↓↓, →)
fecha
tecla alt
tecla Right Down Down Right
foto onda3_6_open_recent

# 7 — três vezes Edit ▸ Undo PELO MENU, com o RATO (a mão do dono): o chip ensina o atalho
PX=965 PY=600
vai() { local x=$1 y=$2 n=${3:-8} i; for i in $(seq 1 "$n"); do xdotool mousemove $((PX + (x - PX) * i / n)) $((PY + (y - PY) * i / n)); sleep 0.15; done; PX=$x PY=$y; sleep 0.6; }
for _ in 1 2 3; do
  fecha
  vai 45 11 4; xdotool click 1; sleep 1      # «Menu»
  vai 60 67 6; vai 300 67 6                   # Edit ▸ Undo (só pairar)
  xdotool click 1; sleep 1.5
done
sleep 1
foto onda3_7_chip 1930 120

# 8 — Ctrl+Z: o chip some (o artista aprendeu)
tecla ctrl+z
foto onda3_8_chip_sumiu 1930 120

# 9 — os recentes da busca sobreviveram no ficheiro (copiado para a pasta das fotos)
echo "log: $SAIDA/onda3_app.log · memória: $SAIDA/onda3_menu-memory.txt"

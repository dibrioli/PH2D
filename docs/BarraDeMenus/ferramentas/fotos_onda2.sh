#!/usr/bin/env bash
# Fotos da faixa nova (onda 2 da barra de menus): o binário de smoke numa Xvfb PRÓPRIA (nunca o ecrã
# do dono), aberto UMA vez como o dono o abre (UI de cartões, sem cena de smoke), conduzido por rato e
# teclas reais (xdotool/XTest) e fotografado por `import -window root`, recortado ao topo.
# Método: docs/CodeEditor/ferramentas/smoke/cena14/fotos_q3.sh.
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   bash docs/BarraDeMenus/ferramentas/fotos_onda2.sh [pasta_de_saida]
#   SO_REPOUSO=1 bash …/fotos_onda2.sh   # só a foto 1 (para reler as coordenadas se a faixa mudar)
# O log da app fica em <pasta>/onda2_app.log (procure `[hero] unhandled` e `panicked`).
# As coordenadas abaixo leem-se na foto 1 (janela 1930×1040); mudou a faixa ⇒ reconfira-as.
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/BarraDeMenus/fotos}
JW=${JW:-1930}
JH=${JH:-1040}
CORTE=${CORTE:-700}
APP="${APP_BIN:-$RAIZ/target/smoke/ph2d-host-desktop}"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
mkdir -p "$SAIDA"

# --- coordenadas (px da janela 1930×1040) ---------------------------------------------------------
MENU="${MENU:-45 11}"            # botão «Menu»
FIND="${FIND:-965 12}"           # campo «Find anything»
SETA="${SETA:-1835 11}"          # seta ▾ ao lado do Play
# linhas do menu raiz e dos submenus: «x y» do centro de cada linha
R_FILE="${R_FILE:-60 45}"
R_EDIT="${R_EDIT:-60 67}"
R_WINDOW="${R_WINDOW:-60 111}"
F_EXPORT="${F_EXPORT:-300 211}"      # File ▸ Export
E_PREFS="${E_PREFS:-300 128}"        # Edit ▸ Preferences…
P_LINHA="${P_LINHA:-500 150}"        # Preferences… ▸ uma linha com submenu
W_PIXEL="${W_PIXEL:-300 343}"        # Window ▸ Pixel Art
# ---------------------------------------------------------------------------------------------------

T=$(mktemp -d /tmp/fotos_onda2.XXXX)
mkdir -p "$T/home"
AP= XV=
fim() {
  cp "$T/app.log" "$SAIDA/onda2_app.log" 2>/dev/null
  [ -n "$AP" ] && kill "$AP" 2>/dev/null
  [ -n "$XV" ] && kill "$XV" 2>/dev/null
  rm -rf "$T"
}
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1930x1040x24 "$T/xvfb.log" || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY -u PH2D_CARDS DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  timeout 600 "$APP" >"$T/app.log" 2>&1) &
AP=$!
sleep 3
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
[ -n "$W" ] || { echo "a janela não apareceu (ver $SAIDA/onda2_app.log)" >&2; exit 2; }
xdotool windowsize "$W" "$JW" "$JH" windowmove "$W" 0 0 2>/dev/null
sleep "${ESPERA:-10}"
xdotool windowfocus --sync "$W"

foto() { # foto <nome> [largura] [altura]
  import -window root -crop "${2:-$JW}x${3:-$CORTE}+0+0" +repage "$SAIDA/$1.png"
  echo "foto: $SAIDA/$1.png"
}
# O rato SEM botão de onde está até «x y», em passos pequenos (a mão real: pairar abre o submenu).
PX=0 PY=0
vai() {
  local x=$1 y=$2 n=${3:-8} i
  for i in $(seq 1 "$n"); do
    xdotool mousemove $((PX + (x - PX) * i / n)) $((PY + (y - PY) * i / n)); sleep 0.2
  done
  PX=$x PY=$y
  sleep 0.6
}
# Em L: primeiro na horizontal (dentro da linha do pai, para não acender a linha vizinha), depois na vertical.
vai_l() { vai "$1" "$PY" 6; vai "$1" "$2" 6; }
clica() { vai "$1" "$2" 4; xdotool click 1; sleep 1; }
tecla() { xdotool key --delay 150 "$@"; sleep 0.8; }
fecha() { tecla Escape Escape Escape Escape; vai 965 600 3; }

# 1 — repouso
vai 965 600 2
foto onda2_1_repouso
[ -n "${SO_REPOUSO:-}" ] && exit 0

# 2 — Menu
# shellcheck disable=SC2086
clica $MENU
foto onda2_2_menu

# 3 — Menu ▸ File ▸ Export (só pairar)
# shellcheck disable=SC2086
vai $R_FILE
# shellcheck disable=SC2086
vai_l $F_EXPORT
foto onda2_3_menu_file_export

# 4 — Menu ▸ Edit ▸ Preferences… ▸ uma linha (quatro níveis)
fecha
# shellcheck disable=SC2086
clica $MENU
# shellcheck disable=SC2086
vai $R_EDIT
# shellcheck disable=SC2086
vai_l $E_PREFS
# shellcheck disable=SC2086
vai_l $P_LINHA
foto onda2_4_menu_edit_preferences

# 5 — Menu ▸ Window ▸ Pixel
fecha
# shellcheck disable=SC2086
clica $MENU
# shellcheck disable=SC2086
vai $R_WINDOW
# shellcheck disable=SC2086
vai_l $W_PIXEL
foto onda2_5_menu_window_pixel

# 6 — Alt sozinho abre o Menu com «File» aceso; Down, Right abre Edit pelo teclado
fecha
tecla alt
foto onda2_6a_alt_so
tecla Down Right
foto onda2_6_alt

# 7 — a busca vazia
fecha
# shellcheck disable=SC2086
clica $FIND
foto onda2_7_find_vazio "$JW" "$JH"

# 8 — «exp»
xdotool type --delay 150 exp; sleep 1
foto onda2_8_find_exp

# 9 — um comando inofensivo pela busca, e ele volta em «Recent»
tecla BackSpace BackSpace BackSpace
xdotool type --delay 150 rulers; sleep 1
foto onda2_9a_find_rulers
tecla Return
foto onda2_9b_depois_do_enter
tecla ctrl+k
foto onda2_9_find_recente

# 10 — a seta ▾ do Play
fecha
# shellcheck disable=SC2086
clica $SETA
foto onda2_10_play_seta

# 11 — janela estreita (várias larguras: a que largura o campo vira lupa)
fecha
for L in ${LARGURAS:-1400 1200 1000 800 700 500}; do
  A=$([ "$L" = 700 ] && echo 600 || echo "$JH")
  xdotool windowsize "$W" "$L" "$A"; sleep 2
  vai $((L / 2)) 400 2
  if [ "$L" = 700 ]; then foto onda2_11_estreita 700 300; else foto "onda2_11_largura_$L" "$L" 120; fi
done
echo "log: $SAIDA/onda2_app.log"

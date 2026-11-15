#!/usr/bin/env bash
# Fotos dos pontos da curva Custom no gráfico da timeline (dono 15/11): vista de VALOR, a 1.ª chave de Position X
# seleccionada (trecho Linear). Um toque no meio da reta cria um ponto; arrastá-lo dobra a curva; Corner na coluna da
# esquerda; puxar a alça de saída (só ela anda); botão direito no ponto apaga-o. Nenhuma chave nasce (o Summary fica
# com as duas). Mesma Xvfb 1600×900 e conferências do fotos_retime_caixa.sh: cada passo tem de mudar a foto.
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Timeline/ferramentas/fotos_pontos_no_grafico.sh [pasta_de_saida]
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/Timeline/fotos}
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/fotos_retime.XXXX)
mkdir -p "$T/home/.ph2d" "$SAIDA"
AP= XV=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; [ -n "$XV" ] && kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1600x900x24 "$T/xvfb.log" 600 || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_ANIMACAO_SMOKE=1 timeout 500 "$APP" >"$T/app.log" 2>&1) &
AP=$!
timeout 60 bash -c 'until xdotool search --onlyvisible --name . >/dev/null 2>&1; do sleep 1; done' \
  || { echo "a janela não apareceu" >&2; exit 2; }
W=$(xdotool search --onlyvisible --name . | head -1)
xdotool windowsize "$W" 1600 900 windowmove "$W" 0 0
sleep 12
eval "$(xdotool getwindowgeometry --shell "$W")"
[ "$WIDTH" = 1600 ] && [ "$HEIGHT" = 900 ] || { echo "RECUSA: a janela tem ${WIDTH}x${HEIGHT}, os cliques são de 1600x900" >&2; exit 3; }
c() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool click "${3:-1}"; sleep 0.8; }
arrasta() { # x0 y0 x1 y1 — larga só se $5 != segura
  xdotool mousemove "$1" "$2"; sleep 0.3; xdotool mousedown 1
  for i in $(seq 1 12); do xdotool mousemove $(($1 + ($3 - $1) * i / 12)) $(($2 + ($4 - $2) * i / 12)); sleep 0.05; done
  sleep 0.5; [ "${5:-}" = segura ] || { xdotool mouseup 1; sleep 0.8; }
}
foto() { import -window root -crop 940x220+340+420 +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
# Píxeis da curva âmbar (ColorToken::Warn no tema do app, medido 15/11: srgb(212,199,158)) na banda.
ambar() { import -window root -crop 600x130+560+495 +repage "$T/b.png"
  magick "$T/b.png" -fuzz 4% -fill white -opaque 'srgb(212,199,158)' -fill black +opaque white -format '%[fx:round(mean*w*h)]' info:; }
confere() { [ "$1" ] || { echo "RECUSA: $2 — o layout mudou? as fotos não saem" >&2; exit 4; }; }
# `compare` sai 1 quando as fotos DIFEREM — com `pipefail` isso apagava o sim do awk.
difere() { { magick compare -metric AE "$SAIDA/$1.png" "$SAIDA/$2.png" null: 2>&1 || true; } | awk '{ exit !($1 > 200) }'; }
c 475 679; c 365 815; c 1211 687; c 981 684; xdotool key Escape; sleep 0.6; c 475 651
arrasta 800 666 800 330
# O roteiro de base liga «Show speed in the curves»; aqui desliga-se (a vista de VALOR).
c 1211 351; c 981 619; xdotool key Escape; sleep 0.6; c 567 479
foto pontos_0_reta_seleccionada
c 714 556
foto pontos_1_toque_cria_o_ponto
confere "$(difere pontos_0_reta_seleccionada pontos_1_toque_cria_o_ponto && echo s)" "o toque na reta não criou ponto"
arrasta 714 556 714 520
foto pontos_2_ponto_arrastado
confere "$(difere pontos_1_toque_cria_o_ponto pontos_2_ponto_arrastado && echo s)" "arrastar o ponto não mudou a curva"
c 390 557
foto pontos_3_corner
confere "$(difere pontos_2_ponto_arrastado pontos_3_corner && echo s)" "o Corner não mudou nada"
arrasta 762 503 795 565
foto pontos_4_alca_puxada_so_ela_anda
confere "$(difere pontos_3_corner pontos_4_alca_puxada_so_ela_anda && echo s)" "puxar a alça não mudou nada"
c 713 520 3
foto pontos_5_direito_apaga
confere "$(difere pontos_4_alca_puxada_so_ela_anda pontos_5_direito_apaga && echo s)" "o botão direito não apagou o ponto"

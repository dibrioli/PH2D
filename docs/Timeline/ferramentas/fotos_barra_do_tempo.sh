#!/usr/bin/env bash
# A barra de rolagem do tempo com alças de zoom (dono 16/11), fotografada no app com cliques reais:
# b0 a fita aberta (a barra no pé, polegar = o clipe todo) · b1 a ponta direita puxada = zoom (a esquerda fica)
# · b2 o meio arrastado = rolou · b3 clique na calha = saltou · b4 duplo clique = cabe tudo · b5 outro = volta.
# Mesma Xvfb 1600×900 dos outros roteiros; cada passo tem de mudar a foto (conferência).
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Timeline/ferramentas/fotos_barra_do_tempo.sh [pasta_de_saida]
# `PARA=<passo>` pára depois desse passo com uma foto da janela inteira (`tela_<passo>.png`) — para medir cliques.
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/Timeline/fotos}
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/fotos_barra.XXXX)
mkdir -p "$T/home/.ph2d" "$SAIDA"
AP= XV=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; [ -n "$XV" ] && kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1600x900x24 "$T/xvfb.log" 600 || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_ANIMACAO_SMOKE=1 timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
timeout 60 bash -c 'until xdotool search --onlyvisible --name . >/dev/null 2>&1; do sleep 1; done' \
  || { echo "a janela não apareceu" >&2; exit 2; }
W=$(xdotool search --onlyvisible --name . | head -1)
xdotool windowsize "$W" 1600 900 windowmove "$W" 0 0
sleep 12
eval "$(xdotool getwindowgeometry --shell "$W")"
[ "$WIDTH" = 1600 ] && [ "$HEIGHT" = 900 ] || { echo "RECUSA: a janela tem ${WIDTH}x${HEIGHT}, os cliques são de 1600x900" >&2; exit 3; }
c() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool click "${3:-1}"; sleep 0.8; }
duplo() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool click --repeat 2 --delay 80 1; sleep 0.8; }
arrasta() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool mousedown 1
  for i in $(seq 1 12); do xdotool mousemove $(($1 + ($3 - $1) * i / 12)) $(($2 + ($4 - $2) * i / 12)); sleep 0.05; done
  sleep 0.4; xdotool mouseup 1; sleep 0.8; }
tela() { import -window root +repage "$SAIDA/tela_$1.png"; echo "tela: $SAIDA/tela_$1.png"; }
para() { [ "${PARA:-}" = "$1" ] && { tela "$1"; exit 0; }; true; }
CROP=${CROP:-960x235+335+662}
foto() { import -window root -crop "$CROP" +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
confere() { [ "$1" ] || { echo "RECUSA: $2 — o layout mudou? as fotos não saem" >&2; exit 4; }; }
difere() { { magick compare -metric AE "$SAIDA/$1.png" "$SAIDA/$2.png" null: 2>&1 || true; } | awk '{ exit !($1 > 200) }'; }
para 0
# A barra no pé da fita, medida na tela_0: calha de x≈568 a x≈1268, y≈880; o clipe cabe, o polegar enche-a.
Y=880
# A cena toca em laço e a fita SEGUE o playhead: pausa (o botão verde dos controlos flutuantes) e volta ao 0.
c 447 651; c 391 651
foto barra_b0_fita_aberta
arrasta 1262 $Y 900 $Y
foto barra_b1_ponta_direita_zoom
confere "$(difere barra_b0_fita_aberta barra_b1_ponta_direita_zoom && echo s)" "puxar a ponta direita não deu zoom"
para 1
arrasta 734 $Y 834 $Y
foto barra_b2_meio_rolou
confere "$(difere barra_b1_ponta_direita_zoom barra_b2_meio_rolou && echo s)" "arrastar o meio não rolou"
para 2
c 1255 $Y
foto barra_b3_clique_na_calha_saltou
confere "$(difere barra_b2_meio_rolou barra_b3_clique_na_calha_saltou && echo s)" "o clique na calha não saltou"
para 3
duplo 1200 $Y
foto barra_b4_duplo_clique_cabe_tudo
confere "$(difere barra_b3_clique_na_calha_saltou barra_b4_duplo_clique_cabe_tudo && echo s)" "o duplo clique não enquadrou"
para 4
duplo 900 $Y
foto barra_b5_outro_duplo_clique_volta
confere "$(difere barra_b4_duplo_clique_cabe_tudo barra_b5_outro_duplo_clique_volta && echo s)" "o segundo duplo clique não voltou"
magick "$SAIDA"/barra_b{0,1,2,3,4,5}_*.png -append "$SAIDA/barra_sequencia.png"
echo "foto: $SAIDA/barra_sequencia.png"

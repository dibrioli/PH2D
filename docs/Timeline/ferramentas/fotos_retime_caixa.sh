#!/usr/bin/env bash
# Fotos do Retime = caixa do tempo (pesquisa 19, opção A, 15/11): binário de smoke numa Xvfb PRÓPRIA a 1600×900,
# HOME de faz de conta, PH2D_ANIMACAO_SMOKE=1, cliques reais (xdotool). Para, abre Position X, liga «Show speed in
# the curves», alarga a Timeline, põe Bounce Out na 1.ª chave, escolhe Retime e arrasta as duas alças para FORA da
# caixa (fotos a meio do arrasto e depois de soltar).
# ⚠️ Os cliques são em pixels fixos de uma janela 1600×900 (o app não despeja o índice de clique); por isso o
# roteiro CONFERE cada passo e PÁRA com erro em vez de entregar fotos erradas (auditoria 15/11): a janela tem de ter
# 1600×900; antes de Retime não há âmbar na banda (controlo), depois há; cada arrasto muda a foto.
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Timeline/ferramentas/fotos_retime_caixa.sh [pasta_de_saida]
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
foto() { import -window root -crop 940x260+340+400 +repage "$SAIDA/$1.png"; echo "foto: $SAIDA/$1.png"; }
# Píxeis da curva âmbar (ColorToken::Warn no tema do app, medido 15/11: srgb(212,199,158)) na banda.
ambar() { import -window root -crop 600x130+560+495 +repage "$T/b.png"
  magick "$T/b.png" -fuzz 4% -fill white -opaque 'srgb(212,199,158)' -fill black +opaque white -format '%[fx:round(mean*w*h)]' info:; }
confere() { [ "$1" ] || { echo "RECUSA: $2 — o layout mudou? as fotos não saem" >&2; exit 4; }; }
# `compare` sai 1 quando as fotos DIFEREM — com `pipefail` isso apagava o sim do awk.
difere() { { magick compare -metric AE "$SAIDA/$1.png" "$SAIDA/$2.png" null: 2>&1 || true; } | awk '{ exit !($1 > 200) }'; }
c 475 679; c 365 815; c 1211 687; c 981 684; xdotool key Escape; sleep 0.6; c 475 651
arrasta 800 666 800 330
c 567 479; c 567 479 3; c 607 560; c 855 640; c 877 785; c 869 837
confere "$( [ "$(ambar)" -lt 20 ] && echo s)" "há âmbar na banda ANTES do Retime (o controlo)"
c 503 548; c 567 479
confere "$( [ "$(ambar)" -gt 100 ] && echo s)" "a caixa âmbar não apareceu depois de clicar Retime"
foto retime_10_caixa_do_tempo
arrasta 665 576 640 455 segura
foto retime_11_alca_presa_na_borda_a_meio
confere "$(difere retime_10_caixa_do_tempo retime_11_alca_presa_na_borda_a_meio && echo s)" "o 1.º arrasto não mudou nada"
xdotool mouseup 1; sleep 0.8
arrasta 762 537 960 660 segura
foto retime_12_outra_alca_no_canto_a_meio
confere "$(difere retime_11_alca_presa_na_borda_a_meio retime_12_outra_alca_no_canto_a_meio && echo s)" "o 2.º arrasto não mudou nada"
xdotool mouseup 1; sleep 0.8
foto retime_13_depois_de_soltar

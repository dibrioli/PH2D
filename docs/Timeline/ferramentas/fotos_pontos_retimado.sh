#!/usr/bin/env bash
# Os três abertos do dono (15/11), fotografados no app: (1) pontos da Custom num trecho RETIMADO (Straight na 1.ª
# chave, Retime com as alças do tempo dentro da caixa, vista de valor): o toque na curva DESENHADA cria o ponto ali,
# arrastá-lo mantém o ritmo, o braço da ponta mexe sem apagar o ponto; (2) «Reset timing» no cartão da chave, só com
# ritmo — clicado, o ritmo sai e o ponto fica; (3) as miniaturas do cartão viradas quando o valor desce.
# Mesma Xvfb 1600×900 e conferências do fotos_retime_valor.sh (cada passo tem de mudar a foto).
# Uso (da raiz da worktree, binário `target/smoke` feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Timeline/ferramentas/fotos_pontos_retimado.sh [pasta_de_saida]
# `PARA=<passo>` pára depois desse passo com uma foto da janela inteira (`tela_<passo>.png`) — para medir cliques.
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
tela() { import -window root +repage "$SAIDA/tela_$1.png"; echo "tela: $SAIDA/tela_$1.png"; }
para() { [ "${PARA:-}" = "$1" ] && { tela "$1"; exit 0; }; true; }
# O estado do fotos_retime_valor.sh: Straight, Retime, alças do tempo dentro da caixa, vista de valor.
c 475 679; c 365 815; c 1211 687; c 981 684; xdotool key Escape; sleep 0.6; c 475 651
arrasta 800 666 800 330
c 567 479; c 567 479 3; c 675 560; c 869 837
c 503 548; c 567 479
confere "$( [ "$(ambar)" -gt 100 ] && echo s)" "a caixa âmbar não apareceu depois de clicar Retime"
arrasta 665 576 640 470
arrasta 762 537 820 600
c 1211 351; c 981 619; xdotool key Escape; sleep 0.6; c 567 479
foto pontos_r0_retimado_sem_pontos
para 0
# (1) O toque na curva desenhada cria o ponto ali; arrastá-lo; o braço da ponta. O ritmo fica (a curva continua a
# do ritmo, nada de reta).
c 710 553
foto pontos_r1_toque_cria_o_ponto
confere "$(difere pontos_r0_retimado_sem_pontos pontos_r1_toque_cria_o_ponto && echo s)" "o toque na curva retimada não criou ponto"
para 1
arrasta 710 553 710 528
foto pontos_r2_ponto_arrastado
confere "$(difere pontos_r1_toque_cria_o_ponto pontos_r2_ponto_arrastado && echo s)" "arrastar o ponto não mudou a curva"
para 2
# O ponto novo encurta o braço da ponta (de Casteljau): a ponta dele está perto da chave.
arrasta 583 590 583 555
foto pontos_r3_braco_da_ponta
confere "$(difere pontos_r2_ponto_arrastado pontos_r3_braco_da_ponta && echo s)" "puxar o braço da ponta não mudou nada"
para 3
# Smoke do dono 15/11: a ponta do braço puxada ACIMA da curva tem de continuar à vista (a banda cresce ao largar).
arrasta 583 555 590 478
foto pontos_r3b_braco_acima_da_curva_a_vista
confere "$(difere pontos_r3_braco_da_ponta pontos_r3b_braco_acima_da_curva_a_vista && echo s)" "puxar o braço para cima não mudou nada"
para 3b
# (2) O cartão da chave: «Reset timing» ao lado do Done.
c 567 479 3
import -window root -crop 560x420+560+470 +repage "$SAIDA/pontos_r4_cartao_com_reset_timing.png"
confere "$( [ "$(magick "$SAIDA/pontos_r4_cartao_com_reset_timing.png" -crop 100x30+170+352 +repage -format '%[fx:round(maxima*255)]' info:)" -gt 150 ] && echo s)" "o Reset timing não está ao lado do Done"
para 4
c 781 837; c 869 837
foto pontos_r5_reset_timing_tira_o_ritmo
confere "$(difere pontos_r3_braco_da_ponta pontos_r5_reset_timing_tira_o_ritmo && echo s)" "o Reset timing não mudou a curva"
para 5
# (3) A chave do segundo 1 (o trecho DESCE até à última): o cartão desenha as miniaturas viradas.
# Curve (Cubic In-out) acende as formas; o cartão arrasta-se pelo título para a esquerda, para a curva do trecho
# (que desce) ficar à vista ao lado das miniaturas.
c 859 479 3
c 899 560
arrasta 1000 502 640 502
import -window root -crop 940x420+340+470 +repage "$SAIDA/pontos_r6_miniaturas_viradas_quando_desce.png"
echo "foto: $SAIDA/pontos_r6_miniaturas_viradas_quando_desce.png"
para 6

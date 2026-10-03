#!/usr/bin/env bash
# Corre o oraculo dos AJUSTES sobre o composto (GIMP, sem interface) e grava a fixtura com cabecalho.
#   bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre_ajustes.sh [saida.bin]
set -euo pipefail
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VER="$(pacman -Q gimp | awk '{print $2}')"
LIC="$(pacman -Qi gimp | awk -F': ' '/^Licen/{print $2}')"
SAIDA="${1:-$AQUI/gimp_${VER%%-*}_ajustes.bin}"
CMD="gimp-console-3.2 -i --batch-interpreter=python-fu-eval -b \"exec(open('oraculo_ajustes.py').read())\" --quit"
export ORACULO_SAIDA="$SAIDA"
export ORACULO_CABECALHO="# FIXTURA DE ORACULO — um ajuste sobre o composto corre em que espaco? (ADR-0177, P3)
# alvo ......: GIMP $VER (pacman -Qi gimp => $LIC ⇒ so' se CORRE, caixa-preta; o fonte nao foi lido)
# porta .....: python-fu-eval sem interface — $CMD (em $AQUI)
# gerado ....: $(date -u +%Y-%m-%dT%H:%MZ)
# entrada ...: NOSSA (entradas.py, a grelha dos modos) — base + UMA camada, Normal 100, compostas em
#              PERCEPTUAL numa imagem FLOAT_NON_LINEAR; o ajuste corre sobre o VISIVEL (new_from_visible).
#              Invert = gimp-drawable-invert(linear); Curves/Levels = gimp:curves/gimp:levels com o 'trc'
#              da corrida; Posterize/Threshold = gimp:posterize/gimp:threshold (sem espaco no GIMP).
# saida .....: o visivel ajustado em float straight R'G'B'A, byte = floor(v*255+0,5). Pixel de alfa 0: a
#              cor e' indefinida (compare so' o alfa).
# formato ...: este cabecalho ate' '#FIM'; 'BASE\\n'+W*H*4; 'TOPO\\n'+W*H*4; por corrida
#              'RUN <ajuste> <perceptual|linear|-> <o que o GIMP aceitou>\\n'+W*H*4.
"
cd "$AQUI"
timeout 600 gimp-console-3.2 -i --batch-interpreter=python-fu-eval \
  -b "exec(open('oraculo_ajustes.py').read())" --quit 2>&1 | grep -E '^oraculo:|Error|Traceback|assert' || true
test -s "$SAIDA" && ls -l "$SAIDA"

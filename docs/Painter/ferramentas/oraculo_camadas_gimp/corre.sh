#!/usr/bin/env bash
# Corre o oraculo das camadas (GIMP, sem interface) e grava a fixtura com cabecalho.
#   bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre.sh [saida.bin]
set -euo pipefail
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VER="$(pacman -Q gimp | awk '{print $2}')"
LIC="$(pacman -Qi gimp | awk -F': ' '/^Licen/{print $2}')"
SAIDA="${1:-$AQUI/gimp_${VER%%-*}.bin}"
CMD="gimp-console-3.2 -i --batch-interpreter=python-fu-eval -b \"exec(open('oraculo.py').read())\" --quit"
export ORACULO_SAIDA="$SAIDA"
export ORACULO_CABECALHO="# FIXTURA DE ORACULO — as camadas juntam-se em que espaco? (doc Painter 45 §5)
# alvo ......: GIMP $VER (pacman -Qi gimp => $LIC ⇒ so' se CORRE, caixa-preta; o fonte nao foi lido)
# porta .....: python-fu-eval sem interface — $CMD (em $AQUI)
# gerado ....: $(date -u +%Y-%m-%dT%H:%MZ)
# entrada ...: NOSSA (oraculo.py: entradas()) — base + UMA camada; coluna = alfa da camada (rampa),
#              linha = alfa da base x cor da base x cor da camada. Imagem FLOAT_NON_LINEAR (sem degrau
#              intermedio); blend space = composite space = o espaco da corrida; composite mode UNION.
# saida .....: o visivel lido em float straight R'G'B'A, byte = floor(v*255+0,5). Pixel de alfa 0: a cor
#              e' indefinida (compare so' o alfa).
# formato ...: este cabecalho ate' '#FIM'; 'BASE\\n'+W*H*4; 'TOPO\\n'+W*H*4; por corrida
#              'RUN <nosso> <gimp> <perceptual|linear> <opacidade%> blend=<aceite> composite=<aceite>\\n'
#              +W*H*4 (o que o GIMP ACEITOU: o blend space fica 'auto' num modo sem funcao de mistura).
"
cd "$AQUI"
timeout 600 gimp-console-3.2 -i --batch-interpreter=python-fu-eval \
  -b "exec(open('oraculo.py').read())" --quit 2>&1 | grep -E '^oraculo:|recusou|Error|Traceback' || true
test -s "$SAIDA" && ls -l "$SAIDA"

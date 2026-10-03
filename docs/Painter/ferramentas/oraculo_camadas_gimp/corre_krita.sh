#!/usr/bin/env bash
# Corre a 2.a opiniao do oraculo das camadas (Krita, sem interface) e grava a fixtura com cabecalho.
#   bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre_krita.sh [saida.bin]
set -euo pipefail
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VER="$(pacman -Q krita | awk '{print $2}')"
LIC="$(pacman -Qi krita | awk -F': ' '/^Licen/{print $2}')"
SAIDA="${1:-$AQUI/krita_${VER%%-*}.bin}"
CMD="env -u DISPLAY -u WAYLAND_DISPLAY QT_QPA_PLATFORM=offscreen PYTHONPATH=. kritarunner -s oraculo_krita"
export ORACULO_SAIDA="$SAIDA"
export ORACULO_CABECALHO="# FIXTURA DE ORACULO — as camadas a 8 bits (2.a opiniao, doc Painter 45 §5 e §6 P2)
# alvo ......: Krita @VERSAO@ ($VER; pacman -Qi krita => $LIC ⇒ so' se CORRE, caixa-preta; o fonte nao foi lido)
# porta .....: kritarunner sem interface — $CMD (em $AQUI)
# gerado ....: $(date -u +%Y-%m-%dT%H:%MZ)
# entrada ...: NOSSA, a MESMA do GIMP (entradas.py). Documento RGBA U8 sRGB: o Krita compoe em codificado.
# saida .....: a projeccao lida em bytes (pixelData, BGRA reordenado a RGBA).
# formato ...: este cabecalho ate' '#FIM'; 'BASE\\n'+W*H*4; 'TOPO\\n'+W*H*4; por corrida
#              'RUN <nosso> <id do krita> krita-u8 <opacidade%>\\n'+W*H*4.
"
cd "$AQUI"
env -u DISPLAY -u WAYLAND_DISPLAY QT_QPA_PLATFORM=offscreen PYTHONPATH=. \
  timeout 600 kritarunner -s oraculo_krita >/dev/null 2>&1 || true
cat "$SAIDA.log" && rm -f "$SAIDA.log"
test -s "$SAIDA" && ls -l "$SAIDA"

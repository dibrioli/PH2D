#!/usr/bin/env bash
# Corre o oraculo dos efeitos de VIZINHANCA (P4) no GIMP e no Krita, sem interface, e grava as duas
# fixturas com cabecalho.   bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre_vizinhanca.sh
set -euo pipefail
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$AQUI"
COMUM="# entrada ...: NOSSA — entradas.py entrada_vizinhanca: 5 faixas de listras (preto|branco, vermelho|azul,
#              laranja|TRANSPARENTE de cor escondida azul, branco a 140|preto, e um impulso branco
#              sobre preto) numa UNICA camada; o filtro corre sobre ela.
# formato ...: este cabecalho ate' '#FIM'; 'ENTRADA\\n'+W*H*4 (straight RGBA u8); por corrida
#              'RUN <efeito> <filtro> <precisao> <o que o programa aceitou>\\n'+W*H*4.
"
GVER="$(pacman -Q gimp | awk '{print $2}')"
GLIC="$(pacman -Qi gimp | awk -F': ' '/^Licen/{print $2}')"
GCMD="gimp-console-3.2 -i --batch-interpreter=python-fu-eval -b \"exec(open('oraculo_vizinhanca.py').read())\" --quit"
ORACULO_SAIDA="$AQUI/gimp_${GVER%%-*}_vizinhanca.bin" ORACULO_CABECALHO="# FIXTURA DE ORACULO — um efeito de VIZINHANCA borra em que espaco? (ADR-0177, P4)
# alvo ......: GIMP $GVER (pacman -Qi gimp => $GLIC ⇒ so' se CORRE, caixa-preta; o fonte nao foi lido)
# porta .....: python-fu-eval sem interface — $GCMD (em $AQUI); Gimp.DrawableFilter
# gerado ....: $(date -u +%Y-%m-%dT%H:%MZ)
# saida .....: a camada filtrada lida em float straight R'G'B'A, byte = floor(v*255+0,5).
$COMUM" timeout 600 gimp-console-3.2 -i --batch-interpreter=python-fu-eval \
  -b "exec(open('oraculo_vizinhanca.py').read())" --quit 2>&1 | grep -E '^oraculo:|Error|Traceback|assert' || true
KVER="$(pacman -Q krita | awk '{print $2}')"
KLIC="$(pacman -Qi krita | awk -F': ' '/^Licen/{print $2}')"
KSAIDA="$AQUI/krita_${KVER%%-*}_vizinhanca.bin"
ORACULO_SAIDA="$KSAIDA" ORACULO_CABECALHO="# FIXTURA DE ORACULO — um efeito de VIZINHANCA a 8 bits (2.a opiniao, ADR-0177, P4)
# alvo ......: Krita @VERSAO@ ($KVER; pacman -Qi krita => $KLIC ⇒ so' se CORRE, caixa-preta; o fonte nao foi lido)
# porta .....: kritarunner sem interface — env -u DISPLAY -u WAYLAND_DISPLAY QT_QPA_PLATFORM=offscreen PYTHONPATH=. kritarunner -s oraculo_vizinhanca_krita (em $AQUI); Filter.apply
# gerado ....: $(date -u +%Y-%m-%dT%H:%MZ)
# saida .....: a camada filtrada em bytes (pixelData, BGRA reordenado a RGBA), documento RGBA U8 sRGB.
$COMUM" env -u DISPLAY -u WAYLAND_DISPLAY QT_QPA_PLATFORM=offscreen PYTHONPATH=. \
  timeout 600 kritarunner -s oraculo_vizinhanca_krita >/dev/null 2>&1 || true
cat "$KSAIDA.log" && rm -f "$KSAIDA.log"
ls -l "$AQUI"/*_vizinhanca.bin

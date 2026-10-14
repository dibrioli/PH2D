#!/bin/bash
# A bancada do prompt 35 no Rebelle: os gestos de pastel_eventos.py, fotografados a meio do gesto, no
# pen-up e 0,5 · 1 · 3 · 6 s depois (relógio de parede; o instante REAL de cada foto vai para tempos.tsv).
# Antes: gotas_inicia.sh (em segundo plano) e o Registration passado. ⛔ Nunca o ecrã do dono (foto.sh recusa :0).
# uso: pastel_tela.sh [args do pastel_eventos.py…]
O="$HOME/Apps/rebelle/oraculo"; S="$O/pastel"; F="$(cd "$(dirname "$0")" && pwd)"
[ "$(cat "$O/display.txt" 2>/dev/null)" = ":0" ] && { echo RECUSA; exit 3; }
rm -rf "$S"; mkdir -p "$S"
python3 -I "$F/pastel_eventos.py" "$S" "$@" || exit 1
cd "$O"
agora() { date +%s.%N; }
conta() { awk "BEGIN { printf \"%.3f\", $1 }"; }
foto() { timeout 60 bash foto.sh "pastel/$1" > /dev/null; echo -e "$1\t$(conta "$(agora) - $2")" >> "$S/tempos.tsv"; }
timeout 60 node "$F/ws_eventos.mjs" "$S/setup.json" 4000 2 > /dev/null
# A tela nova abre ENCAIXADA (141 %) e o Rebelle mede o tamanho do pincel no ECRÃ: o zoom vai a 100 % pelo
# campo do Navigator (coordenadas da janela de 1920×1080; `ZOOM_XY` troca-as) — a tela fica em (480, 220).
timeout 30 bash inj.sh "click ${ZOOM_XY:-1851 236}; key ctrl+a; type 100; key enter; wait 1500" > /dev/null
# O SET_BRUSH que TROCA de preset pinta o 1.º gesto com o tamanho velho (medido 14/10: Graphite 8B a 4 px
# depois do Oil Pastel saiu com 38 px no recto e 4 no zig-zague): um SET_BRUSH sozinho antes, e espera.
python3 -I -c "import json,sys;e=json.load(open(sys.argv[1]))['frames'][0]['events'][0];json.dump({'frames':[{'events':[e]}]},open(sys.argv[2],'w'))" "$S/recto_a.json" "$S/escova.json"
timeout 60 node "$F/ws_eventos.mjs" "$S/escova.json" 2000 2 > /dev/null
foto vazio "$(agora)"
for g in recto zigue; do
  timeout 60 node "$F/ws_eventos.mjs" "$S/${g}_a.json" 100 16 > /dev/null
  foto "${g}_meio" "$(agora)"
  timeout 60 node "$F/ws_eventos.mjs" "$S/${g}_b.json" 100 16 > /dev/null
  up=$(agora)
  # O Rebelle desenha o fim do traço com ATRASO (medido 14/10: a foto logo após o RELEASE ainda não tinha
  # os últimos 140 px a 70 px de pincel): o «pen-up» é a 1.ª foto igual à anterior; os tempos contam do RELEASE.
  foto "${g}_up" "$up"
  for k in $(seq 1 20); do
    cp "$S/${g}_up.png" "$S/.ant.png"; foto "${g}_up" "$up"
    python3 -I -c "import sys,numpy as n;from PIL import Image as I;a,b=(n.asarray(I.open(f).convert('RGB')) for f in sys.argv[1:]);sys.exit(0 if (a==b).all() else 1)" "$S/.ant.png" "$S/${g}_up.png" && break
  done
  for t in 0.5 1 3 6; do
    falta=$(conta "$up + $t - $(agora)")
    [ "${falta:0:1}" != "-" ] && sleep "$falta"
    foto "${g}_$t" "$up"
  done
done
echo "PASTEL_FIM"; cat "$S/tempos.tsv"

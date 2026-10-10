#!/bin/bash
# A bancada da W1e (mistura e papel), UMA tela: depois de gotas_inicia.sh e de passar o Registration e o New
# Artwork (as duas primeiras etapas do ~/Apps/rebelle/oraculo/arranque.sh).
# uso: mistura_papel_tela.sh <nome> <args do mistura_papel.py depois do nome…>
#   fotos (recortadas à tela, 1:1): m_<nome>_durante (ponteiro em baixo) · _a (~2 s depois do Up) ·
#   _b (+400 passos) · _c (+1200 passos)
O="$HOME/Apps/rebelle/oraculo"; S="$O/mistura"; F="$(cd "$(dirname "$0")" && pwd)"; mkdir -p "$S"
modo=$1; nome=$2; shift 2
python3 -I $F/mistura_papel.py "$modo" "$S" "$nome" "$@" || exit 1
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":400}]}]}' > $S/sim400.json
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":1200}]}]}' > $S/sim1200.json
cd $O
foto() { timeout 60 bash foto.sh "m_${nome}_$1" >/dev/null && magick "m_${nome}_$1.png" -crop 1000x640+480+220 +repage "$S/m_${nome}_$1.png" && echo "  foto $1"; }
timeout 60 node $F/ws_eventos.mjs $S/${nome}_0.json 2500 2 | tail -1
timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
timeout 90 node $F/ws_eventos.mjs $S/${nome}_1.json 600 2 | tail -1
foto durante
timeout 30 node $F/ws_eventos.mjs $S/${nome}_2.json 2000 2 | tail -1
foto a
timeout 30 node $F/ws_eventos.mjs $S/sim400.json 3000 2 >/dev/null
foto b
timeout 30 node $F/ws_eventos.mjs $S/sim1200.json 6000 2 >/dev/null
foto c

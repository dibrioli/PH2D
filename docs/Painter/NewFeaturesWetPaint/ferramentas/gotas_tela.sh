#!/bin/bash
# A bancada das GOTAS (doc 06), uma tela: depois de gotas_inicia.sh e de passar o Registration (arranque.sh, as duas primeiras etapas).
# uso: tela.sh <nome> [args do gotas.py…] — pinta uma tela, fotografa a ~2 s, depois de +400 e +1200 passos.
O="$HOME/Apps/rebelle/oraculo"; S="$O/gotas"; F="$(cd "$(dirname "$0")" && pwd)"; mkdir -p "$S"
nome=$1; shift
python3 -I $F/gotas.py "$nome" "$@" > $S/ev_$nome.json || exit 1
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":400}]}]}' > $S/sim400.json
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":1200}]}]}' > $S/sim1200.json
cd $O
timeout 60 node $F/ws_eventos.mjs $S/ev_$nome.json 1500 2 | grep -v '^<<' | tail -1
timeout 60 bash foto.sh g_${nome}_a | tail -1 | cut -c1-60
timeout 30 node $F/ws_eventos.mjs $S/sim400.json 3000 2 >/dev/null
timeout 60 bash foto.sh g_${nome}_b | tail -1 | cut -c1-60
timeout 30 node $F/ws_eventos.mjs $S/sim1200.json 6000 2 >/dev/null
timeout 60 bash foto.sh g_${nome}_c | tail -1 | cut -c1-60

#!/bin/bash
# A bancada das CAMADAS (prompt 15), UMA tela: depois de gotas_inicia.sh e de passar o Registration e o New Artwork
# (as duas primeiras etapas do ~/Apps/rebelle/oraculo/arranque.sh).
# uso: camadas_tela.sh <nome> molhado|seco [passadas]
#   fotos (recortadas à tela, 1:1) em ~/Apps/rebelle/oraculo/camadas/: c_<nome>_p00 (o azul seco) · _pKK (~1,5 s
#   depois do Up da passada K) · _fim (+400 passos) · _seco (DRY). Depois: camadas_regua.py.
O="$HOME/Apps/rebelle/oraculo"; S="$O/camadas"; F="$(cd "$(dirname "$0")" && pwd)"; mkdir -p "$S"
nome=$1; estado=$2; n=${3:-10}
python3 -I $F/camadas.py "$S" "$nome" --estado "$estado" --passadas "$n" || exit 1
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":400}]}]}' > $S/sim400.json
echo '{"frames":[{"events":[{"event_type":"DRY"}]}]}' > $S/dry.json
cd $O
foto() { timeout 60 bash foto.sh "c_${nome}_$1" >/dev/null && magick "c_${nome}_$1.png" -crop 1000x640+480+220 +repage "$S/c_${nome}_$1.png" && echo "  foto $1"; }
timeout 120 node $F/ws_eventos.mjs $S/${nome}_base.json 4000 2 | tail -1
timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
foto p00
for k in $(seq 1 "$n"); do
  kk=$(printf %02d "$k")
  timeout 60 node $F/ws_eventos.mjs $S/${nome}_p${k}.json 1500 2 | tail -1
  foto "p$kk"
done
timeout 30 node $F/ws_eventos.mjs $S/sim400.json 3000 2 >/dev/null
foto fim
timeout 30 node $F/ws_eventos.mjs $S/dry.json 2000 2 >/dev/null
foto seco

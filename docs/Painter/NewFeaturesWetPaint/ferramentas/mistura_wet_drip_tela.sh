#!/bin/bash
# A bancada da MISTURA com escorrimento (prompt 16), UMA tela: depois de gotas_inicia.sh e de passar o Registration e
# o New Artwork (as duas primeiras etapas do ~/Apps/rebelle/oraculo/arranque.sh).
# uso: mistura_wet_drip_tela.sh <nome> molhado|seco [passadas] [opacidade 0..100]
#   fotos (recortadas à tela, 1:1) em ~/Apps/rebelle/oraculo/mistura_wet_drip/: w_<nome>_p00 (o vermelho, tela já
#   inclinada) · _pKK (~1,5 s depois da passada K) · _a (+400 passos) · _b (+1200) · _seco (DRY).
O="$HOME/Apps/rebelle/oraculo"; S="$O/mistura_wet_drip"; F="$(cd "$(dirname "$0")" && pwd)"; mkdir -p "$S"
nome=$1; estado=$2; n=${3:-4}; op=${4:+--opacidade $4}
python3 -I $F/mistura_wet_drip.py "$S" "$nome" --estado "$estado" --passadas "$n" $op || exit 1
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":400}]}]}' > $S/sim400.json
echo '{"frames":[{"events":[{"event_type":"SIMULATION","repeats":1200}]}]}' > $S/sim1200.json
echo '{"frames":[{"events":[{"event_type":"DRY"}]}]}' > $S/dry.json
cd $O
foto() { timeout 60 bash foto.sh "w_${nome}_$1" >/dev/null && magick "w_${nome}_$1.png" -crop 1000x640+480+220 +repage "$S/w_${nome}_$1.png" && echo "  foto $1"; }
timeout 120 node $F/ws_eventos.mjs $S/${nome}_base.json 2500 2 | tail -1
timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1000" >/dev/null
foto p00
for k in $(seq 1 "$n"); do
  kk=$(printf %02d "$k")
  timeout 60 node $F/ws_eventos.mjs $S/${nome}_p${k}.json 1500 2 | tail -1
  foto "p$kk"
done
timeout 30 node $F/ws_eventos.mjs $S/sim400.json 3000 2 >/dev/null
foto a
timeout 30 node $F/ws_eventos.mjs $S/sim1200.json 6000 2 >/dev/null
foto b
timeout 30 node $F/ws_eventos.mjs $S/dry.json 2000 2 >/dev/null
foto seco

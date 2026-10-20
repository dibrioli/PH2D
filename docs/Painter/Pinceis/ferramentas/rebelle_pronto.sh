#!/bin/bash
# Depois de `gotas_inicia.sh`: espera a tela virtual, passa o Registration (Try → Pro), cancela o New Artwork
# e confere que a porta WebSocket responde. As coordenadas são as da `NewFeaturesWetPaint/ferramentas/arranque.sh`
# (tela 1920×1080). Saída 0 = pronto; 1 = não ficou pronto (feche com gotas_fecha.sh e tente de novo).
O="$HOME/Apps/rebelle/oraculo"; F="$(cd "$(dirname "$0")" && pwd)"
for _ in $(seq 1 60); do [ -s "$O/display.txt" ] && break; sleep 2; done
[ -s "$O/display.txt" ] || { echo "sem display.txt"; exit 1; }
cd "$O" || exit 1
n=0; until timeout 20 bash inj.sh "rect Registration" 2>/dev/null | grep -q janela; do
  sleep 3; n=$((n+1)); [ $n -gt 40 ] && { echo "sem Registration"; exit 1; }; done
timeout 30 bash inj.sh "click 1567 1049; wait 4000; click 1355 1049; wait 1000" >/dev/null
n=0; until timeout 20 bash inj.sh "rect New Artwork" 2>/dev/null | grep -q janela; do
  sleep 3; n=$((n+1)); [ $n -gt 40 ] && { echo "sem New Artwork"; exit 1; }; done
timeout 30 bash inj.sh "click 1407 790; wait 4000" >/dev/null
echo '{"frames":[{"events":[{"event_type":"BOOKMARK","id":"pronto","comment":"porta"}]}]}' > "$O/_pronto.json"
timeout 40 node "$F/ws_bateria.mjs" "$O/_pronto.json" 30000 2 | tail -1 | grep -q '^OK' || { echo "porta muda"; exit 1; }
echo PRONTO

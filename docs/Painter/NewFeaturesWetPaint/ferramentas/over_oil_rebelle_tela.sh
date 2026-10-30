#!/bin/bash
# A cena do Over Oil (J9) no Rebelle, UMA variante por chamada: depois de gotas_inicia.sh e do Registration/New Artwork.
# uso: over_oil_rebelle_tela.sh <saída> <nome> --oleo molhado|seco|nenhum [--papel r,g,b]
#   fotos (a cena 240×160, 1:1 com a sonda): <nome>_oleo (o óleo só) · _meio (o ponteiro em baixo) · _fim (+600 passos)
O="$HOME/Apps/rebelle/oraculo"; F="$(cd "$(dirname "$0")" && pwd)"; S="$1"; nome="$2"; shift 2; mkdir -p "$S"
python3 -I "$F/over_oil_rebelle.py" "$S" "$nome" "$@" || exit 1
cd "$O" || exit 1
foto() { timeout 60 bash foto.sh "oo_${nome}_$1" >/dev/null \
  && magick "oo_${nome}_$1.png" -crop 1000x640+480+220 +repage -crop 240x160+380+240 +repage "$S/${nome}_$1.png" \
  && rm -f "oo_${nome}_$1.png" && echo "  foto $1"; }
timeout 90 node "$F/ws_eventos.mjs" "$S/${nome}_0.json" 4000 2 | tail -1
timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
foto oleo
timeout 90 node "$F/ws_eventos.mjs" "$S/${nome}_1.json" 600 2 | tail -1
foto meio
timeout 90 node "$F/ws_eventos.mjs" "$S/${nome}_2.json" 8000 2 | tail -1
foto fim

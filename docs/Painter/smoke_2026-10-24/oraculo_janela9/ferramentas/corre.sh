#!/bin/bash
# uso: corre.sh <cena> <pasta saída>
set -u
O="$HOME/Apps/rebelle/oraculo"; F=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/Pinceis/ferramentas
J="$(cd "$(dirname "$0")" && pwd)"; C="$1"; S="$2"
D=$(cat "$O/display.txt"); [ "$D" = ":0" ] && { echo RECUSA; exit 3; }
janelas() { xprop -display "$D" -root _NET_CLIENT_LIST 2>/dev/null | grep -o '0x[0-9a-f]*'; }
principal() { for w in $(janelas); do xprop -display "$D" -id "$w" WM_NAME | grep -q 'Trial\|pixels\|RebelleArtwork' && { echo "$w"; return; }; done; }
cap() { local w full="$J/_full.png"; w=$(principal); [ -n "$w" ] || { echo "sem janela"; return 1; }
  import -display "$D" -window "$w" "$full" || return 3
  python3 -I - "$full" <<'PY' || return 4
import sys
from PIL import Image
a = Image.open(sys.argv[1]).convert("RGB")
fora, dentro = a.getpixel((479, 219)), a.getpixel((480, 220))
assert max(fora) < 140 and min(dentro) > 200, f"recorte desalinhado: fora={fora} dentro={dentro}"
PY
  magick "$full" -crop 1000x640+480+220 +repage -strip "$S/$1.png" && echo "foto $1 $(date +%T.%N | cut -c1-12)"; }
envia() { timeout 200 node "$F/ws_bateria.mjs" "$J/$1.json" 180000 2 | tail -1; }
envia "${C}_0"
timeout 30 bash "$O/inj.sh" "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1200" >/dev/null
t=$(date +%s.%N); envia "${C}_1"; echo "traços enviados em $(echo "$(date +%s.%N) - $t" | bc) s"
cap "${C}_t0"; sleep 1; cap "${C}_t1s"; sleep 4; cap "${C}_t5s"
envia "${C}_S"; sleep 1; cap "${C}_sim240"
envia "${C}_D"; sleep 1.5; cap "${C}_seco"

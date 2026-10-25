#!/bin/bash
# Lote de observacao no Aseprite Trial (ja aberto no DISPLAY :78, janela 1600x1000).
# Para cada imagem: abre, zoom 800%, foto de controlo; para cada paleta: carrega; para cada dither:
# converte (More Options > Indexed > dither > OK), fotografa, desfaz.
set -u
cd "$1"
export DISPLAY=:78
ITEM_Y=(104 162 220 278 336 394 452 510)   # No, Ord2, Ord4, Ord8, Old2, Old4, Old8, FS
open_img() {
  xdotool key ctrl+o; sleep 1.5
  xdotool type --delay 40 "$PWD/$1"; sleep 0.4; xdotool key Return; sleep 2.5
  xdotool mousemove 850 300; xdotool key 4; sleep 1
}
load_pal() {
  xdotool mousemove 177 134 click 1; sleep 0.8
  xdotool mousemove 234 442 click 1; sleep 1.5
  xdotool type --delay 40 "$PWD/$1"; sleep 0.4
  xdotool mousemove 1261 860 click 1; sleep 1.5
}
convert_to() {
  xdotool mousemove 185 74 click 1; sleep 0.6
  xdotool mousemove 222 124 click 1; sleep 0.6
  xdotool mousemove 462 198 click 1; sleep 1.3
  xdotool mousemove 732 406 click 1; sleep 0.8
  xdotool mousemove 1230 560 click 1; sleep 0.9
  xdotool mousemove 900 "$1" click 1; sleep 1.0
  import -window root "$2.dlg.png"
  xdotool mousemove 1057 692 click 1; sleep 2.0
  xdotool mousemove 850 300; sleep 0.3
  import -window root "$2"
  xdotool key ctrl+z; sleep 1.2
}
for img in cubo grade rampas; do
  open_img in_$img.png
  import -window root ctl_$img.png
  for pal in gb4 bw2 db32; do
    load_pal pal_$pal.gpl
    import -window root pal_${img}_$pal.png
    for k in 0 1 2 3 4 5 6 7; do
      convert_to "${ITEM_Y[$k]}" "idx_${img}_${pal}_$k.png"
      echo "$img $pal $k" >&2
    done
  done
done
echo FIM >&2

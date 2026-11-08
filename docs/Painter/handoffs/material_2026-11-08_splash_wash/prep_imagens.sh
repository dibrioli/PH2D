#!/bin/bash
set -e
magick -size 256x256 radial-gradient:white-black -level 30%,100% -evaluate pow 0.6 janela.png
i=0
for c in "560 140 180" "90 440 240" "720 230 240"; do
  set -- $c
  g=$(python3 -I -c "s=2.553;x,y,w=$1,$2,$3;print(f'{int(w*s)}x{int(w*s)}+{int(x*s)}+{int(y*s)}')")
  magick w11_azul.png -crop $g +repage -resize 256x256! -level 38%,72% janela.png -compose multiply -composite -level 0,90% sp$i.png
  i=$((i+1))
done
montage sp0.png sp1.png sp2.png -tile 3x1 -geometry +4+4 -background gray50 sp_m.png
# grão da aguada (w7 = watercolor_7_0.jpg):
# magick w7.jpg -gravity center -crop 1600x1600+0+0 +repage -colorspace gray -negate -auto-level -resize 256x256 -level 10%,95% g0.png
# magick g0.png \( g0.png -flop \) +append \( +clone -flip \) -append +repage -depth 8 -define png:color-type=0 wash_mottle.png
# depois, cada sp<i>.png aparado ao conteúdo e re-enquadrado (a mancha enche o dab):
# g=$(magick sp.png -threshold 4% -trim -format "%wx%h%O" info:); magick sp.png -crop $g +repage -gravity center -background black \
#   -extent "%[fx:max(w,h)*1.08]x%[fx:max(w,h)*1.08]" -resize 256x256! -colorspace gray -depth 8 -define png:color-type=0 splash_<i>.png

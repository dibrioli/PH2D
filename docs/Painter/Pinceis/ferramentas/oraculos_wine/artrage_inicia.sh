#!/bin/bash
# Abre a tela :71 (Xvfb, só software) e o ArtRage 6 Demo; fecha o «Buy ArtRage» do arranque e MAXIMIZA a janela
# (1920x1080: a pintura de 1000x640 fica em 460,235 sem painéis por cima). Idempotente.
X="$HOME/Apps/pincel-oraculos/x.sh"
bash "$HOME/Apps/pincel-oraculos/tela.sh" artrage >/dev/null
if ! bash "$X" artrage search --name '^ArtRage' >/dev/null 2>&1; then
  cd "$HOME/Apps/artrage" || exit 1
  setsid nohup bash artrage.sh > run.log 2>&1 < /dev/null &
  disown
  for i in $(seq 90); do sleep 1; bash "$X" artrage search --name 'Buy ArtRage' >/dev/null 2>&1 && break; done
  sleep 3
fi
geo=$(bash "$X" artrage search --name '^ArtRage' getwindowgeometry %@ 2>/dev/null | grep Geometry | head -1)
case "$geo" in
  *1920x1080*) bash "$X" artrage mousemove 1232 845 click 1 sleep 1 mousemove 1900 540 ;;
  *) bash "$X" artrage mousemove 1232 845 click 1 sleep 1 mousemove 1553 43 click 1 sleep 2 mousemove 1900 540 ;;
esac
bash "$X" artrage search --name '^ArtRage' getwindowgeometry %@ | grep Geometry
echo "ArtRage pronto"

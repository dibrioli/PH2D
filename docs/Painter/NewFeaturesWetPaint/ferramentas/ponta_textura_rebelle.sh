#!/bin/bash
# Pontas TEXTURADAS no Rebelle (13/10, plano 27 W0) — UMA sessão, um lote: 5 presets × 4 traços + a varredura
# da opacidade. Antes: gotas_inicia.sh (tela virtual + Rebelle com a porta WebSocket) em segundo plano e os
# diálogos Registration (Try → Pro) e New Artwork passados (as duas primeiras etapas do arranque.sh). Depois:
# gotas_fecha.sh (fecha SÓ esta tela virtual, por PID conferido).
# Por bloco (uma tela): parte 0 = NEW_ARTWORK 1000×640 + SET_BRUSH de aquecimento, 2,5 s, zoom 100 % (campo do
# Navigator, por clique), os traços; cada parte seguinte = aquecimento, 2,5 s, os traços. Foto a (~2 s depois
# do último Up); nos blocos com simulação, SIMULATION 400 + DRY e foto b. Recorta a tela (1000×640 em 480,220),
# monta as fixtures e mede-as.
O="$HOME/Apps/rebelle/oraculo"; S="$O/ponta_textura"; F="$(cd "$(dirname "$0")" && pwd)"
FX="$F/../fixtures/rebelle"; mkdir -p "$S"
python3 -I "$F/ponta_textura_rebelle.py" eventos "$S" || exit 1
cd "$O"
envia() { timeout 90 node "$F/ws_eventos.mjs" "$1" "$2" 2 | grep -v "^ligado\|^enviados\|^fechado"; }
foto() { timeout 60 bash foto.sh "pt_$1" >/dev/null && magick "pt_$1.png" -crop 1000x640+480+220 +repage "$S/ponta_$1.png" && echo "  foto $1"; }
while read -r bloco partes com_sim; do
  echo "== $bloco"
  for ((k = 0; k < partes; k++)); do
    envia "$S/${bloco}_${k}_0.json" 2500
    [ $k -eq 0 ] && timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
    envia "$S/${bloco}_${k}_1.json" 2000
  done
  sleep 1
  if [ "$com_sim" = 1 ]; then
    foto "${bloco}_a"; envia "$S/sim.json" 6000; foto "${bloco}_b"
  else
    foto "$bloco"
  fi
done < "$S/blocos.txt"
cp "$S/ponta_op_granulation.png" "$FX/ponta_textura_opacidade_granulation_40.png"
cp "$S/ponta_op_tres.png" "$FX/ponta_textura_opacidade_gouachedry_fan_round_40.png"
python3 -I "$F/ponta_textura_rebelle.py" monta "$S" "$FX"
python3 -I "$F/ponta_textura_rebelle.py" cabecalhos "$S" "$FX"
python3 -I "$F/ponta_textura_rebelle.py" regua "$FX" | tee "$S/medidas.txt"
echo PONTA_TEXTURA_FIM

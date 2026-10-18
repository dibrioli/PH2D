#!/bin/bash
# A varredura do TILT do lápis no Rebelle (18/10) — UMA sessão, três telas (A ao longo · B através · C rampas).
# Antes: gotas_inicia.sh (tela virtual + Rebelle com a porta WebSocket) em segundo plano e os diálogos
# Registration (Try → Pro) e New Artwork passados (as duas primeiras etapas do arranque.sh). Depois:
# gotas_fecha.sh (fecha SÓ esta tela virtual, por PID conferido).
# Por tela: NEW_ARTWORK 1000×640 + SET_BRUSH de aquecimento, 2,5 s, zoom 100 % (campo do Navigator, por clique),
# os traços; foto ~2 s depois do último Up (o lápis é seco: sem simulação), recorte 1000×640 em 480,220, a régua.
O="$HOME/Apps/rebelle/oraculo"; S="$O/tilt_lapis"; F="$(cd "$(dirname "$0")" && pwd)"
FX="$F/../fixtures/rebelle_tilt"; mkdir -p "$S" "$FX"
python3 -I "$F/tilt_lapis_rebelle.py" eventos "$S" || exit 1
cd "$O"
envia() { timeout 90 node "$F/ws_eventos.mjs" "$1" "$2" "${3:-2}" | grep -v "^ligado\|^enviados\|^fechado"; }
while read -r bloco partes _ intervalo; do
  [ -n "$TILT_BLOCOS" ] && [[ " $TILT_BLOCOS " != *" $bloco "* ]] && continue
  echo "== $bloco"
  envia "$S/${bloco}_0_0.json" 2500
  timeout 30 bash inj.sh "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
  envia "$S/${bloco}_0_1.json" 2000 "$intervalo"
  timeout 60 bash foto.sh "tl_$bloco" >/dev/null && magick "tl_$bloco.png" -crop 1000x640+480+220 +repage \
    "$FX/tilt_lapis_$bloco.png" && echo "  foto $bloco"
done < "$S/blocos.txt"
python3 -I "$F/tilt_lapis_rebelle.py" regua "$FX" | tee "$S/medidas.txt"
echo TILT_LAPIS_FIM

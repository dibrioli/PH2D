#!/bin/bash
# Espera até a tela da app MUDAR em relação ao estado inicial (ou ao prazo). Uso: bash espera.sh <app> <prazo_s> [saida.png]
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"; P="$2"; O="${3:-$A/_espera.png}"
import -display "$DISPLAY" -window root "$O.0.png"; h0=$(md5sum < "$O.0.png"); t=0
while [ $t -lt "$P" ]; do sleep 3; t=$((t+3)); import -display "$DISPLAY" -window root "$O"
  [ "$(md5sum < "$O")" != "$h0" ] && { echo "mudou aos ${t}s: $O"; exit 0; }; done
echo "igual após ${P}s: $O"

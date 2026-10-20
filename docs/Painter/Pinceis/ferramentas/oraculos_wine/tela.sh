#!/bin/bash
# Abre a tela Xvfb da app (1920x1080x24, só software), prazo 4 h. Uso: bash tela.sh <app>
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"
n="${DISPLAY#:}"
[ -e "/tmp/.X11-unix/X$n" ] && { echo "tela $DISPLAY já aberta"; exit 0; }
setsid nohup timeout 14400 Xvfb "$DISPLAY" -screen 0 1920x1080x24 -nolisten tcp > "$A/xvfb.log" 2>&1 < /dev/null &
for i in $(seq 50); do [ -e "/tmp/.X11-unix/X$n" ] && break; sleep 0.2; done
echo "tela $DISPLAY pronta"

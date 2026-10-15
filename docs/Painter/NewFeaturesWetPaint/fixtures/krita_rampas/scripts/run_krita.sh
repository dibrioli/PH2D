#!/bin/bash
# Uso: run_krita.sh <script.py> <log>   — corre o Krita no Xvfb :97 privado, config/dados descartáveis.
S=$(dirname "$(readlink -f "$0")")
rm -f "$2" "$2.done"
env -u WAYLAND_DISPLAY DISPLAY=:97 QT_QPA_PLATFORM=xcb \
  XDG_CONFIG_HOME=$S/config XDG_DATA_HOME=$S/data XDG_CACHE_HOME=$S/cache HOME=$S/home \
  RAMPAS_SCRIPT="$1" RAMPAS_LOG="$2" RAMPAS_DELAY_MS=${RAMPAS_DELAY_MS:-6000} \
  /usr/bin/krita --nosplash > "$2.stdout" 2>&1 &
echo $!

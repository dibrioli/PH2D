#!/bin/bash
# Dentro da tela virtual (socket ph2d-oraculo-j9): recusa os ecrãs do dono (:0/:1, wayland-1), publica o DISPLAY.
O="$HOME/Apps/rebelle/oraculo"
if [ -z "$DISPLAY" ] || [ "$DISPLAY" = ":0" ] || [ "$DISPLAY" = ":1" ] || [ "$WAYLAND_DISPLAY" != "ph2d-oraculo-j9" ]; then
  echo "RECUSA DISPLAY=$DISPLAY WAYLAND_DISPLAY=$WAYLAND_DISPLAY" > "$O/recusa.txt"; exit 3
fi
echo "$DISPLAY" > "$O/display.txt"
env -u WAYLAND_DISPLAY bash "$HOME/Apps/rebelle/rebelle.sh" -websocket-server-enable -websocket-port 8265 \
  -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1 > "$O/rebelle_j9.log" 2>&1 &
sleep 3600

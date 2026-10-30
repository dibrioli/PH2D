#!/bin/bash
# Corre DENTRO do kwin_wayland --virtual (bancada das gotas): recusa o ecra do dono, publica o DISPLAY,
# lanca o Rebelle em X11 com a porta WebSocket.
O="$HOME/Apps/rebelle/oraculo"
recusa=0
[ -z "$DISPLAY" ] && recusa=1
[ "$DISPLAY" = ":0" ] && recusa=1
printf ' %s ' "${PH2D_X_DONO:-}" | grep -q " ${DISPLAY#:} " && recusa=1
[ -n "${PH2D_WAYLAND_DONO:-}" ] && [ "$WAYLAND_DISPLAY" = "$PH2D_WAYLAND_DONO" ] && recusa=1
if [ "$recusa" = 1 ]; then
  echo "RECUSA DISPLAY=$DISPLAY WAYLAND_DISPLAY=$WAYLAND_DISPLAY dono_x=[$PH2D_X_DONO] dono_wl=[$PH2D_WAYLAND_DONO]" > "$O/recusa.txt"; exit 3
fi
echo "$DISPLAY" > "$O/display.txt"
env -u WAYLAND_DISPLAY bash "$HOME/Apps/rebelle/rebelle.sh" -websocket-server-enable -websocket-port 8265 \
  -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1 > "$O/rebelle_gotas.log" 2>&1 &
echo $! > "$O/rebelle.pid"
sleep 3600

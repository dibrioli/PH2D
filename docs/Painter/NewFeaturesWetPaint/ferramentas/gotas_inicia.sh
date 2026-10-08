#!/bin/bash
# A bancada das GOTAS (doc 06): abre UMA tela virtual 1920×1080 com o Rebelle e a porta WebSocket já
# ligada (gotas_sessao.sh). Corre em segundo plano; prazo 1 h. ⛔ Fechar com gotas_fecha.sh — NUNCA o
# fecha.sh do oráculo, que mata TODA tela virtual (as das outras linhas também).
O="$HOME/Apps/rebelle/oraculo"; F="$(cd "$(dirname "$0")" && pwd)"
rm -f "$O/display.txt" "$O/recusa.txt"
U="$HOME/Apps/rebelle/wineprefix/drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/settings.ini"
sed -i 's/^is_running=1/is_running=0/; s/^clean_exit=false/clean_exit=true/' "$U"
XDG_CONFIG_HOME="$O/cfg" timeout 3600 kwin_wayland --virtual --width 1920 --height 1080 \
  --xwayland --exit-with-session "$F/gotas_sessao.sh" > "$O/kwin_gotas.log" 2>&1

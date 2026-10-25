#!/bin/bash
O="$HOME/Apps/rebelle/oraculo"; J="$(cd "$(dirname "$0")" && pwd)"
rm -f "$O/display.txt" "$O/recusa.txt"
U="$HOME/Apps/rebelle/wineprefix/drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/settings.ini"
sed -i 's/^is_running=1/is_running=0/; s/^clean_exit=false/clean_exit=true/' "$U"
XDG_CONFIG_HOME="$O/cfg" timeout 3600 kwin_wayland --virtual --socket ph2d-oraculo-j9 --width 1920 --height 1080 \
  --xwayland --exit-with-session "$J/sessao.sh" > "$J/kwin.log" 2>&1

#!/bin/bash
# Mata o Rebelle (por PID exacto, nunca por padrão que case com a própria shell), garante use_ink=false,
# relança-o na tela virtual e passa pelos diálogos Registration (Try → Pro) e New Artwork (HD 720).
O="$HOME/Apps/rebelle/oraculo"; cd "$O"
for p in $(pgrep -f "Rebelle 8[.]exe"); do [ "$p" != "$$" ] && kill -9 "$p" 2>/dev/null && echo "matei $p"; done
sleep 3
U="$HOME/Apps/rebelle/wineprefix/drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/settings.ini"
sed -i 's/^use_ink=true/use_ink=false/; s/^is_running=1/is_running=0/; s/^clean_exit=false/clean_exit=true/' "$U"
grep -n "^use_ink\|^is_running\|^clean_exit" "$U"
(DISPLAY=:1 nohup env -u WAYLAND_DISPLAY bash "$HOME/Apps/rebelle/rebelle.sh" > "$O/rebelle3.log" 2>&1 &)
n=0; until bash "$O/inj.sh" "rect Registration" 2>/dev/null | grep -q janela; do sleep 3; n=$((n+1)); [ $n -gt 60 ] && { echo "sem Registration"; exit 1; }; done
echo "registo em $((n*3)) s"
bash "$O/inj.sh" "click 1567 1049; wait 4000; click 1355 1049; wait 1000" >/dev/null
n=0; until bash "$O/inj.sh" "rect New Artwork" 2>/dev/null | grep -q janela; do sleep 3; n=$((n+1)); [ $n -gt 40 ] && { echo "sem New Artwork"; exit 1; }; done
bash "$O/inj.sh" "click 863 563; wait 500; click 1327 790; wait 7000" >/dev/null
bash "$O/inj.sh" "rect e" | grep janela | head -3
echo "instancias: $(pgrep -fc 'Rebelle 8[.]exe')"

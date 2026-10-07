#!/bin/bash
# Lança o Rebelle na tela virtual DESLIGADO desta tarefa (setsid), passa Registration (Try → Pro) e cancela o
# New Artwork (fica o documento de arranque); fotografa e localiza a tela. Cada chamada ao injector tem prazo.
O="$HOME/Apps/rebelle/oraculo"; cd "$O"
U="$HOME/Apps/rebelle/wineprefix/drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/settings.ini"
sed -i 's/^is_running=1/is_running=0/; s/^clean_exit=false/clean_exit=true/' "$U"
DISPLAY=:1 setsid nohup env -u WAYLAND_DISPLAY bash "$HOME/Apps/rebelle/rebelle.sh" -websocket-server-enable -websocket-port 8265 -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1 > "$O/rebelle6.log" 2>&1 < /dev/null &
disown
n=0; until timeout 20 bash inj.sh "rect Registration" 2>/dev/null | grep -q janela; do sleep 3; n=$((n+1)); [ $n -gt 40 ] && { echo "sem Registration"; exit 1; }; done
echo "Registration após $((n*3)) s"
timeout 30 bash inj.sh "click 1567 1049; wait 4000; click 1355 1049; wait 1000" >/dev/null
n=0; until timeout 20 bash inj.sh "rect New Artwork" 2>/dev/null | grep -q janela; do sleep 3; n=$((n+1)); [ $n -gt 40 ] && { echo "sem New Artwork"; timeout 20 bash inj.sh "rect e" | grep janela; exit 1; }; done
echo "New Artwork após $((n*3)) s"
timeout 30 bash inj.sh "click 1407 790; wait 6000" >/dev/null
timeout 20 bash inj.sh "rect e" | grep janela | head -2
timeout 60 bash foto.sh d00 | tail -1
python3 -I - d00.png <<'PY'
import sys, numpy as np
from PIL import Image
a = np.asarray(Image.open(sys.argv[1]).convert("RGB"))
c = a[500, 900]; print("cor no centro:", c)
same = (np.abs(a.astype(int) - c.astype(int)).sum(2) < 6)
rows = np.where(same.sum(1) > 600)[0]; cols = np.where(same.sum(0) > 400)[0]
print("tela na foto: x", cols[0], cols[-1], "y", rows[0], rows[-1], "=>", cols[-1]-cols[0]+1, "x", rows[-1]-rows[0]+1)
PY
echo ARRANQUE_FIM

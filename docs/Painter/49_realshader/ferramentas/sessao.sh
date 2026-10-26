#!/bin/bash
# uso: sessao.sh <tag> [chave=valor ...] — ini → abre Rebelle (porta) → entra → pinta → fotos por parâmetro → fecha.
D="$(cd "$(dirname "$0")" && pwd)"; tag=$1; shift
U="$HOME/Apps/rebelle/wineprefix/drive_c/users/enio/AppData/Local/Escape Motions/Rebelle 8/Data/settings.ini"
cp "$D/settings.ini.dono" "$U"
python3 -I - "$U" "$@" <<'PY'
import sys,struct
u=sys.argv[1]; t=open(u,encoding='latin1').read().split('\n')
def enc(v): return '@Variant(\\0\\0\\0\\x87'+''.join('\\x%02x'%c for c in struct.pack('>f',v))+')'
for kv in sys.argv[2:]:
    k,v=kv.split('=',1); n=0
    for i,l in enumerate(t):
        if l.startswith(k+'='):
            n+=1; t[i]=k+'='+(v if k in ('soft_shadows','raytracer_enabled','real_shader','mat_environmentSphere') else enc(float(v)))
    assert n==1,(k,n)
for i,l in enumerate(t):
    if l.startswith('is_running='): t[i]='is_running=0'
    if l.startswith('clean_exit='): t[i]='clean_exit=true'
open(u,'w',encoding='latin1').write('\n'.join(t))
PY
cd "$HOME/Apps/rebelle/wineprefix/drive_c/Program Files/Rebelle 8"
DISPLAY=:0 setsid env -u WAYLAND_DISPLAY WINEPREFIX="$HOME/Apps/rebelle/wineprefix" WINEDEBUG=-all timeout 900 wine "Rebelle 8.exe" -websocket-server-enable -websocket-port 8265 -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1 > "$D/out/$tag.log" 2>&1 < /dev/null &
cd "$D"
n=0; until bash inj0.sh "rect Registration" 2>/dev/null | grep -q janela; do sleep 2; n=$((n+1)); [ $n -gt 60 ] && { echo "$tag sem Registration"; exit 1; }; done
sleep 3
for k in 1 2 3 4 5; do
  t=$(for w in $(xprop -display :0 -root _NET_CLIENT_LIST | grep -o "0x[0-9a-f]*"); do xprop -display :0 -id $w WM_CLASS | grep -qi rebelle && xprop -display :0 -id $w WM_NAME | cut -d'"' -f2; done)
  echo "  passo $k: $t" | tr '\n' ' '; echo
  echo "$t" | grep -q "New Artwork" && break
  echo "$t" | grep -q "^Rebelle 8 Pro" && ! echo "$t" | grep -q Registration && break
  bash inj0.sh "rect Registration" | grep -q janela && bash inj0.sh "front Registration; wait 400; click 1147 791; wait 3000; front Registration; wait 400; click 1142 791; wait 3000" >/dev/null
  sleep 3
done
n=0; until bash inj0.sh "rect New Artwork" 2>/dev/null | grep -q janela; do sleep 2; n=$((n+1)); [ $n -gt 60 ] && { echo "$tag sem New Artwork"; exit 1; }; done
sleep 3
for k in 1 2 3 4 5; do bash inj0.sh "rect New Artwork" | grep -q janela || break; bash inj0.sh "front New Artwork; wait 400; click 1407 790; wait 2500" >/dev/null; done
bash inj0.sh "rect New Artwork" | grep -q janela && { echo "$tag New Artwork não fecha"; exit 1; }
timeout 120 node "$HOME/Apps/rebelle/oraculo/ws_envia.mjs" "$D/t.jsonl" 8000 | grep -v "Pro here" | tail -3
sleep 6; for w in $(xprop -display :0 -root _NET_CLIENT_LIST | grep -o "0x[0-9a-f]*"); do xprop -display :0 -id $w WM_CLASS | grep -qi rebelle && echo "  janela $w $(xprop -display :0 -id $w WM_NAME | cut -d= -f2)"; done; kscreen-doctor --dpms on >/dev/null 2>&1; sleep 3; bash foto0.sh "$D/out/${tag}_base.png"
if [ "$tag" = A ]; then
  for v in "rs0:\"realshader\":false" "g0:\"gloss\":0" "g5:\"gloss\":5" "i2:\"impasto_depth\":2" "i5:\"impasto_depth\":5" "i10:\"impasto_depth\":10"; do
    nm=${v%%:*}; kv=${v#*:}
    timeout 30 node "$HOME/Apps/rebelle/oraculo/ws_envia.mjs" "{\"event_type\":\"SET_ENGINE_PARAMS\",\"realshader\":true,\"impasto_depth\":8,\"gloss\":10}" 500 >/dev/null
    timeout 30 node "$HOME/Apps/rebelle/oraculo/ws_envia.mjs" "{\"event_type\":\"SET_ENGINE_PARAMS\",$kv}" 1500 >/dev/null
    kscreen-doctor --dpms on >/dev/null 2>&1; sleep 3; bash foto0.sh "$D/out/${tag}_$nm.png" >/dev/null
  done
fi
WINEPREFIX="$HOME/Apps/rebelle/wineprefix" wineserver -k; for i in $(seq 1 30); do pgrep -f "Rebelle 8[.]exe" >/dev/null || break; sleep 1; done; sleep 3
cp "$D/settings.ini.dono" "$U"
echo "$tag ok: $(ls out | grep "^${tag}_" | tr '\n' ' ')"

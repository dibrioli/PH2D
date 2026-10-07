#!/bin/bash
# Regenera os 12 conjuntos de eventos (SET_BRUSH completo antes de cada gesto) e corre-os no Rebelle.
# ⛔ Sem nenhum kill por padrão: um `pgrep -f` com o nome do guião mata a shell que o chamou.
O="$HOME/Apps/rebelle/oraculo"; cd "$O"
P="Z:/home/enio/Apps/rebelle/oraculo/saida"
python3 gera_eventos.py OIL_AND_ACRYLIC "Bristle/Bristle Flat" $P oil > ev_oil.json || { echo "gerador falhou"; exit 1; }
python3 gera_eventos.py EXPRESS_OIL "Round/Hairy" $P express > ev_express.json
python3 gera_eventos.py WATERCOLOR "Watercolor/Flat 2" $P wc > ev_wc.json
python3 gera_eventos.py INK_PEN "Pen/Bullet" $P ink > ev_ink.json
python3 gera_eventos.py PENCIL "Graphite Pencil/8B" $P pencil > ev_pencil.json
python3 gera_eventos.py PASTEL "Pastel Dry/Dry Thin" $P pastel > ev_pastel.json
python3 gera_eventos.py MARKER "Marker/Liner" $P marker > ev_marker.json
python3 gera_eventos.py AIRBRUSH "Default/Spray 2" $P airbrush > ev_airbrush.json
python3 gera_eventos.py WATERCOLOR "Watercolor/Flat 2" $P wcpreset --opacity 50 --water 30 > ev_wcpreset.json
python3 gera_eventos.py WATERCOLOR "Watercolor/Round" $P wcwet --opacity 60 --water 80 > ev_wcwet.json
python3 gera_eventos.py PENCIL "Graphite Pencil/8B" $P penciltilt --tilt 55 0 > ev_penciltilt.json
python3 gera_eventos.py OIL_AND_ACRYLIC "Bristle/Bristle Flat" $P oildirty --dirty --paint_type PAINT_MIX > ev_oildirty.json
python3 - <<'PY'
import json, glob
for f in sorted(glob.glob("ev_*.json")):
    if f.endswith("_A.json") or f.endswith("_B.json"): continue
    d = json.load(open(f)); ev = [e for e in d["frames"][0]["events"] if e["event_type"] != "SAVE"]
    k = next((i for i, e in enumerate(ev) if e["event_type"] == "SIMULATION"), len(ev)); A = ev[:k+1]
    json.dump({"frames": [{"events": A}]}, open(f.replace(".json", "_A.json"), "w"))
    json.dump({"frames": [{"events": [{"event_type": "DRY"}]}]}, open(f.replace(".json", "_B.json"), "w"))
d = json.load(open("ev_ink_A.json")); sbs = [e for e in d["frames"][0]["events"] if e["event_type"] == "SET_BRUSH"]
print(len(sbs), "SET_BRUSH; 1.º:", sbs[0]); print("7.º cor:", sbs[6]["color"])
PY
echo "--- a correr os 12"
./corre_todas.sh 2>&1 | grep -v "^ligado\|^fechado\|enviados" | tail -2

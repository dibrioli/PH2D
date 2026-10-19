#!/bin/bash
# Fotografa um LOTE de presets do Rebelle na bancada dos gestos (gera_eventos.py, a mesma da sonda_midias):
# por linha da lista `nome|TOOL|Grupo/Nome[|base TOOL:Grupo/Nome ou args do gerador[|r,g,b do topo]]` — eventos, A (traços + SIMULATION), zoom 100 %, foto, B (DRY),
# foto, recorte 1000×640. Com `base` (outro `TOOL:Grupo/Nome`), a bancada é pintada PRIMEIRO com a base e o preset
# passa por cima dos mesmos gestos (os marcadores de efeito: sobre papel branco não deixam marca).
# Precisa da tela virtual com a porta ligada (gotas_inicia.sh + Registration/New Artwork já passados).
# uso: fotos_lote.sh <lista> <pasta de saída>
set -u
O="$HOME/Apps/rebelle/oraculo"; F="$(cd "$(dirname "$0")" && pwd)"; L="$1"; S="$2"
mkdir -p "$S"
P="Z:${S}"
divide() { # ev.json → ev_A.json (sem SAVE, até à SIMULATION) e ev_B.json (DRY)
  python3 -I - "$1" <<'PY'
import json, sys
f = sys.argv[1]
ev = [e for e in json.load(open(f))["frames"][0]["events"] if e["event_type"] != "SAVE"]
k = next((i for i, e in enumerate(ev) if e["event_type"] == "SIMULATION"), len(ev))
json.dump({"frames": [{"events": ev[:k + 1]}]}, open(f.replace(".json", "_A.json"), "w"))
json.dump({"frames": [{"events": [{"event_type": "DRY"}]}]}, open(f.replace(".json", "_B.json"), "w"))
PY
}
n=0
while IFS='|' read -r nm tool preset base cor; do
  [ -z "$nm" ] || [ "${nm:0:1}" = "#" ] && continue
  ev="$S/ev_${nm}.json"
  if [[ "${base:-}" == *:* && "${base:-}" != --* ]]; then
    python3 -I "$F/gera_eventos.py" "${base%%:*}" "${base#*:}" "$P" "$nm" > "$S/base_${nm}.json" || { echo "✗ $nm base"; continue; }
    python3 -I "$F/gera_eventos.py" "$tool" "$preset" "$P" "$nm" --so_tracos > "$S/topo_${nm}.json" || { echo "✗ $nm topo"; continue; }
    python3 -I - "$S/base_${nm}.json" "$S/topo_${nm}.json" "$ev" "${cor:-}" <<'PY'
import json, sys
b = json.load(open(sys.argv[1]))["frames"][0]["events"]
t = json.load(open(sys.argv[2]))["frames"][0]["events"]
if sys.argv[4]:  # o efeito numa cor DIFERENTE da base (Hue/Color/Saturation da mesma cor não mudam nada)
    r, g, bb = map(int, sys.argv[4].split(","))
    for e in t:
        if "color" in e:
            e["color"] = {"r": r, "g": g, "b": bb}
k = next(i for i, e in enumerate(b) if e["event_type"] == "SIMULATION")
b = [e for e in b[:k + 1] if e["event_type"] != "SAVE"] + [{"event_type": "DRY"}]
json.dump({"frames": [{"events": b + t + [{"event_type": "SIMULATION", "repeats": 240}]}]}, open(sys.argv[3], "w"))
PY
  else
    # Sem base, o 4.º campo são argumentos do gerador (ex.: `--size_px 80`).
    # shellcheck disable=SC2086
    python3 -I "$F/gera_eventos.py" "$tool" "$preset" "$P" "$nm" ${base:-} > "$ev" || { echo "✗ $nm eventos"; continue; }
  fi
  divide "$ev"
  timeout 180 node "$F/ws_eventos.mjs" "${ev%.json}_A.json" 6000 2 | grep -v "SAVE and LOAD\|^ligado\|^fechado\|enviados" | tail -1
  timeout 30 bash "$O/inj.sh" "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1500" >/dev/null
  timeout 60 bash "$O/foto.sh" "ws_${nm}_A" >/dev/null
  timeout 60 node "$F/ws_eventos.mjs" "${ev%.json}_B.json" 4000 2 >/dev/null
  timeout 60 bash "$O/foto.sh" "ws_${nm}_B" >/dev/null
  for s in A B; do
    [ -f "$O/ws_${nm}_${s}.png" ] && magick "$O/ws_${nm}_${s}.png" -crop 1000x640+480+220 +repage "$S/c_${nm}_${s}.png" \
      && rm -f "$O/ws_${nm}_${s}.png"
  done
  [ -f "$S/c_${nm}_B.png" ] && { echo "✓ $nm"; n=$((n+1)); } || echo "✗ $nm sem foto"
done < "$L"
echo "LOTE_FIM $n"

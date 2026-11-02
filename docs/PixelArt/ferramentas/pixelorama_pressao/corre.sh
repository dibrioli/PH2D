#!/usr/bin/env bash
# Oráculo Pixelorama 1.2.3: pressão simulada -> PNG. Uso: bash corre.sh <pasta_de_trabalho>
# Xvfb próprio no :93, HOME isolado; nunca o ecrã real. A instalação em ~/.local/opt não é tocada.
set -euo pipefail
W=${1:?pasta de trabalho}; AQUI=$(cd "$(dirname "$0")" && pwd)
APP=${PIXELORAMA:-$HOME/.local/opt/pixelorama/Pixelorama-Linux-64bit/Pixelorama.x86_64}
mkdir -p "$W/home/.local/share/pixelorama/extensions" "$W/out"
(cd "$AQUI/extensao" && python3 -I -c "
import zipfile,os,sys
z=zipfile.ZipFile(sys.argv[1],'w')
for r,_,fs in os.walk('src'):
  for f in fs: z.write(os.path.join(r,f))
z.close()" "$W/home/.local/share/pixelorama/extensions/PressaoProbe.zip")
printf '[extensions]\n\nPressaoProbe=true\n' > "$W/home/.local/share/pixelorama/config.ini"
Xvfb :93 -screen 0 1600x1000x24 -nolisten tcp >"$W/xvfb.log" 2>&1 & XP=$!
trap 'kill $XP 2>/dev/null || true' EXIT
sleep 2
env -u WAYLAND_DISPLAY DISPLAY=:93 HOME="$W/home" PROBE_OUT="$W/out" timeout 120 "$APP" >"$W/run.log" 2>&1 || true
grep -E '^PROBE' "$W/run.log"
python3 -I "$AQUI/medir.py" "$W/out" size_default_thr size_thr_0_1 alpha_thr_0_1

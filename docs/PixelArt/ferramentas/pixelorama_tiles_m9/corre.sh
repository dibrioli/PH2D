#!/usr/bin/env bash
# Oraculo Pixelorama 1.2.3, rodada M9. Uso: bash corre.sh <pasta_de_trabalho> <PROBE_MODE> [display]
# Xvfb proprio, HOME isolado; nunca o ecra real. A instalacao em ~/.local/opt nao e tocada.
set -euo pipefail
W=${1:?pasta}; MODE=${2:?modo}; D=${3:-96}; EXT=TilesProbeM9; AQUI=$(cd "$(dirname "$0")" && pwd)
APP=${PIXELORAMA:-$HOME/.local/opt/pixelorama/Pixelorama-Linux-64bit/Pixelorama.x86_64}
rm -rf "$W/home"; mkdir -p "$W/home/.local/share/pixelorama/extensions" "$W/out"
(cd "$AQUI/extensao" && python3 -I -c "
import zipfile,os,sys
z=zipfile.ZipFile(sys.argv[1],'w')
for r,_,fs in os.walk('src'):
  for f in fs: z.write(os.path.join(r,f))
z.close()" "$W/home/.local/share/pixelorama/extensions/$EXT.zip")
printf '[extensions]\n\n%s=true\n' "$EXT" > "$W/home/.local/share/pixelorama/config.ini"
Xvfb :$D -screen 0 1600x1000x24 -nolisten tcp >"$W/xvfb.log" 2>&1 & XP=$!
trap 'kill $XP 2>/dev/null || true' EXIT
sleep 2
env -u WAYLAND_DISPLAY DISPLAY=:$D HOME="$W/home" PROBE_OUT="$W/out" PROBE_MODE="$MODE" timeout ${PROBE_T:-180} "$APP" >"$W/run.log" 2>&1 || true
grep -E '^PROBE' "$W/run.log" || tail -30 "$W/run.log"

#!/usr/bin/env bash
# Roteiro CONDUZIDO da cena 8 do CodeEditor (fase 8d, os scripts tocam as propriedades pelo nome): o binário de
# smoke em `PH2D_CODE_SMOKE=8` numa Xvfb 1280x800 (a janela abre a 1024x768 em 0,0). A cena arranca a correr e
# ninguém mexe: as fotos marcam o tempo — o arranque, os espinhos, a moeda, a porta aberta, a câmera perto — e o
# Shift+F5 (parar e rebobinar) no fim. A saída do app (o console do script: os nomes da câmera) fica em saida.txt.
# Uso (da raiz da worktree, com o binário de smoke feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/conduz_propriedades/corre.sh
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/conduz_propriedades.XXXX); mkdir -p "$T/home"
DSP=:83
Xvfb $DSP -screen 0 1280x800x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_CODE_SMOKE=8 \
  timeout 240 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 60); do grep -q 'code-smoke\] =8' "$T/app.log" 2>/dev/null && break; sleep 0.25; done
T0=$(date +%s%3N)
foto() { # segundos-desde-a-montagem nome
  local alvo=$(( T0 + $1 * 1000 )); while [ "$(date +%s%3N)" -lt "$alvo" ]; do sleep 0.05; done
  import -window root "$AQUI/$2.png" 2>/dev/null
}
foto 1 foto_01_arranque
foto 2 foto_02_espinhos
foto 4 foto_03_moeda
foto 5 foto_04_porta_aberta
foto 9 foto_05_camera_perto
W=$(xdotool search --onlyvisible --name . 2>/dev/null | head -1)
xdotool windowfocus --sync "$W"; xdotool key shift+F5; sleep 1.5
import -window root "$AQUI/foto_06_rebobinado.png" 2>/dev/null
{ echo "# cena 8 conduzida — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg) · HEAD $(git -C "$RAIZ" rev-parse --short HEAD)"
  grep -E 'code-smoke|height|script|error|panick' "$T/app.log" | head -40; } >"$AQUI/saida.txt"
cat "$AQUI/saida.txt"

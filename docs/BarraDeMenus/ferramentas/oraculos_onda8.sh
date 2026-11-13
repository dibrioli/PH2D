#!/usr/bin/env bash
# Oráculos da onda 8 (os MENUS abertos, MEDIDOS; molde: oraculos_onda7.sh): corre VS Code (barra de título «custom» com a busca
# no centro — o análogo directo da maquete C), Godot 4 e Blender numa Xvfb PRÓPRIA a escala 1, HOME de faz
# de conta, e fotografa o topo da janela. Só a SAÍDA é usada (CLAUDE.md §0.9): nada de fonte lido.
# Uso (da raiz da worktree):  PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/BarraDeMenus/ferramentas/oraculos_onda7.sh
set -uo pipefail
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${SAIDA:-$RAIZ/docs/BarraDeMenus/fotos/oraculos}
mkdir -p "$SAIDA"
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
JW=1930 JH=1040

um() { # nome espera comando...
  local nome=$1 espera=$2; shift 2
  local T; T=$(mktemp -d /tmp/oraculo8.XXXX); mkdir -p "$T/home"
  xvfb_livre ${JW}x${JH}x24 "$T/xvfb.log" 240 || return 1
  (cd "$T" && env -u WAYLAND_DISPLAY HOME="$T/home" XDG_SESSION_TYPE=x11 GDK_SCALE=1 QT_SCALE_FACTOR=1 \
     LIBGL_ALWAYS_SOFTWARE=1 timeout 200 "$@" >"$T/app.log" 2>&1) &
  local AP=$!
  sleep "$espera"
  local W; W=$(xdotool search --onlyvisible --name . 2>/dev/null | tail -1)
  [ -n "$W" ] && xdotool windowsize "$W" $JW $JH windowmove "$W" 0 0 2>/dev/null
  sleep 4
  if [ -n "${TECLAS:-}" ]; then
    [ -n "$W" ] && xdotool windowfocus --sync "$W" 2>/dev/null
    for k in $TECLAS; do xdotool key "$k"; sleep 1; done; [ -z "${CLIQUE:-}" ] && { xdotool mousemove 960 600; sleep 2; }
  fi
  if [ -n "${CLIQUE:-}" ]; then
    for p in $CLIQUE; do xdotool mousemove ${p%,*} ${p#*,}; sleep 0.8; done; xdotool click 1; sleep 1.5
    for p in ${PAIRA:-}; do xdotool mousemove ${p%,*} ${p#*,}; sleep 0.6; done; sleep 1.5
  fi
  import -window root "$SAIDA/$nome.png" && echo "foto: $SAIDA/$nome.png"
  cp "$T/app.log" "$SAIDA/$nome.log" 2>/dev/null
  kill "$AP" 2>/dev/null; pkill -P "$AP" 2>/dev/null; kill "$XV" 2>/dev/null
  sleep 1; rm -rf "$T"
}

# VS Code: barra de título desenhada pelo app, Command Center ligado, tema escuro de fábrica.
vscode() {
  local ud; ud=$(mktemp -d /tmp/oraculo8_vsc.XXXX); mkdir -p "$ud/User" "$ud/proj"
  printf 'fn main() {}\n' >"$ud/proj/main.rs"
  cat >"$ud/User/settings.json" <<'J'
{ "window.titleBarStyle": "custom", "window.commandCenter": true, "workbench.layoutControl.enabled": true,
  "window.zoomLevel": 0, "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off",
  "update.mode": "none", "workbench.colorTheme": "Default Dark Modern", "security.workspace.trust.enabled": false, "chat.disableAIFeatures": true,
  "workbench.welcomePage.experimentalOnboarding": false, "workbench.tips.enabled": false }
J
  TECLAS="Escape Escape alt+f Down Down Down Down Down Down Down Down Down Down Down Down Right" um vscode_menu 14 code --user-data-dir "$ud" --extensions-dir "$ud/ext" --disable-gpu --no-sandbox \
     --new-window --wait "$ud/proj" "$ud/proj/main.rs"
  rm -rf "$ud"
}

godot4() {
  local p; p=$(mktemp -d /tmp/oraculo8_gd.XXXX)
  printf 'config_version=5\n[application]\nconfig/name="oraculo"\n' >"$p/project.godot"
  CLIQUE="${GODOT_MENU:-30,12}" PAIRA="${GODOT_PAIRA:-}" um godot_menu 18 godot --editor --path "$p" --rendering-driver opengl3 --display-driver x11
  rm -rf "$p"
}

blender5() { um blender 14 blender --factory-startup; }

case "${1:-tudo}" in
  vscode) vscode ;; godot) godot4 ;; blender) blender5 ;;
  *) vscode; godot4 ;;
esac

#!/usr/bin/env bash
# O app INTEIRO, nativo no Wayland de um KWin virtual: outro cliente larga «água teste.png» no meio
# do ecrã e o roteiro fotografa o depois (ScreenShot2 da sessão virtual).
#
# Uso: bash docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome> [CHAVE=VALOR …]
#   ex.: no_app.sh cena                        → a cena de abertura: a imagem entra como sprite
#        no_app.sh quadro PH2D_BOARD_SMOKE=2   → um quadro na aba: a recusa aparece na tela
# Saída: docs/MiroClone/ferramentas/soltar_wayland/<nome>_depois.png (meio segundo depois de largar)
# ⛔ `--no-lockscreen`: sem ele o KWin virtual engole o rato falso (ver corre.sh).
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(cd "$AQUI/../../../.." && pwd)"
NOME="${1:?nome da corrida}"
shift
ENV_KV="$*"
cd "$RAIZ"

APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta: bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
NOVO="$(find "$RAIZ/crates" "$RAIZ/shells" -name '*.rs' -not -path '*/tests/*' -not -name '*_tests.rs' -newer "$APP" -print -quit 2>/dev/null || true)"
[ -z "$NOVO" ] || { echo "RECUSA: ha' codigo mais novo que o binario (${NOVO#"$RAIZ/"})" >&2; exit 2; }
FONTE="$(bash scripts/ph2d-run.sh cargo test -p ph2d-wayland-dnd --test kwin_drop --no-run 2>&1 \
  | sed -n 's/.*Executable tests\/kwin_drop.rs (\(.*\))/\1/p' | tail -1)"
[ -x "$RAIZ/$FONTE" ] || { echo "RECUSA: a origem do arrasto nao compilou" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg" "$TMP/runtime" "$TMP/home" "$TMP/ficheiros"
chmod 700 "$TMP/runtime"
IMG="$TMP/ficheiros/água teste.png"
magick -size 160x120 gradient:'#2a9d8f-#e9c46a' "$IMG"
cat > "$TMP/cfg/kwinrulesrc" <<'EOF'
[General]
count=1
rules=r1

[r1]
Description=maximiza tudo
wmclassmatch=0
types=1
maximizehoriz=true
maximizehorizrule=3
maximizevert=true
maximizevertrule=3
EOF

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$WAYLAND_DISPLAY" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a sessao virtual" > "$TMP/log"
  exit 3
fi
cd "$RAIZ"
env HOME="$TMP/home" $ENV_KV PH2D_EXIT_AFTER_FRAMES=100000 "$APP" > "$TMP/app.log" 2>&1 &
PID=\$!
sleep 12
PH2D_DND_FILES="$IMG" "$RAIZ/$FONTE" --ignored --exact drag_the_env_files_onto_the_screen_centre --nocapture >> "$TMP/log" 2>&1
echo "origem=\$?" >> "$TMP/log"
sleep 0.5
python3 -I "$AQUI/foto_kwin.py" "$AQUI/${NOME}_depois.png" >> "$TMP/log" 2>&1
kill -9 \$PID 2>/dev/null
EOF
chmod +x "$TMP/sessao.sh"
if grep -q '`' "$TMP/sessao.sh"; then
  echo "RECUSA: o roteiro gerado tem uma CRASE" >&2
  exit 4
fi

XDG_CONFIG_HOME="$TMP/cfg" XDG_RUNTIME_DIR="$TMP/runtime" KWIN_WAYLAND_NO_PERMISSION_CHECKS=1 \
  KWIN_SCREENSHOT_NO_PERMISSION_CHECKS=1 \
  timeout 180 dbus-run-session -- kwin_wayland --virtual --no-lockscreen --no-global-shortcuts \
  --width 1280 --height 800 --socket ph2d-dnd-app --exit-with-session "$TMP/sessao.sh" \
  > "$TMP/kwin.log" 2>&1 || true

cat "$TMP/log" 2>/dev/null || true
echo "--- app.log (fim) ---"
tail -15 "$TMP/app.log" 2>/dev/null || true
grep -q '^origem=0$' "$TMP/log"

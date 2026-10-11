#!/usr/bin/env bash
# O receptor de «soltar ficheiros» (crate ph2d-wayland-dnd) contra um KWin DE VERDADE, invisível.
#
# Um `kwin_wayland --virtual` num D-Bus PRÓPRIO (o da sessão do dono tem o KWin dele com o mesmo
# nome), com o `org_kde_kwin_fake_input` liberado por KWIN_WAYLAND_NO_PERMISSION_CHECKS=1: o rato do
# teste é o da sessão virtual — nunca `ydotool` (move o rato REAL) nem `spectacle`.
# ⛔ `--no-lockscreen`: sem ele o KWin virtual engole TODO o rato falso, calado (medido 11/10: zero
# `wl_pointer` em 5 s de movimentos, com o dispositivo criado e autorizado).
#
# Uso: bash docs/MiroClone/ferramentas/soltar_wayland/corre.sh   (exit 0 = o teste passou)
set -euo pipefail
RAIZ="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$RAIZ"

BIN="$(bash scripts/ph2d-run.sh cargo test -p ph2d-wayland-dnd --test kwin_drop --no-run 2>&1 \
  | sed -n 's/.*Executable tests\/kwin_drop.rs (\(.*\))/\1/p' | tail -1)"
[ -x "$RAIZ/$BIN" ] || { echo "RECUSA: o teste nao compilou" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg" "$TMP/runtime"
chmod 700 "$TMP/runtime"

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$WAYLAND_DISPLAY" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a sessao virtual (WAYLAND_DISPLAY=\$WAYLAND_DISPLAY)" > "$TMP/log"
  exit 3
fi
"$RAIZ/$BIN" --ignored --nocapture --test-threads=1 > "$TMP/log" 2>&1
echo "exit=\$?" >> "$TMP/log"
EOF
chmod +x "$TMP/sessao.sh"
if grep -q '`' "$TMP/sessao.sh"; then
  echo "RECUSA: o roteiro gerado tem uma CRASE" >&2
  exit 4
fi

XDG_CONFIG_HOME="$TMP/cfg" XDG_RUNTIME_DIR="$TMP/runtime" KWIN_WAYLAND_NO_PERMISSION_CHECKS=1 \
  timeout 120 dbus-run-session -- kwin_wayland --virtual --no-lockscreen --no-global-shortcuts --width 1280 --height 800 \
  --socket ph2d-dnd --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
cp "$TMP/kwin.log" /tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/9b80c6d0-9127-45af-ae3e-e8c45393c509/scratchpad/kwin.log 2>/dev/null || true

cat "$TMP/log" 2>/dev/null || { echo "o teste nem correu; o KWin disse:" >&2; tail -20 "$TMP/kwin.log" >&2; exit 2; }
grep -q '^exit=0$' "$TMP/log"

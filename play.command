#!/usr/bin/env bash
# ▶ Play.command — double-clickable launcher for the PH2D desktop shell.
#
# Saved with .command extension so macOS Finder treats it as executable
# (double-click → opens Terminal + runs). Useful when launching from outside a
# shell session.
#
# Runs the editor in RELEASE with the `wash` + `fluid` features ON — live
# watercolor on the GPU. `wash` (ADR-0086..0091) is the minimal core Enio uses:
# toggle "Wash" in the Brush Studio panel to paint with it (mutually exclusive
# with the "Fluid" toggle, ADR-0049 W15.3 — both compiled so both toggles are
# live). `scripts/run-shell.sh` does NOT enable these features, so this launcher
# runs cargo directly. Uses the warm build slot so the first launch reuses the
# existing build instead of a full rebuild in `target/`.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Reuse the warm build slot (created/maintained during development); cargo
# rebuilds inside it if it's ever emptied, so this stays correct either way.
export CARGO_TARGET_DIR="$SCRIPT_DIR/target-slots/slot-brushoverhaul"

echo "[play] building + launching PH2D editor (release, --features \"wash fluid\")…"
echo "[play] (toggle \"Wash\" in Brush Studio to paint watercolor — close the window or Cmd+Q to exit)"
exec cargo run -p ph2d-host-desktop --release --features "wash fluid"

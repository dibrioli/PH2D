#!/usr/bin/env bash
# ▶ Play.command — double-clickable launcher for the PH2D desktop shell.
#
# Saved with .command extension so macOS Finder treats it as executable
# (double-click → opens Terminal + runs). Useful when launching from outside a
# shell session.
#
# Runs the editor in RELEASE with the `fluid` feature ON — ADR-0049 W15.3 live
# watercolor wet-on-wet on the GPU. `scripts/run-shell.sh` does NOT enable the
# feature, so this launcher runs cargo directly. Uses the warm build slot so the
# first launch reuses the existing build instead of a full rebuild in `target/`.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Reuse the warm build slot (created/maintained during development); cargo
# rebuilds inside it if it's ever emptied, so this stays correct either way.
export CARGO_TARGET_DIR="$SCRIPT_DIR/target-slots/slot-brushoverhaul"

echo "[play] building + launching PH2D editor (release, --features fluid)…"
echo "[play] (close the window or Cmd+Q to exit)"
exec cargo run -p ph2d-host-desktop --release --features fluid

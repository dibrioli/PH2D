#!/usr/bin/env bash
# ▶ Play.command — double-clickable launcher for the PH2D desktop shell.
#
# Saved with .command extension so macOS Finder treats it as executable
# (double-click → opens Terminal + runs). Useful when launching from outside a
# shell session.
#
# Runs the editor from THIS checkout in the `smoke` profile, the same binary the
# owner tests (CLAUDE.md §2): opens at once when already built, ~1 min otherwise.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "[play] building + launching PH2D editor (smoke profile)…"
echo "[play] (close the window or Cmd+Q to exit)"
exec cargo run -p ph2d-host-desktop --profile smoke

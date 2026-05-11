#!/usr/bin/env bash
# ▶ Reference.command — double-clickable launcher for the FROZEN
# `screens::hero_ref/` snapshot of the PH2D desktop shell.
#
# Pair with `play.command` (current hero) to A/B compare while
# iterating on the working hero. Saved with .command extension so
# macOS Finder treats it as executable (double-click → opens Terminal
# + runs).
#
# Forwards to scripts/run-shell-reference.sh in release mode.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
exec "$SCRIPT_DIR/scripts/run-shell-reference.sh"

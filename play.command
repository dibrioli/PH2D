#!/usr/bin/env bash
# ▶ Play.command — double-clickable launcher for the PH2D desktop shell.
#
# Saved with .command extension so macOS Finder treats it as
# executable (double-click → opens Terminal + runs). Useful when
# launching from outside a shell session.
#
# Forwards to scripts/run-shell.sh in release mode.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
exec "$SCRIPT_DIR/scripts/run-shell.sh"

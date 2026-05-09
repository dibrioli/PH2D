#!/usr/bin/env bash
# Build (incremental) + launch the PH2D desktop shell.
#
# Usage:
#   ./scripts/run-shell.sh           # release build (default — best perf)
#   ./scripts/run-shell.sh dev       # debug build (fastest compile, slow runtime)
#   ./scripts/run-shell.sh clean     # wipe target/ first (full rebuild)
#
# What it does:
#   1. Resolves the workspace root (this script's parent dir's parent)
#   2. Optionally `cargo clean` if "clean" arg passed
#   3. `cargo run -p ph2d-host-desktop --release` (or --no --release for dev)
#
# The shell binary auto-generates 16 PNG fixtures in assets/sprites/
# on first launch (gitignored). Close the window or Cmd+Q to exit.

set -euo pipefail

# Resolve the workspace root regardless of where the script is invoked from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

PROFILE="release"
if [[ "${1:-}" == "dev" ]]; then
    PROFILE="dev"
elif [[ "${1:-}" == "clean" ]]; then
    echo "[run-shell] cargo clean"
    cargo clean
fi

PROFILE_FLAG=""
if [[ "$PROFILE" == "release" ]]; then
    PROFILE_FLAG="--release"
fi

echo "[run-shell] cargo run -p ph2d-host-desktop $PROFILE_FLAG"
echo "[run-shell] (close window or Cmd+Q to exit)"
exec cargo run -p ph2d-host-desktop $PROFILE_FLAG

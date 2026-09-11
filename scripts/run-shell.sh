#!/usr/bin/env bash
# Build (incremental) + launch the PH2D desktop shell.
#
# Usage:
#   ./scripts/run-shell.sh           # `smoke` profile (default since 2026-09-10: optimized, no LTO,
#                                    #  incremental — a one-line fix rebuilds in ~3 s instead of 161 s)
#   ./scripts/run-shell.sh release   # the delivery profile (cgu = 1 + thin LTO): PERFORMANCE smokes only
#   ./scripts/run-shell.sh dev       # debug build (fastest compile, slow runtime)
#   ./scripts/run-shell.sh clean     # wipe target/ first (full rebuild)
#
# What it does:
#   1. Resolves the workspace root (this script's parent dir's parent)
#   2. Optionally `cargo clean` if "clean" arg passed
#   3. `cargo run -p ph2d-host-desktop --profile smoke` (or --release / dev)
#
# The shell binary auto-generates 16 PNG fixtures in assets/sprites/
# on first launch (gitignored). Close the window or Cmd+Q to exit.

set -euo pipefail

# Resolve the workspace root regardless of where the script is invoked from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

PROFILE="smoke"
if [[ "${1:-}" == "dev" ]]; then
    PROFILE="dev"
elif [[ "${1:-}" == "release" ]]; then
    PROFILE="release"
elif [[ "${1:-}" == "clean" ]]; then
    echo "[run-shell] cargo clean"
    cargo clean
fi

PROFILE_FLAG=""
if [[ "$PROFILE" == "release" ]]; then
    PROFILE_FLAG="--release"
elif [[ "$PROFILE" == "smoke" ]]; then
    PROFILE_FLAG="--profile smoke"
fi

echo "[run-shell] cargo run -p ph2d-host-desktop $PROFILE_FLAG"
echo "[run-shell] (PH2D editor — close window or Cmd+Q to exit)"
# Default mode = full editor (TopBar / LeftRail / Hierarchy /
# Inspector / BottomHUD) with the M14.4a live ECS bridge wired since
# the 2026-05-14 gate inversion. The legacy 1000-sprite M5 perf demo
# is opt-in via `PH2D_M5_DEMO=1` for HR-4 bench / stress work.
exec cargo run -p ph2d-host-desktop $PROFILE_FLAG

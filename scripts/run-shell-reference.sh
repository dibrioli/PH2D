#!/usr/bin/env bash
# Launch the PH2D desktop shell built against the FROZEN reference
# snapshot of `screens::hero`. The reference lives at
# `crates/ph2d-editor/src/screens/hero_ref/` — a verbatim copy taken
# at the moment the snapshot was created — and is selected via the
# `reference-snapshot` cargo feature.
#
# Use this to A/B against the working hero you are iterating on:
#   ./play.command       → current hero (the one you're editing)
#   ./reference.command  → frozen baseline (unchanged since snapshot)
#
# Usage:
#   ./scripts/run-shell-reference.sh           # release (default)
#   ./scripts/run-shell-reference.sh dev       # debug build
#   ./scripts/run-shell-reference.sh clean     # wipe target/ first
#
# Note: switching between play.command and reference.command rebuilds
# ph2d-editor + ph2d-host-desktop because cargo features change the
# compiled artifact. Subsequent runs of the SAME variant are cached.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

PROFILE="release"
if [[ "${1:-}" == "dev" ]]; then
    PROFILE="dev"
elif [[ "${1:-}" == "clean" ]]; then
    echo "[run-shell-reference] cargo clean"
    cargo clean
fi

PROFILE_FLAG=""
if [[ "$PROFILE" == "release" ]]; then
    PROFILE_FLAG="--release"
fi

echo "[run-shell-reference] cargo run -p ph2d-host-desktop $PROFILE_FLAG --features reference-snapshot"
echo "[run-shell-reference] (frozen hero_ref/ snapshot — close window or Cmd+Q to exit)"
exec cargo run -p ph2d-host-desktop $PROFILE_FLAG --features reference-snapshot

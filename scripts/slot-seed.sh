#!/usr/bin/env bash
# slot-seed.sh <slot-id> — create a warm CoW clone of target-slots/base for this
# slot and print the CARGO_TARGET_DIR to PREFIX cargo commands with.
#
# WHY this exists separate from slot-env.sh: Claude Code agents run cargo via the
# Bash tool, where (a) `source`-ing a BASH_SOURCE-guarded script misbehaves under
# zsh, and (b) env vars do NOT persist between tool calls. So the robust pattern
# is: RUN this once (not source), then PREFIX every cargo command with the printed
# CARGO_TARGET_DIR. Works in zsh and bash.
#
# Usage:
#   bash scripts/slot-seed.sh impl-sprite
#     → prints (stdout):  CARGO_TARGET_DIR=/abs/.../target-slots/slot-impl-sprite
#   then, on EVERY cargo call:
#     CARGO_TARGET_DIR=/abs/.../target-slots/slot-impl-sprite cargo check -p ph2d-render
set -euo pipefail

id="${1:?usage: bash scripts/slot-seed.sh <slot-id>   (e.g. impl-sprite)}"
root="$(git rev-parse --show-toplevel)"
base="$root/target-slots/base"
slot="$root/target-slots/slot-$id"

if [ -d "$slot" ]; then
    echo "[slot-seed] slot-$id already exists — reusing." >&2
elif [ -d "$base" ]; then
    # APFS clonefile: near-instant, 0 physical bytes (shares blocks copy-on-write).
    # Only works on the same APFS volume as base (target-slots is on MAC_EXTERNO).
    cp -c -R "$base" "$slot"
    echo "[slot-seed] CoW-cloned slot-$id from warm base (~1s, 0 bytes)." >&2
else
    mkdir -p "$slot"
    echo "[slot-seed] WARNING: no warm base at target-slots/base — created COLD slot." >&2
    echo "[slot-seed] Ask the Coordinator to build it: CARGO_TARGET_DIR=$base cargo check --workspace" >&2
fi

# stdout = the one line an agent prefixes cargo with (or `eval export` in a shell).
echo "CARGO_TARGET_DIR=$slot"

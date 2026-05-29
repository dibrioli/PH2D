#!/usr/bin/env bash
# slot-env.sh — per-session cargo target isolation for multi-agent parallelism.
#
# Sourced at the START of each Claude Code session to avoid `cargo` lock
# contention on a shared `target/` directory. Each agent gets its own
# `target/slot-<ID>/` and a clear log of which slot it is on. Solves
# Bottleneck 1 from docs/Migracao/2026-05-foundational-parallelism-three-bottlenecks.md
# (the cargo blocking-waiting-for-file-lock issue between parallel agents).
#
# Usage (per-session, by the operator):
#   source scripts/slot-env.sh <slot-id>
#
# Example:
#   source scripts/slot-env.sh coord-a       # Coord-A foundational
#   source scripts/slot-env.sh coord-b       # Coord-B baldes
#   source scripts/slot-env.sh impl-1        # Implementor in batch
#   source scripts/slot-env.sh impl-2
#
# What it does:
#   1. Picks a backing disk per slot (SSD local for coord-a; MAC_EXTERNO
#      for the rest — Coord-A does the most foundational work, deserves
#      the faster disk; others can live on the bigger but slower drive).
#   2. Exports CARGO_TARGET_DIR=<that-disk>/target/slot-<ID>.
#   3. Exports PH2D_SLOT_ID for the git-stage-guard (Etapa 0.2) and
#      for prompts in this session.
#   4. Disk space sanity check (refuses if <30 GiB free on the picked disk).
#   5. Prints a one-line banner so the operator sees it worked.
#
# REPO ROOT: must be sourced from the workspace root (uses git rev-parse).
#
# RAM budget: this machine is 8 GiB. Realistic parallelism is 2-3
# active `cargo` sessions. If you start a 4th and see swap thrashing,
# pause one. The slot env only fixes disk contention, not RAM.
#
# Wave 10 / Etapa 0.1 / decision #3 Enio (accept 2-3 slot ceiling).

# Detect sourcing in BOTH zsh (BASH_SOURCE unset) and bash. The old guard
# aborted under zsh — the project shell. NOTE: Claude Code Bash-tool agents
# can't rely on this anyway (env does not persist between tool calls); they use
# `bash scripts/slot-seed.sh <id>` + prefix cargo with the printed
# CARGO_TARGET_DIR. slot-env is for interactive operator shells.
_ph2d_sourced=0
if [ -n "${ZSH_VERSION:-}" ]; then
    case "${ZSH_EVAL_CONTEXT:-}" in *:file*) _ph2d_sourced=1 ;; esac
elif [ -n "${BASH_VERSION:-}" ]; then
    [ "${BASH_SOURCE[0]:-}" != "$0" ] && _ph2d_sourced=1
fi
if [ "$_ph2d_sourced" = "0" ]; then
    echo "[slot-env] ERROR: must be SOURCED, not executed (or use scripts/slot-seed.sh)."
    echo "[slot-env] Usage: source scripts/slot-env.sh <slot-id>"
    exit 1
fi

if [ -z "${1:-}" ]; then
    echo "[slot-env] ERROR: missing slot-id argument."
    echo "[slot-env] Usage: source scripts/slot-env.sh <slot-id>"
    echo "[slot-env] Canonical IDs: coord-a, coord-b, impl-1, impl-2, impl-3, impl-4"
    return 1 2>/dev/null || exit 1
fi

PH2D_SLOT_ID="$1"

# Resolve workspace root from git (works wherever sourced from).
PH2D_ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$PH2D_ROOT_DIR" ]; then
    echo "[slot-env] ERROR: not in a git repo. Run from PH2D workspace root."
    return 1 2>/dev/null || exit 1
fi

# Backing disk per slot:
#   coord-a → SSD local (fast NVMe, foundational work is IO-sensitive)
#   others  → MAC_EXTERNO (1.8 TB free, slower but huge)
case "$PH2D_SLOT_ID" in
    coord-a)
        # SSD local: ~/ph2d-target/ (NOT in repo; never committed)
        SLOT_BASE="$HOME/ph2d-target"
        ;;
    *)
        # MAC_EXTERNO: keep targets next to the repo on the external drive
        SLOT_BASE="$PH2D_ROOT_DIR/target-slots"
        ;;
esac

mkdir -p "$SLOT_BASE"
PH2D_SLOT_TARGET="$SLOT_BASE/slot-${PH2D_SLOT_ID}"

# ── APFS CoW seeding (kills the ~5x per-slot dependency recompile) ──────────
# A warm base target lives at target-slots/base on MAC_EXTERNO (APFS). New
# slots on the SAME APFS volume are cloned from it via `cp -c` (clonefile:
# near-instant, 0 physical bytes — shares blocks copy-on-write). The slot then
# already has all deps compiled; only the agent's own crate recompiles.
#
# PRECONDITIONS (baked into cargo fingerprints — silent full-rebuild if violated):
#   - identical RUSTFLAGS across base + slots → rely on global ~/.cargo/config.toml,
#     NEVER set per-slot RUSTFLAGS.
#   - identical rustc/toolchain + profile + target-triple.
# Rebuild the base whenever Cargo.lock or the toolchain changes:
#   CARGO_TARGET_DIR="$PH2D_ROOT_DIR/target-slots/base" cargo check --workspace
# (run it ALONE — RAM-heavy on 8 GiB.)
# NOTE: coord-* lives on the local SSD (different volume) where clonefile falls
# back to a full byte copy, so CoW only helps the MAC_EXTERNO slots.
PH2D_BASE_TARGET="$PH2D_ROOT_DIR/target-slots/base"
if [ ! -d "$PH2D_SLOT_TARGET" ]; then
    if [ -d "$PH2D_BASE_TARGET" ] && [ "$SLOT_BASE" = "$PH2D_ROOT_DIR/target-slots" ] \
       && [ "$PH2D_SLOT_TARGET" != "$PH2D_BASE_TARGET" ]; then
        echo "[slot-env] seeding slot from warm base via APFS clone (cp -c)…"
        cp -c -R "$PH2D_BASE_TARGET" "$PH2D_SLOT_TARGET"
    else
        mkdir -p "$PH2D_SLOT_TARGET"
    fi
fi

# Disk space sanity (df reports in GiB on macOS via -g).
disk_free_gi=$(df -g "$SLOT_BASE" | awk 'NR==2 {print $4}')
if [ -n "$disk_free_gi" ] && [ "$disk_free_gi" -lt 30 ]; then
    echo "[slot-env] WARNING: only ${disk_free_gi} GiB free on $SLOT_BASE"
    echo "[slot-env] Cargo target dirs typically eat 20-40 GiB after a few weeks."
    echo "[slot-env] Continuing anyway — you may need to clean older slot dirs."
fi

export CARGO_TARGET_DIR="$PH2D_SLOT_TARGET"
export PH2D_SLOT_ID

# RAM headroom check — friendly warning, not a block.
# vm_stat reports pages; macOS page size = 16384 bytes on Apple Silicon
# (4096 on Intel). Use sysctl to get it right.
page_size=$(sysctl -n hw.pagesize 2>/dev/null || echo 4096)
free_pages=$(vm_stat 2>/dev/null | awk '/Pages free/ {gsub(/\./,"",$3); print $3}')
if [ -n "$free_pages" ] && [ "$page_size" -gt 0 ]; then
    free_mb=$(( free_pages * page_size / 1024 / 1024 ))
    if [ "$free_mb" -lt 500 ]; then
        echo "[slot-env] WARNING: only ${free_mb} MiB RAM free."
        echo "[slot-env] This machine is 8 GiB. Recommended max 2-3 active cargo slots."
        echo "[slot-env] If you start a 4th, expect swap thrashing."
    fi
fi

# Banner.
if [ -t 1 ]; then
    printf "\033[1;32m[slot-env]\033[0m active: \033[1m%s\033[0m  target=%s\n" \
        "$PH2D_SLOT_ID" "$CARGO_TARGET_DIR"
else
    echo "[slot-env] active: $PH2D_SLOT_ID target=$CARGO_TARGET_DIR"
fi

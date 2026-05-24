#!/usr/bin/env bash
# arch-gate-budget.sh — measures total arch-gate runtime and warns if drift.
#
# Wave 10 / Etapa 0.5: prevents the long-term "gates pile up, hook becomes
# 5min" drift. Runs the architecture/staleness/contract gates and prints
# elapsed time. WARNS (does not fail) if > 90s.
#
# Why warn-only: legitimate gate growth happens (cross-platform GPU init,
# new arch test). Failure mode would break a CI run for a soft signal.
# A console warning + CI log inspection is the right tone.
#
# Usage:
#   ./scripts/arch-gate-budget.sh              # all arch-tests in workspace
#   ./scripts/arch-gate-budget.sh -p <crate>   # one crate only
#
# Output:
#   Per-test wall time (sorted), total elapsed, threshold check.

set -uo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

if [ -t 1 ]; then
    G="\033[1;32m"; R="\033[1;31m"; Y="\033[1;33m"; B="\033[1;34m"; N="\033[0m"
else
    G=""; R=""; Y=""; B=""; N=""
fi

# Budget threshold (seconds). Bump only after intentional cost (new platform,
# bigger fixture). Cap stays apertado as a tripwire.
BUDGET_SECONDS=${PH2D_ARCH_GATE_BUDGET:-90}

# Filter for arch-test names (heuristic — matches our gate naming convention).
ARCH_TEST_PATTERN="architecture_|staleness|cycle_prevention|contract_surface|widget_loc_cap|widget_showcase|panel_chip_pill|no_literal_color|no_magic_numeric|no_tofu_glyphs|hr12_|hr15_|mockup_tokens_exist|tool_manifest_design_sync|node_id_collisions|grid_toggle|make_square_algorithm|canonical_icon_button|register_all_alphabetical|file_loc_caps|interaction_dispatch_no_alloc"

note() { printf "${B}[arch-gate-budget]${N} %s\n" "$1"; }
warn() { printf "${Y}[arch-gate-budget WARN]${N} %s\n" "$1" >&2; }
ok()   { printf "${G}[arch-gate-budget]${N} %s\n" "$1"; }

note "running arch-test subset (pattern: $ARCH_TEST_PATTERN)"
note "budget: ${BUDGET_SECONDS}s (override via PH2D_ARCH_GATE_BUDGET)"

START=$(date +%s)

if command -v cargo-nextest >/dev/null 2>&1; then
    # nextest: -E <expr> filters by test name; --no-fail-fast keeps measuring
    # even if a test fails (we measure time, not correctness).
    cargo nextest run "$@" -E "test(/${ARCH_TEST_PATTERN}/)" --no-fail-fast || true
else
    # Fallback: cargo test filter is substring, not regex — coarse but works
    cargo test "$@" --tests architecture -- --test-threads=1 || true
    cargo test "$@" --tests staleness -- --test-threads=1 || true
fi

END=$(date +%s)
ELAPSED=$((END - START))

printf "\n"
note "elapsed: ${ELAPSED}s"

if [ "$ELAPSED" -gt "$BUDGET_SECONDS" ]; then
    warn "arch-gates took ${ELAPSED}s — exceeds ${BUDGET_SECONDS}s budget."
    warn "Investigate which gate is slow. If legitimate, bump PH2D_ARCH_GATE_BUDGET."
    warn "(non-fatal: this is a tripwire, not a CI gate. See DIRETRIZ §0.5.)"
    exit 0  # warn-only
fi

ok "arch-gates within ${BUDGET_SECONDS}s budget (${ELAPSED}s)."

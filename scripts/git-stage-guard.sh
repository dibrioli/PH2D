#!/usr/bin/env bash
# git-stage-guard.sh — pre-commit anti-collision guard for parallel agents.
#
# Detects two anti-patterns that have burned this project (vide memory
# entries feedback_parallel_agent_collision and feedback_scoped_commit_shared_index):
#
#   1. Staging files outside the slot's declared exclusive folder
#      (set via PH2D_SLOT_FOLDER env in the session — see CLAUDE.md).
#   2. Using `git add -A` / `git add -a` / `git add .` (history check).
#
# Bypass: export COORD_OVERRIDE=1 in the shell session (Coord roles
# legitimately touch foundational code outside any single folder).
#
# Hooked into scripts/pre-commit.sh — runs FIRST (before tier classification),
# fails fast with a clear message before any cargo invocation.
#
# Wave 10 / Etapa 0.2 / decision #5 Enio (2 Coords with COORD_OVERRIDE bypass).

set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

if [ -t 1 ]; then
    G="\033[1;32m"; R="\033[1;31m"; Y="\033[1;33m"; N="\033[0m"
else
    G=""; R=""; Y=""; N=""
fi

die() { printf "${R}[git-stage-guard FAIL]${N} %s\n" "$1" >&2; exit 1; }
note() { printf "${Y}[git-stage-guard]${N} %s\n" "$1"; }
ok() { printf "${G}[git-stage-guard]${N} %s\n" "$1"; }

# ── Coord override bypass ──────────────────────────────────────────────────
if [ "${COORD_OVERRIDE:-0}" = "1" ]; then
    note "COORD_OVERRIDE=1 — bypass active (foundational work allowed)."
    exit 0
fi

# ── Detect `git add -A/-a/.` in recent shell history ──────────────────────
# This is best-effort: the hook runs INSIDE the commit, so $HISTFILE may or
# may not have the most recent command. We check the last 20 history lines.
# Detection is informational — we do NOT block on history (too brittle);
# we DO block on staged-paths checks below.
if [ -n "${HISTFILE:-}" ] && [ -r "$HISTFILE" ]; then
    if tail -20 "$HISTFILE" 2>/dev/null | grep -qE '^[[:space:]]*git[[:space:]]+add[[:space:]]+(-A|-a|\.)'; then
        note "history shows recent 'git add -A/-a/.' — verify staged paths below are intentional."
    fi
fi

# ── Declared slot folder (optional, opt-in per session) ────────────────────
# Format: PH2D_SLOT_FOLDER="crates/ph2d-tool-bgremoval"
# (or comma-separated for multi-crate Coord work, but in that case
#  COORD_OVERRIDE=1 is the cleaner answer.)
SLOT_FOLDER="${PH2D_SLOT_FOLDER:-}"

# ── Collect staged paths ──────────────────────────────────────────────────
STAGED=$(git diff --cached --name-only --diff-filter=ACMR)
if [ -z "$STAGED" ]; then
    # nothing staged — defer to next hook (or empty commit will be rejected by git itself)
    exit 0
fi

# ── If no slot folder declared, skip the folder check ─────────────────────
# (back-compat: pre-Wave-10 sessions had no PH2D_SLOT_FOLDER set.)
if [ -z "$SLOT_FOLDER" ]; then
    note "PH2D_SLOT_FOLDER not set — folder check skipped. Set it via 'export PH2D_SLOT_FOLDER=crates/ph2d-tool-<slug>' for scoped sessions, or COORD_OVERRIDE=1 for Coord work."
    exit 0
fi

# ── Path classification ────────────────────────────────────────────────────
# Allow paths that:
#   a) start with $SLOT_FOLDER (the declared exclusive folder)
#   b) are scripts/, docs/plans/, docs/HANDOFF*, MEMORY edits (collab artifacts)
#   c) are the codegen-touched files for tool/node-sync (DIRETRIZ §1.4 note)
#      — i.e., crates/ph2d-{node,tool,panel}-registry-init/ and Cargo.lock
violations=""
while IFS= read -r p; do
    [ -z "$p" ] && continue
    case "$p" in
        "$SLOT_FOLDER"|"$SLOT_FOLDER"/*) continue ;;
        # Codegen artifacts — valid for tool/node-sync regens
        crates/ph2d-tool-registry-init/*|crates/ph2d-node-registry-init/*|crates/ph2d-panel-registry-init/*|Cargo.lock) continue ;;
        # Collab artifacts
        docs/plans/*|docs/HANDOFF*|docs/SESSION_ACTIVE.md) continue ;;
        # Hard-block everything else
        *) violations="${violations}  - ${p}\n" ;;
    esac
done <<< "$STAGED"

if [ -n "$violations" ]; then
    printf "${R}[git-stage-guard FAIL]${N} staged paths outside slot folder %s:\n" "$SLOT_FOLDER" >&2
    printf "%b" "$violations" >&2
    printf "\n  Options:\n" >&2
    printf "    1. Unstage them: git restore --staged <path>\n" >&2
    printf "    2. If this is Coord work, run: export COORD_OVERRIDE=1\n" >&2
    printf "    3. If this is legit codegen, add the path to the allowlist in scripts/git-stage-guard.sh\n" >&2
    exit 1
fi

ok "all staged paths inside slot folder ($SLOT_FOLDER) or allowlisted."

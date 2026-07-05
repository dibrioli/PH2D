#!/usr/bin/env bash
# foundational-integrate.sh — Modo L tested integration gate (ADR-0107, Camada 2).
#
# `git merge --ff-only` proves "nobody landed between my rebase and my merge";
# it does NOT prove the COMBINED tree compiles. For a drop-crate line that's
# fine (physical isolation guarantees it). For a line that touched foundational
# it is exactly where a silent semantic/build conflict hides (agent A changes a
# signature, agent B adds a caller — each compiles alone, together they break).
#
# This script closes that hole: rebase → re-sync → BUILD+TEST the rebased tip
# (== the future main) → only then ff-merge. Because --ff-only makes the tip
# BECOME main, a green rebased tip is a green main. If another line lands in the
# window, the ff-merge fails → rerun (rebases onto it, re-tests). No line ever
# merges an untested combination. (This is the Zuul/Bors gate, one slot.)
#
# Run from INSIDE your line worktree, module gate already batched-green.
# Usage:  bash scripts/foundational-integrate.sh
set -euo pipefail

# ---------- preconditions ----------
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
PRIMARY="$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
HERE="$(git rev-parse --show-toplevel)"

if [[ "$HERE" == "$PRIMARY" ]]; then
    echo "✗ You are in the PRIMARY checkout, not a line worktree." >&2
    echo "  Integrate FROM a line worktree (Worktrees/line-<módulo>/)." >&2
    exit 1
fi
if [[ "$BRANCH" != line/* ]]; then
    echo "✗ HEAD is '$BRANCH', not a line/* branch — refusing to integrate." >&2
    exit 1
fi
# Primary must be clean and on main (--ff-only lands there).
if [[ -n "$(git -C "$PRIMARY" status --porcelain)" ]]; then
    echo "✗ Primary checkout ($PRIMARY) is dirty — refusing." >&2
    echo "  A dirty primary means someone is coding in main (DIRETRIZ §1.5.1 violation)." >&2
    exit 1
fi
if [[ "$(git -C "$PRIMARY" symbolic-ref --short HEAD)" != "main" ]]; then
    echo "✗ Primary checkout is not on 'main' — refusing." >&2
    exit 1
fi
# Your own work must be committed before we rebase.
if [[ -n "$(git status --porcelain)" ]]; then
    echo "✗ This worktree has uncommitted changes — commit them first (§1.5.2.2)." >&2
    exit 1
fi

echo "▸ Integrating $BRANCH → main (primary: $PRIMARY)"

# ---------- 1. rebase onto the current main ----------
echo "▸ [1/5] git rebase main"
if ! git rebase main; then
    cat >&2 <<'EOF'
✗ rebase hit a conflict. The ONLY legitimate conflicts (DIRETRIZ §1.5.5):
    • Cargo.lock            → NEVER by hand: `git checkout main -- Cargo.lock`
                              then `cargo check -p <sua-crate>`, `git add Cargo.lock`
    • *-registry-init/      → NEVER by hand: accept either side, re-run the sync
    • icons.rs (IconId)     → keep BOTH variants, alphabetical
A conflict in code OUTSIDE your module's files means an isolation violation
(concurrent same-symbol edit in foundational) — STOP and report (ADR-0107).
Resolve, `git rebase --continue`, then re-run this script.
EOF
    exit 1
fi

# ---------- 2. re-sync codegen, auto-commit ONLY generated targets ----------
echo "▸ [2/5] re-sync codegen (tool/node registries)"
cargo run -q -p ph2d-tool-sync
cargo run -q -p ph2d-node-sync
GEN_GLOBS=('crates/ph2d-tool-registry-init' 'crates/ph2d-node-registry-init'
           'crates/ph2d-panel-registry-init' 'crates/ph2d-editor-core/src/icons.rs'
           'Cargo.lock')
git add -- "${GEN_GLOBS[@]}" 2>/dev/null || true
# Anything dirty OUTSIDE the generated targets after a sync is unexpected.
if [[ -n "$(git status --porcelain --untracked-files=no | grep -v '^[MA] ' || true)" ]]; then
    echo "✗ Post-sync working tree dirty outside generated targets — inspect manually." >&2
    git status --short >&2
    exit 1
fi
if [[ -n "$(git diff --cached --name-only)" ]]; then
    git commit --no-verify -q -m "chore(sync): regenerate registries post-rebase"
    echo "    committed registry regen"
fi

# ---------- 3. staleness gate (generated surfaces match the crate glob) ----------
echo "▸ [3/5] registry staleness"
cargo test -q -p ph2d-tool-registry-init -p ph2d-node-registry-init

# ---------- 4. combined-tree BUILD gate (the ADR-0107 core) ----------
# Foundational touched anywhere? → the whole workspace must still compile.
# Only self-isolated drop-crates (crates/ph2d-{tool,node,panel}-<slug>/) + docs/
# are exempt (physical isolation already guarantees them).
CHANGED="$(git diff --name-only main...HEAD)"
FOUNDATIONAL_TOUCH="$(echo "$CHANGED" \
    | grep -vE '^(crates/ph2d-(tool|node|panel)-[a-z0-9_-]+/|docs/)' || true)"
echo "▸ [4/5] combined-tree build gate"
if [[ -n "$FOUNDATIONAL_TOUCH" ]]; then
    echo "    foundational touched → cargo check --workspace"
    echo "$FOUNDATIONAL_TOUCH" | sed 's/^/      · /'
    cargo check --workspace
else
    echo "    drop-crate only → isolated; checking changed crates"
    CRATES="$(echo "$CHANGED" | sed -n 's#^crates/\([^/]*\)/.*#-p \1#p' | sort -u | tr '\n' ' ')"
    # shellcheck disable=SC2086
    [[ -n "$CRATES" ]] && cargo check $CRATES || echo "    (no crate changes to check)"
fi

# ---------- 5. impacted tests, then land ----------
echo "▸ [5/5] impacted tests"
BASE=main bash scripts/nextest-impacted.sh

echo "▸ merge --ff-only into main"
if git -C "$PRIMARY" merge --ff-only "$BRANCH"; then
    echo "✓ $BRANCH integrated into main. Green combined tree landed."
    echo "  (If you close the LAST integration of the journey: ship.sh + push + babysit CI — §1.5.4)"
else
    echo "✗ --ff-only failed: another line landed after your rebase." >&2
    echo "  That is the serialization working — just re-run this script (rebase+retest)." >&2
    exit 1
fi

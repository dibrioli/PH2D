#!/usr/bin/env bash
# Wave 10 / Etapa 7.1 — auto-merge eligibility check.
#
# Exits 0 if a PR's diff is auto-merge-eligible per the Wave 10 policy:
#   (i)   diff only in `crates/ph2d-{node,tool,panel}-<slug>/` directories
#         (drop-crate fan-out per ADR-0040 / ADR-0039 / DIRETRIZ §3.8),
#   (ii)  no foundational / contract-frozen file touched,
#   (iii) at most one panel/tool/node crate added or modified at a time
#         (multi-crate work goes through Coord review).
#
# Exits non-zero ⇒ Coord review required.
#
# Usage:
#   scripts/auto-merge-eligibility.sh <base-ref> [<head-ref>]
#   scripts/auto-merge-eligibility.sh main           # diff vs main..HEAD
#   scripts/auto-merge-eligibility.sh main feat/x    # diff vs main..feat/x
#
# Fail-safe: ANY ambiguity defaults to coord-review (exit 1).

set -euo pipefail

BASE_REF="${1:?usage: auto-merge-eligibility.sh <base-ref> [<head-ref>]}"
HEAD_REF="${2:-HEAD}"

# Verify both refs resolve.
git rev-parse --verify "$BASE_REF^{commit}" >/dev/null 2>&1 || {
    echo "auto-merge: base ref '$BASE_REF' not found — coord-review required" >&2
    exit 1
}
git rev-parse --verify "$HEAD_REF^{commit}" >/dev/null 2>&1 || {
    echo "auto-merge: head ref '$HEAD_REF' not found — coord-review required" >&2
    exit 1
}

CHANGED=$(git diff --name-only "$BASE_REF...$HEAD_REF")

if [[ -z "$CHANGED" ]]; then
    echo "auto-merge: empty diff — nothing to merge" >&2
    exit 1
fi

# ---------- (ii) foundational / frozen-contract guard ----------
# These paths must always trigger Coord review. Listed explicitly so
# changes here are conscious (and reviewable).
FOUNDATIONAL_PATTERNS=(
    "^Cargo\.toml$"                                          # workspace
    "^Cargo\.lock$"                                          # dep tree
    "^scripts/"                                              # build infra
    "^\.github/"                                             # CI
    "^docs/architecture/decisions/"                          # ADRs
    "^docs/IntegracaoMultiAgente/"                           # protocol
    "^CLAUDE\.md$"                                           # instructions
    "^SKILL_"                                                # canonical doc
    "^shells/"                                               # desktop shell
    "^crates/ph2d-editor-core/"                              # core orchestrator
    "^crates/ph2d-tool-runtime/"                             # contract gateway
    "^crates/ph2d-tool-registry/"                            # tool dispatch
    "^crates/ph2d-tool-registry-init/"                       # codegen target
    "^crates/ph2d-panel-registry-init/"                      # codegen target
    "^crates/ph2d-nodegraph/"                                # node substrate
    "^crates/ph2d-expr/"                                     # shared compute
    "^crates/ph2d-color/"                                    # color substrate
    "^crates/ph2d-tokens/"                                   # design tokens
    "^crates/ph2d-render/"                                   # renderer
    "^crates/ph2d-gpu/"                                      # GPU layer
    "^crates/ph2d-text/"                                     # text shaping
    "^crates/ph2d-a11y/"                                     # accessibility
    "^crates/ph2d-host/"                                     # host abstraction
    "^crates/ph2d-asset-cooker/"                             # asset pipeline
    "^crates/ph2d-vector/"                                   # vector scene
    "^tools/"                                                # codegen tools
)

while IFS= read -r file; do
    for pattern in "${FOUNDATIONAL_PATTERNS[@]}"; do
        if [[ "$file" =~ $pattern ]]; then
            echo "auto-merge: foundational path touched ($file) — coord-review required" >&2
            exit 1
        fi
    done
done <<< "$CHANGED"

# ---------- (i) only drop-crate paths ----------
# Allowed: crates/ph2d-{node,tool,panel}-<slug>/** AND docs/Testes/**
# (smoke notes from the implementer).
ALLOWED_PATTERN='^(crates/ph2d-(node|tool|panel)-[a-z0-9_-]+/|docs/Testes/)'
while IFS= read -r file; do
    if [[ ! "$file" =~ $ALLOWED_PATTERN ]]; then
        echo "auto-merge: file outside drop-crate scope ($file) — coord-review required" >&2
        exit 1
    fi
done <<< "$CHANGED"

# ---------- (iii) at most one drop-crate ----------
# Extract the crate directory for every diff entry; collapse to unique.
CRATES=$(echo "$CHANGED" \
    | grep -E '^crates/ph2d-(node|tool|panel)-[a-z0-9_-]+/' \
    | sed -E 's|^(crates/ph2d-(node|tool|panel)-[a-z0-9_-]+)/.*$|\1|' \
    | sort -u)
N_CRATES=$(echo "$CRATES" | grep -c .)
if [[ "$N_CRATES" -gt 1 ]]; then
    echo "auto-merge: multi-crate diff ($N_CRATES crates) — coord-review required" >&2
    echo "$CRATES" | sed 's/^/  - /' >&2
    exit 1
fi

echo "auto-merge: ELIGIBLE — single drop-crate scope, no foundational touch."
echo "  crate: $CRATES"
exit 0

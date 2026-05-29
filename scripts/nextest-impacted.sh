#!/usr/bin/env bash
# nextest-impacted.sh — run nextest ONLY for crates changed vs BASE plus their
# reverse-dependents, ALWAYS including the HR-5 determinism golden. For the fast
# module-close loop. NOT a replacement for the final gate:
#   ./scripts/ship.sh (cargo nextest run --workspace --cargo-profile ci-test)
# stays the unconditional pre-push gate, and the CI cross-OS determinism job
# (spike.yml `determinism`/`determinism-compare`) cannot be reproduced locally.
#
# Usage:  ./scripts/nextest-impacted.sh            # vs origin/main
#         BASE=<ref> ./scripts/nextest-impacted.sh # vs custom ref
set -uo pipefail

BASE="${BASE:-origin/main}"

# Changed crate DIR names under crates/ (dir == package name by ph2d-* convention).
CHANGED=$(git diff --name-only "${BASE}"... 2>/dev/null \
  | sed -n 's#^crates/\([^/]*\)/.*#\1#p' | sort -u)

if [ -z "$CHANGED" ]; then
    echo "[nextest-impacted] no crate changes vs ${BASE}; running determinism golden only."
    exec cargo nextest run -E 'test(/transform_determinism/)' --cargo-profile ci-test
fi

# rdeps(set) = the crate AND everything that depends on it (the real "impacted"
# set). `-p X` does NOT include dependents — filterset rdeps() does.
EXPR=""
for c in $CHANGED; do
    EXPR="${EXPR:+$EXPR + }rdeps(${c})"
done

# MANDATORY determinism net: force the HR-5 golden so a transitive math/dep edit
# can never silently skip it locally.
EXPR="$EXPR + test(/transform_determinism/)"

echo "[nextest-impacted] changed: $(echo "$CHANGED" | tr '\n' ' ')"
echo "[nextest-impacted] -E '$EXPR'"
# If a changed dir name is not a real package, the filterset errors out (non-zero)
# — that surfaces the dir→package mismatch rather than silently under-testing.
exec cargo nextest run -E "$EXPR" --cargo-profile ci-test

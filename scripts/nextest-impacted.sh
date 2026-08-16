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

# `ci-test` só roda em BATCH (uma ou duas vezes por jornada, sobre a workspace
# inteira), então compilação incremental não colhe nada ali e paga 11 GB — o
# número que o CLAUDE.md §2 nomeia, MEDIDO em 2026-08-16. Esta é a única coisa
# que este script executa, então o `export` é seguro: o `cargo check -p` do
# inner loop (perfil `dev`, outro processo) fica em paz, de propósito.
#
# ⚠️ A regra vivia SÓ no CLAUDE.md, e o integrador de 16/08 a seguiu à mão e
# viu os 11 GB aparecerem na mesma — porque quem compila é ESTE script. Uma
# regra fora do caminho de quem a executa é uma regra que não existe.
# ⚠️ E nesta workstation o `target/` é symlink para /dev/shm: o diretório de
# build É RAM, e o número sai do mesmo orçamento em que o app corre.
export CARGO_INCREMENTAL=0

BASE="${BASE:-origin/main}"

# Changed crate DIR names under crates/ (dir == package name by ph2d-* convention).
CHANGED=$(git diff --name-only "${BASE}"... 2>/dev/null \
  | sed -n 's#^crates/\([^/]*\)/.*#\1#p' | sort -u)

if [ -z "$CHANGED" ]; then
    echo "[nextest-impacted] no crate changes vs ${BASE}; running determinism golden only."
    # `binary(...)` matches the TEST BINARY name (tests/transform_determinism.rs);
    # `test(...)` matches individual fn NAMES (cross_os_golden_hash_pinned, …) which
    # do NOT contain "transform_determinism" → it silently matched 0 (false-green).
    exec cargo nextest run -E 'binary(transform_determinism)' --cargo-profile ci-test
fi

# rdeps(set) = the crate AND everything that depends on it (the real "impacted"
# set). `-p X` does NOT include dependents — filterset rdeps() does.
EXPR=""
for c in $CHANGED; do
    EXPR="${EXPR:+$EXPR + }rdeps(${c})"
done

# MANDATORY determinism net: force the HR-5 golden so a transitive math/dep edit
# can never silently skip it locally. `binary(...)` (NOT `test(...)`) — the golden
# lives in the `transform_determinism` test binary; its fn names don't contain that
# string, so `test(/transform_determinism/)` matched 0 and gave a false-green gate.
EXPR="$EXPR + binary(transform_determinism)"

echo "[nextest-impacted] changed: $(echo "$CHANGED" | tr '\n' ' ')"
echo "[nextest-impacted] -E '$EXPR'"
# If a changed dir name is not a real package, the filterset errors out (non-zero)
# — that surfaces the dir→package mismatch rather than silently under-testing.
exec cargo nextest run -E "$EXPR" --cargo-profile ci-test

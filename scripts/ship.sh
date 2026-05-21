#!/usr/bin/env bash
# ship.sh — local "is this CI-clean?" gate. Run ONCE before a push (or at
# the end-of-day "ship"), so the GitHub `spike.yml` lint + test jobs go
# GREEN on the first try.
#
# WHY THIS EXISTS: the local pre-commit hook (scripts/pre-commit.sh) does
# NOT match CI. The 2026-05-19 perf audit dropped `clippy --all-targets`
# from the T2 workspace tier, and the hook never ran `cargo machete` /
# `cargo deny` / `cargo audit`. So a change can pass every local commit
# and still redden CI (test-code clippy lints, unused deps, ...). This
# script runs the EXACT CI commands so there are no surprises at push.
# See docs/IntegracaoMultiAgente/DIRETRIZ.md §8 (fast mode / ship).
#
# Runs EVERY check (does not stop at the first failure), prints a summary,
# and exits non-zero if any failed. It does NOT commit or push — the agent
# does that AFTER this is green (then babysits CI; DIRETRIZ §8).

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

fail=0
results=()

run() { # run "<label>" <cmd...>
    local label="$1"
    shift
    printf '\n\033[1m▶ %s\033[0m\n' "$label"
    if "$@"; then
        results+=("✓ $label")
    else
        results+=("✗ $label")
        fail=1
    fi
}

run_optional() { # run_optional "<label>" <tool-binary> <cmd...>
    local label="$1" tool="$2"
    shift 2
    if command -v "$tool" >/dev/null 2>&1; then
        run "$label" "$@"
    else
        results+=("⚠ $label — SKIPPED ($tool not installed; CI runs it · cargo install $tool)")
    fi
}

echo "ship.sh — mirroring spike.yml lint + test jobs (this compiles a lot; ~minutes)"

# ── CI `lint` job parity (spike.yml) ────────────────────────────────────
run "fmt --check" cargo fmt --all -- --check
run "clippy (workspace, all-targets, CI features)" \
    cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
run_optional "cargo-machete (unused deps)" cargo-machete cargo machete
run_optional "cargo-deny (licenses/advisories/bans/sources)" cargo-deny \
    cargo deny --all-features check
run_optional "cargo-audit (CVE scan)" cargo-audit cargo audit
# typos: CI's lint job runs a project-wide scan (config in .typos.toml).
# The pre-commit hook DOES run it, but a `--no-verify` fast-mode session
# bypasses the hook — so without this row a stray typo lands in CI lint
# red after the push. Same engine + config as spike.yml.
run_optional "typos (project-wide typo scan)" typos typos

# ── CI `test` job parity (nextest covers arch gates + cook-hash) ─────────
if command -v cargo-nextest >/dev/null 2>&1; then
    run "nextest run --workspace" cargo nextest run --workspace
else
    run "cargo test --workspace" cargo test --workspace
fi

# ── summary ─────────────────────────────────────────────────────────────
printf '\n\033[1m── ship.sh summary ──\033[0m\n'
for r in "${results[@]}"; do printf '  %s\n' "$r"; done
if [ "$fail" -ne 0 ]; then
    printf '\n\033[1;31m✗ NOT CI-clean — fix the ✗ rows above, re-run, before pushing.\033[0m\n'
    exit 1
fi
printf '\n\033[1;32m✓ CI-clean. Safe to commit + push (then babysit CI).\033[0m\n'

#!/usr/bin/env bash
# Pre-commit gate — workspace-wide validation before each commit.
#
# The everyday inner loop uses `-p <crate>` (5–10× faster), so this
# hook is the safety net that catches cross-crate breakage before it
# reaches the remote.
#
# Runs the same gate CI does, in the same order, but locally:
#   1. cargo fmt --check    (fast; fails first on whitespace)
#   2. cargo clippy --workspace --all-targets -- -D warnings
#   3. cargo nextest run --workspace
#   4. cargo test --doc --workspace   (nextest skips doctests)
#
# Pass `--no-verify` to git commit to bypass (use sparingly).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

# Color helpers — quiet when stdout isn't a TTY (git hook usually is).
if [ -t 1 ]; then
    G="\033[1;32m"; R="\033[1;31m"; Y="\033[1;33m"; N="\033[0m"
else
    G=""; R=""; Y=""; N=""
fi

step() { printf "${Y}[pre-commit]${N} %s\n" "$1"; }
ok()   { printf "${G}[pre-commit]${N} %s\n" "$1"; }
die()  { printf "${R}[pre-commit FAIL]${N} %s\n" "$1" >&2; exit 1; }

step "1/4 cargo fmt --check"
cargo fmt --all -- --check || die "fmt check failed — run 'cargo fmt --all'"

step "2/4 cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings || die "clippy found issues"

step "3/4 cargo nextest run --workspace"
if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run --workspace || die "nextest failed"
else
    step "    (nextest not installed — falling back to cargo test)"
    cargo test --workspace --lib --bins --tests || die "tests failed"
fi

step "4/4 cargo test --doc --workspace"
cargo test --doc --workspace || die "doc tests failed"

ok "all checks passed"

#!/usr/bin/env bash
# Pre-commit gate — TIERED. Speed-tunes validation to scope of changes.
#
# Tiers (auto-selected from `git diff --cached`):
#   T0 (~5s)   — docs / config-only: fmt + typos on staged paths, no
#                rust compile.
#   T1 (~30s)  — single crate: cargo check/clippy/nextest -p <crate>
#                + fmt + typos.
#   T2 (~3-5m) — workspace gate: full fmt + clippy + nextest +
#                doc-tests + typos. Used when staged set touches
#                Cargo.{toml,lock}, shells/desktop, multiple crates,
#                or workspace-affecting files.
#
# Bypass: `git commit --no-verify` skips everything. Use after manual
# per-crate validation, especially for small isolated edits.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

if [ -t 1 ]; then
    G="\033[1;32m"; R="\033[1;31m"; Y="\033[1;33m"; B="\033[1;34m"; N="\033[0m"
else
    G=""; R=""; Y=""; B=""; N=""
fi
step() { printf "${Y}[pre-commit]${N} %s\n" "$1"; }
note() { printf "${B}[pre-commit]${N} %s\n" "$1"; }
ok()   { printf "${G}[pre-commit]${N} %s\n" "$1"; }
die()  { printf "${R}[pre-commit FAIL]${N} %s\n" "$1" >&2; exit 1; }

# ── Collect staged paths ──────────────────────────────────────────────────
STAGED=$(git diff --cached --name-only --diff-filter=ACMR)
if [ -z "$STAGED" ]; then
    note "no staged changes — nothing to validate."
    exit 0
fi

# ── Classify each path ────────────────────────────────────────────────────
# Categories:
#   workspace_trigger — forces T2 (Cargo.toml/Lock, shells/desktop,
#                       .cargo, rust-toolchain, clippy/deny config)
#   crate_dirs        — collected set of `crates/<X>` / `tools/<X>` /
#                       `tests/spike` / `shells/<X>` touched
#   docs_only_paths   — paths that don't trigger compile (docs, .md,
#                       .github/workflows, scripts/, MEMORY etc.)
#   needs_typos       — any path under typos's scan scope

has_workspace_trigger=0
crate_dirs_set=""
all_docs_only=1

is_workspace_trigger() {
    case "$1" in
        Cargo.toml|Cargo.lock|rust-toolchain.toml|clippy.toml|deny.toml|.cargo/config.toml)
            return 0;;
        shells/desktop/*)
            # Host shell depends on everything; safer to run full workspace.
            return 0;;
        crates/ph2d-core/*|crates/ph2d-ecs/*|crates/ph2d-host/*|crates/ph2d-tokens/*|crates/ph2d-a11y/*)
            # Foundational crates: changes here ripple to many downstreams,
            # workspace gate catches cross-crate regressions.
            return 0;;
    esac
    return 1
}

classify_path() {
    local p="$1"
    if is_workspace_trigger "$p"; then
        has_workspace_trigger=1
        all_docs_only=0
        return
    fi
    case "$p" in
        # Single-crate buckets — record the crate dir
        crates/*/*|tools/*/*|tests/spike/*|shells/*/*)
            local dir
            dir=$(echo "$p" | cut -d/ -f1-2)
            case "$crate_dirs_set" in
                *"|$dir|"*) ;;
                *) crate_dirs_set="${crate_dirs_set}|$dir|";;
            esac
            all_docs_only=0
            ;;
        # Docs / config that don't compile rust
        docs/*|*.md|.gitignore|.editorconfig|.github/*|scripts/*|.typos.toml)
            ;;
        # Anything else outside these — be conservative, full workspace
        *)
            has_workspace_trigger=1
            all_docs_only=0
            ;;
    esac
}

while IFS= read -r path; do
    [ -z "$path" ] && continue
    classify_path "$path"
done <<< "$STAGED"

# Count distinct crate dirs. `set -euo pipefail` requires we avoid
# `grep -v '^$'` here — `grep` exits 1 when no matches and that kills
# the pipeline. Use awk to skip blanks instead (always exits 0).
crate_count=$(echo "$crate_dirs_set" | tr '|' '\n' | awk 'NF' | sort -u | wc -l | tr -d ' ')

# Decide tier
if [ "$has_workspace_trigger" = "1" ] || [ "$crate_count" -gt 1 ]; then
    tier="T2"
elif [ "$crate_count" -eq 1 ]; then
    tier="T1"
else
    tier="T0"
fi

# Resolve the single crate dir → package name (last path segment).
single_crate=""
if [ "$tier" = "T1" ]; then
    single_crate_dir=$(echo "$crate_dirs_set" | tr '|' '\n' | awk 'NF' | head -1)
    single_crate=$(basename "$single_crate_dir")
fi

note "tier=$tier  crates_touched=$crate_count  staged=$(echo "$STAGED" | wc -l | tr -d ' ') files"

# ── T0: fmt + typos only ─────────────────────────────────────────────────
run_fmt_typos() {
    step "fmt check (staged paths)"
    # cargo fmt has no per-file mode; fmt-check the workspace is fast
    # (no compile, just rustfmt parse).
    cargo fmt --all -- --check || die "fmt check failed — run 'cargo fmt --all'"

    if command -v typos >/dev/null 2>&1; then
        step "typos (full project — respects .typos.toml excludes)"
        typos || die "typos found issues"
    else
        note "typos CLI not installed; skipping (CI will catch it)"
    fi
}

if [ "$tier" = "T0" ]; then
    run_fmt_typos
    ok "T0 done — docs/config-only commit, no rust compile needed."
    exit 0
fi

# ── T1: single crate ─────────────────────────────────────────────────────
if [ "$tier" = "T1" ]; then
    run_fmt_typos
    step "cargo check -p $single_crate"
    cargo check -p "$single_crate" || die "cargo check failed"
    step "cargo clippy -p $single_crate --all-targets -- -D warnings"
    cargo clippy -p "$single_crate" --all-targets -- -D warnings || die "clippy failed"
    if command -v cargo-nextest >/dev/null 2>&1; then
        step "cargo nextest run -p $single_crate"
        # --no-tests=warn: stub crates (i18n, audio etc.) have no tests
        # yet; that shouldn't fail the hook. Real failures still fail.
        cargo nextest run -p "$single_crate" --no-tests=warn || die "nextest failed"
    else
        step "cargo test -p $single_crate (nextest not installed)"
        cargo test -p "$single_crate" || die "tests failed"
    fi
    ok "T1 done — single-crate commit, workspace gate skipped."
    exit 0
fi

# ── T2: workspace ────────────────────────────────────────────────────────
run_fmt_typos
step "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings || die "clippy found issues"
if command -v cargo-nextest >/dev/null 2>&1; then
    step "cargo nextest run --workspace"
    cargo nextest run --workspace || die "nextest failed"
else
    step "cargo test --workspace --lib --bins --tests (nextest not installed)"
    cargo test --workspace --lib --bins --tests || die "tests failed"
fi
step "cargo test --doc --workspace"
cargo test --doc --workspace || die "doc tests failed"
ok "T2 done — workspace gate passed."

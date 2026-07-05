#!/usr/bin/env bash
# mergiraf-setup.sh — register Mergiraf as this repo's syntax-aware merge
# driver (ADR-0107, Camada 1). Run ONCE per machine.
#
# WHAT: writes the `[merge "mergiraf"]` driver into the repo-local git config
# (`.git/config`, the common dir shared by every `git worktree`), so all lines
# inherit it. The `merge=mergiraf` attributes themselves live in the versioned
# `.gitattributes` (already committed) — this script only wires the driver that
# those attributes name.
#
# SAFE / IDEMPOTENT: re-running just rewrites the same two keys. A machine that
# never runs this still works — git falls back to its built-in 3-way merge when
# `merge=mergiraf` names an unregistered driver.
#
# Usage:  bash scripts/mergiraf-setup.sh
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# ---------- 1. Mergiraf present? ----------
if ! command -v mergiraf >/dev/null 2>&1; then
    cat >&2 <<'EOF'
✗ mergiraf not found on PATH.

Install it (pick one):
    cargo install mergiraf                       # from crates.io (Rust toolchain)
    # or a prebuilt binary from the releases page:
    #   https://codeberg.org/mergiraf/mergiraf/releases

Then re-run:  bash scripts/mergiraf-setup.sh

(Skipping is safe — git uses its built-in 3-way merge until this is set up —
 but concurrent foundational lines then resolve trivial textual conflicts by
 hand instead of via the AST. See ADR-0107.)
EOF
    exit 1
fi

MRG_VERSION="$(mergiraf --version 2>/dev/null || echo 'mergiraf (version unknown)')"

# ---------- 2. git new enough for the label placeholders? ----------
# `-s %S -x %X -y %Y` pass the revision NAMES for nicer conflict markers; they
# need git >= 2.44. Below that we register the reduced driver (still correct,
# just plainer markers).
GIT_VER="$(git --version | awk '{print $3}')"
ver_ge() { [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -1)" = "$2" ]; }

if ver_ge "$GIT_VER" "2.44.0"; then
    DRIVER='mergiraf merge --git %O %A %B -s %S -x %X -y %Y -p %P -l %L'
else
    echo "! git $GIT_VER < 2.44 — registering reduced driver (plainer conflict markers)." >&2
    DRIVER='mergiraf merge --git %O %A %B -p %P -l %L'
fi

# ---------- 3. register the driver in the shared repo config ----------
# `git config` (local) writes to the common `.git/config`, which every worktree
# shares — one setup covers all lines.
git config merge.mergiraf.name   "mergiraf syntax-aware merge (ADR-0107)"
git config merge.mergiraf.driver "$DRIVER"

echo "✓ Mergiraf registered as merge driver for $(git rev-parse --show-toplevel)"
echo "    $MRG_VERSION"
echo "    driver: $DRIVER"
echo
echo "Attributes (versioned in .gitattributes): $(grep -cE '^[^#].*merge=mergiraf' .gitattributes) globs → mergiraf"
echo "Sanity: mergiraf reports $(mergiraf languages 2>/dev/null | grep -ci rust || echo '?') Rust grammar(s)."
echo
echo "Reminder: Mergiraf is Camada 1 only. The tested integration gate"
echo "(scripts/foundational-integrate.sh) is what keeps main green (ADR-0107)."

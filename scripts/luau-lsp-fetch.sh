#!/usr/bin/env bash
# luau-lsp-fetch.sh — the PINNED luau-lsp (crates/ph2d-code-lsp/luau-lsp.toml) for this OS, sha256-verified,
# in ${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/luau-lsp/<version>/. The law: pinned-helper-fetch.sh.
#
# Usage: bash scripts/luau-lsp-fetch.sh            # prints the binary path on stdout
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$raiz/scripts/pinned-helper-fetch.sh" "$raiz/crates/ph2d-code-lsp/luau-lsp.toml" luau-lsp

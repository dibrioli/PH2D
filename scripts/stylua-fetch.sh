#!/usr/bin/env bash
# stylua-fetch.sh — the PINNED StyLua (crates/ph2d-code-format/stylua-pin.toml) for this OS, sha256-verified,
# in ${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/stylua/<version>/. The law: pinned-helper-fetch.sh.
#
# Usage: bash scripts/stylua-fetch.sh              # prints the binary path on stdout
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$raiz/scripts/pinned-helper-fetch.sh" "$raiz/crates/ph2d-code-format/stylua-pin.toml" stylua

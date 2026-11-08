#!/usr/bin/env bash
# pinned-helpers-next-to-exe.sh — puts the PINNED helper programs (luau-lsp, StyLua) next to the built
# editor, where `Pin::resolve` looks second (crates/ph2d-code-lsp/src/resolve.rs): a clean home folder
# then still has suggestions, help, rename and format-on-save. ADR-0188; owner decisions 2 and 6
# (docs/CodeEditor/00_decisoes_do_dono.md: the helpers travel WITH the app).
#
# Each helper comes from its fetch script (pinned version + sha256; a verified copy already in the cache
# is reused with NO network), then is copied into `<target>/<profile>/`. Never called from build.rs:
# `cargo build` stays offline-capable.
#
# Usage: bash scripts/pinned-helpers-next-to-exe.sh [profile ...]    # default: smoke
#        CARGO_TARGET_DIR is honoured. Prints one `<helper> -> <path>` line per placed copy.
# Exit: 0 all placed · 2 usage/unsupported OS · 3 sha256 mismatch · 4 wrong version (from the fetch law).
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
alvo="${CARGO_TARGET_DIR:-$raiz/target}"
case "$alvo" in /*) ;; *) alvo="$raiz/$alvo" ;; esac

[ $# -gt 0 ] || set -- smoke
perfis=()
for p in "$@"; do
  case "$p" in
    dev|debug) perfis+=(debug) ;;
    -*|*/*|'') echo "pinned-helpers: bad profile «$p»" >&2; exit 2 ;;
    *) perfis+=("$p") ;;
  esac
done

for fetch in luau-lsp-fetch.sh stylua-fetch.sh; do
  bin="$(bash "$raiz/scripts/$fetch")"
  [ -x "$bin" ] || { echo "pinned-helpers: $fetch gave no executable ($bin)" >&2; exit 2; }
  for perfil in "${perfis[@]}"; do
    dir="$alvo/$perfil"
    mkdir -p "$dir"
    # A COPY, not a link: the build folder works even after the cache is wiped (reflink on btrfs = free).
    rm -f "$dir/$(basename "$bin")"
    cp --reflink=auto "$bin" "$dir/" 2>/dev/null || cp "$bin" "$dir/"
    echo "$(basename "$bin") -> $dir/$(basename "$bin")"
  done
done

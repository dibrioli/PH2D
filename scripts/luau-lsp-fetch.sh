#!/usr/bin/env bash
# luau-lsp-fetch.sh — downloads the PINNED luau-lsp for this OS, verifies its sha256, unpacks it and prints
# the binary path. Idempotent: a verified copy already in the cache is reused (and re-verified).
#
# Pin: crates/ph2d-code-lsp/luau-lsp.toml (version + sha256 per asset). Cache:
# ${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/luau-lsp/<version>/ — the same rule `ph2d-code-lsp` resolves.
# A sha256 mismatch is FATAL (exit 3) and nothing is unpacked. ADR-0188, CodeEditor Onda 0 T3.
#
# Usage: bash scripts/luau-lsp-fetch.sh            # prints the binary path on stdout
set -euo pipefail

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifesto="$raiz/crates/ph2d-code-lsp/luau-lsp.toml"
[ -f "$manifesto" ] || { echo "luau-lsp-fetch: manifest missing: $manifesto" >&2; exit 2; }

# Reads `key = "value"` at top level (section "") or inside `[asset.<name>]`.
ler() { # ler <section> <key>
  awk -v sec="$1" -v key="$2" '
    /^\[/ { cur = $0; gsub(/[\[\]]/, "", cur); next }
    /^[[:space:]]*#/ { next }
    {
      split($0, kv, "=")
      k = kv[1]; gsub(/[[:space:]]/, "", k)
      if (k == key && cur == sec) {
        v = substr($0, index($0, "=") + 1); gsub(/^[[:space:]]*"|"[[:space:]]*$/, "", v)
        print v; exit
      }
    }' "$manifesto"
}

versao="$(ler "" version)"
[ -n "$versao" ] || { echo "luau-lsp-fetch: no version in $manifesto" >&2; exit 2; }

case "$(uname -s)" in
  Linux)
    case "$(uname -m)" in
      x86_64|amd64) alvo=linux-x86_64 ;;
      aarch64|arm64) alvo=linux-arm64 ;;
      *) echo "luau-lsp-fetch: no published luau-lsp for Linux $(uname -m)" >&2; exit 2 ;;
    esac ;;
  Darwin) alvo=macos ;;
  MINGW*|MSYS*|CYGWIN*) alvo=windows-x86_64 ;;
  *) echo "luau-lsp-fetch: unsupported OS $(uname -s)" >&2; exit 2 ;;
esac

url="$(ler "asset.$alvo" url)"
sha="$(ler "asset.$alvo" sha256)"
bin="$(ler "asset.$alvo" binary)"
[ -n "$url" ] && [ -n "$sha" ] && [ -n "$bin" ] || { echo "luau-lsp-fetch: asset.$alvo incomplete in $manifesto" >&2; exit 2; }

sha256_de() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

cache="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/luau-lsp/$versao"
zip="$cache/$(basename "$url")"
destino="$cache/$bin"
mkdir -p "$cache"

if [ ! -f "$zip" ] || [ "$(sha256_de "$zip")" != "$sha" ]; then
  parcial="$zip.part.$$"
  trap 'rm -f "$parcial"' EXIT
  echo "luau-lsp-fetch: downloading $url" >&2
  curl -sSfL --retry 3 -o "$parcial" "$url"
  obtido="$(sha256_de "$parcial")"
  if [ "$obtido" != "$sha" ]; then
    echo "luau-lsp-fetch: SHA256 MISMATCH for $url" >&2
    echo "  expected $sha" >&2
    echo "  got      $obtido" >&2
    exit 3
  fi
  mv -f "$parcial" "$zip"
  rm -f "$destino"
fi

if [ ! -x "$destino" ]; then
  tmp="$(mktemp -d "$cache/unpack.XXXXXX")"
  if command -v unzip >/dev/null 2>&1; then unzip -q -o "$zip" -d "$tmp"
  else bsdtar -xf "$zip" -C "$tmp"; fi
  [ -f "$tmp/$bin" ] || { echo "luau-lsp-fetch: $bin not inside $(basename "$zip")" >&2; rm -rf "$tmp"; exit 2; }
  chmod +x "$tmp/$bin"
  mv -f "$tmp/$bin" "$destino"
  rm -rf "$tmp"
fi

relatada="$("$destino" --version 2>/dev/null | tr -d '\r' | head -1 || true)"
case "$relatada" in
  *"$versao"*) ;;
  *) echo "luau-lsp-fetch: $destino reports «$relatada», expected $versao" >&2; exit 4 ;;
esac

echo "$destino"

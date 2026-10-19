#!/usr/bin/env bash
# pinned-helper-fetch.sh — downloads a PINNED helper program for this OS, verifies its sha256, unpacks it
# and prints the binary path. Idempotent: a verified copy already in the cache is reused (and re-verified).
# ONE law for every helper (the Rust side is `ph2d_code_lsp::Pin`): `luau-lsp-fetch.sh`, `stylua-fetch.sh`.
#
# Pin: the manifest (version + sha256 per asset). Cache: ${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/<tool>/<version>/
# — the same rule `Pin::resolve` follows. A sha256 mismatch is FATAL (exit 3) and nothing is unpacked.
# ADR-0188, CodeEditor Onda 0 T3 (luau-lsp) and Onda 2 I13 (StyLua).
#
# Asset keys: linux-x86_64 · linux-arm64 · macos-x86_64 / macos-arm64 (or one universal `macos`) ·
# windows-x86_64.
#
# Usage: bash scripts/pinned-helper-fetch.sh <manifest> <tool>     # prints the binary path on stdout
set -euo pipefail

[ $# -eq 2 ] || { echo "usage: pinned-helper-fetch.sh <manifest> <tool>" >&2; exit 2; }
manifesto="$1"
ferramenta="$2"
[ -f "$manifesto" ] || { echo "$ferramenta-fetch: manifest missing: $manifesto" >&2; exit 2; }

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
[ -n "$versao" ] || { echo "$ferramenta-fetch: no version in $manifesto" >&2; exit 2; }

case "$(uname -m)" in
  x86_64|amd64) arq=x86_64 ;;
  aarch64|arm64) arq=arm64 ;;
  *) arq="$(uname -m)" ;;
esac
case "$(uname -s)" in
  Linux) alvo="linux-$arq" ;;
  Darwin)
    # One asset per arch (StyLua) or one universal Mach-O (luau-lsp).
    alvo="macos-$arq"
    [ -n "$(ler "asset.$alvo" url)" ] || alvo=macos ;;
  MINGW*|MSYS*|CYGWIN*) alvo=windows-x86_64 ;;
  *) echo "$ferramenta-fetch: unsupported OS $(uname -s)" >&2; exit 2 ;;
esac

url="$(ler "asset.$alvo" url)"
sha="$(ler "asset.$alvo" sha256)"
bin="$(ler "asset.$alvo" binary)"
[ -n "$url" ] && [ -n "$sha" ] && [ -n "$bin" ] || {
  echo "$ferramenta-fetch: no published $ferramenta for $(uname -s) $(uname -m) (asset.$alvo in $manifesto)" >&2
  exit 2
}

sha256_de() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

cache="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d/$ferramenta/$versao"
zip="$cache/$(basename "$url")"
destino="$cache/$bin"
mkdir -p "$cache"

if [ ! -f "$zip" ] || [ "$(sha256_de "$zip")" != "$sha" ]; then
  parcial="$zip.part.$$"
  trap 'rm -f "$parcial"' EXIT
  echo "$ferramenta-fetch: downloading $url" >&2
  curl -sSfL --retry 3 -o "$parcial" "$url"
  obtido="$(sha256_de "$parcial")"
  if [ "$obtido" != "$sha" ]; then
    echo "$ferramenta-fetch: SHA256 MISMATCH for $url" >&2
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
  [ -f "$tmp/$bin" ] || { echo "$ferramenta-fetch: $bin not inside $(basename "$zip")" >&2; rm -rf "$tmp"; exit 2; }
  chmod +x "$tmp/$bin"
  mv -f "$tmp/$bin" "$destino"
  rm -rf "$tmp"
fi

relatada="$("$destino" --version 2>/dev/null | tr -d '\r' | head -1 || true)"
case "$relatada" in
  *"$versao"*) ;;
  *) echo "$ferramenta-fetch: $destino reports «$relatada», expected $versao" >&2; exit 4 ;;
esac

echo "$destino"

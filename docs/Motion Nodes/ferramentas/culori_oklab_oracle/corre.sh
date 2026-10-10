#!/usr/bin/env bash
# O ORÁCULO da cor (ciclo 18) — baixa o culori 4.0.2 (MIT, conferido no package.json do ARTEFACTO)
# para uma pasta temporária própria, corre `oracle.mjs` sobre as NOSSAS cores e grava, com
# cabeçalho, em `crates/ph2d-color/tests/fixtures/culori.txt`. Correr de novo é mudar os gates.
#
#   bash "docs/Motion Nodes/ferramentas/culori_oklab_oracle/corre.sh"
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
PKG="$(mktemp -d)"
trap 'rm -rf "$PKG"' EXIT
( cd "$PKG" && npm pack culori@4.0.2 >/dev/null 2>&1 && tar xzf culori-4.0.2.tgz )
LIC="$(node -e "console.log(require('$PKG/package/package.json').license)")"
[ "$LIC" = "MIT" ] || { echo "licenca inesperada: $LIC" >&2; exit 1; }
SAIDA="$RAIZ/crates/ph2d-color/tests/fixtures/culori.txt"
mkdir -p "$(dirname "$SAIDA")"
{
  echo "# oraculo: culori 4.0.2 (npm pack culori@4.0.2) · licenca $LIC · node $(node --version)"
  echo "# comando: bash \"docs/Motion Nodes/ferramentas/culori_oklab_oracle/corre.sh\""
  echo "# CONV nome | L a b (oklab) | L C h (oklch, h em graus) — entrada em RGB LINEAR"
  echo "# MIX a-b espaco | r,g,b (RGB linear, SEM corte) a t = k/8, k = 0..8"
  echo "# GAMUT a-b espaco | o mesmo depois do toGamut('rgb','oklch') — o mapeamento do CSS Color 4"
  echo "# data: $(date -I)"
  ( cd "$PKG" && node "$AQUI/oracle.mjs" "$PKG/package" )
} > "$SAIDA"
wc -l "$SAIDA"

#!/usr/bin/env bash
# O ORÁCULO da cascata e das curvas (ciclo 17, doc 126 §2) — baixa o anime.js 3.2.2 (MIT, conferido
# no package.json do ARTEFACTO) para uma pasta temporária própria, corre `oracle.cjs` sobre as NOSSAS
# grelhas e grava, com cabeçalho, em `crates/ph2d-ease/tests/fixtures/animejs.txt`. Os gates da
# `ph2d-ease` leem DESTE ficheiro: correr o oráculo de novo é mudar os gates.
#
#   bash "docs/Motion Nodes/ferramentas/animejs_stagger_oracle/corre.sh"
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
PKG="$(mktemp -d)"
trap 'rm -rf "$PKG"' EXIT
( cd "$PKG" && npm pack animejs@3.2.2 >/dev/null 2>&1 && tar xzf animejs-3.2.2.tgz )
LIC="$(node -e "console.log(require('$PKG/package/package.json').license)")"
[ "$LIC" = "MIT" ] || { echo "licenca inesperada: $LIC" >&2; exit 1; }
SAIDA="$RAIZ/crates/ph2d-ease/tests/fixtures/animejs.txt"
mkdir -p "$(dirname "$SAIDA")"
{
  echo "# oraculo: anime.js 3.2.2 (npm pack animejs@3.2.2) · licenca $LIC · node $(node --version)"
  echo "# comando: bash \"docs/Motion Nodes/ferramentas/animejs_stagger_oracle/corre.sh\""
  echo "# STAGGER = anime.stagger(1, {grid, from, axis, easing})(el, i, n); EASE = anime.easing(nome)(k/64)"
  echo "# data: $(date -I)"
  ( cd "$PKG" && node "$AQUI/oracle.cjs" "$PKG/package/lib/anime.js" )
} > "$SAIDA"
wc -l "$SAIDA"

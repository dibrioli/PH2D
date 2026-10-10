#!/usr/bin/env bash
# O ORÁCULO do crossfade (ciclo 19) — corre o Godot instalado (licença conferida no ARTEFACTO pelo
# pacman) sobre `oracle.gd` e grava, com cabeçalho, em
# `crates/ph2d-node-value-follow/tests/fixtures/godot_xfade.txt`. Correr de novo é mudar o gate.
#
#   bash "docs/Motion Nodes/ferramentas/godot_xfade_oracle/corre.sh"
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
LIC="$(LANG=C pacman -Qi godot | sed -n 's/^Licenses *: *//p')"
VER="$(LANG=C pacman -Qi godot | sed -n 's/^Version *: *//p')"
[ "$LIC" = "MIT" ] || { echo "licenca inesperada: $LIC" >&2; exit 1; }
SAIDA="$RAIZ/crates/ph2d-node-value-follow/tests/fixtures/godot_xfade.txt"
mkdir -p "$(dirname "$SAIDA")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
{
  echo "# oraculo: godot $VER (pacman) · licenca $LIC"
  echo "# comando: bash \"docs/Motion Nodes/ferramentas/godot_xfade_oracle/corre.sh\""
  echo "# AnimationNodeTransition, xfade_time X, curva linear; duas animacoes paradas (0 e 1);"
  echo "# tree.advance(1/60) a mao; o pedido de transicao antes do quadro 'pedido_no'."
  echo "# XFADE X pedido_no K | o valor da propriedade (= o peso do estado novo) a cada quadro"
  echo "# data: $(date -I)"
  HOME="$TMP" XDG_DATA_HOME="$TMP" XDG_CONFIG_HOME="$TMP" godot --headless --script "$AQUI/oracle.gd" 2>/dev/null | grep '^XFADE'
} > "$SAIDA"
wc -l "$SAIDA"

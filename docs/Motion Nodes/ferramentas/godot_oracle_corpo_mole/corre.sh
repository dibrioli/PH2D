#!/usr/bin/env bash
# O ORÁCULO do Bug #15 (doc 124 §0.2) — corre o Godot (MIT, `pacman -Qi godot`) sobre a NOSSA
# fixtura nos três modos e grava a saída, com cabeçalho, em `saida.txt`. O gate
# `the_collider_holds_the_diameter_and_the_body_does_not_swell` lê o número do modo `lei` DESTE
# ficheiro: correr o oráculo de novo é mudar o gate.
#
#   bash "docs/Motion Nodes/ferramentas/godot_oracle_corpo_mole/corre.sh"
set -euo pipefail
cd "$(dirname "$0")"
{
  echo "# oraculo: $(godot --version 2>/dev/null | tail -1) · licenca $(pacman -Qi godot | awk -F': ' '/^(Licen)/{print $2}')"
  echo "# fixtura: soft_body_radius (5x5, passo 0,30, raio 0,28, ancora ±2,0 / 0,7 s), 3 s a 960 Hz"
  echo "# data: $(date -I)"
  for modo in nu lei todos; do
    timeout 300 godot --headless --script corpo_mole.gd -- "$modo" 2>&1 | grep '^ORACULO'
  done
} > saida.txt
cat saida.txt

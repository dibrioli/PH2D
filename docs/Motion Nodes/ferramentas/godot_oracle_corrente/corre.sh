#!/usr/bin/env bash
# O ORÁCULO do Bug #16 (doc 125) — corre o Godot (MIT, `pacman -Qi godot`) sobre a NOSSA fixtura
# `rope_thickness` nos três modos e grava a saída, com cabeçalho, em `saida.txt`. O gate
# `the_rope_reads_its_pieces_collider_and_does_not_cross` lê o número do modo `lei` DESTE ficheiro:
# correr o oráculo de novo é mudar o gate.
#
#   bash "docs/Motion Nodes/ferramentas/godot_oracle_corrente/corre.sh"
set -euo pipefail
cd "$(dirname "$0")"
{
  echo "# oraculo: $(godot --version 2>/dev/null | tail -1) · licenca $(pacman -Qi godot | awk -F': ' '/^(Licen)/{print $2}')"
  echo "# fixtura: rope_thickness (24 contas, comprimento 6,0, raio 0,5, ancora 3,0·sin(2πt/0,6)), 4 s a 4800 Hz, molas k=1e6 ao longo dos elos"
  echo "# data: $(date -I)"
  for modo in nu lei todos; do
    timeout 300 godot --headless --script corrente.gd -- "$modo" 2>&1 | grep '^CORRENTE'
  done
} > saida.txt
cat saida.txt

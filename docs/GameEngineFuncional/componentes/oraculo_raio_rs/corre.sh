#!/usr/bin/env bash
# Corre o oráculo do Godot e regrava `godot_rs.txt` (a fixture do gate `oraculo_godot_das_melhorias`).
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/GameEngineFuncional/componentes/oraculo_raio_rs/corre.sh
set -euo pipefail
cd "$(dirname "$0")"
godot --headless --path . --script oraculo.gd >/dev/null 2>&1 || true
test -s godot_rs.txt && cat godot_rs.txt

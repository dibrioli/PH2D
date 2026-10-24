#!/usr/bin/env bash
# Item 4 (24/10): as formas do miolo × o produto, nas duas placas (a NVIDIA com a vez; a Radeon pelo radeon.sh).
# Uso: bash docs/Luz2D/ferramentas/item4_formas.sh [nome do teste]   (logs em target/item4/)
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
nome="${1:-medida_luz2d_miolo_formas}"
t=(cargo test -p ph2d-app-light2d --profile smoke --test it "$nome" -- --ignored --nocapture --test-threads=1)
mkdir -p target/item4
PH2D_GPU=1 PH2D_PRAZO=2300 bash scripts/ph2d-run.sh "${t[@]}" > "target/item4/${nome}_nvidia.log" 2>&1
echo "nvidia exit $?"
PH2D_PRAZO=2300 bash scripts/ph2d-run.sh bash docs/Luz2D/ferramentas/radeon.sh "${t[@]}" > "target/item4/${nome}_radeon.log" 2>&1
echo "radeon exit $?"

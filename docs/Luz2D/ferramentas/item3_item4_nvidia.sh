#!/usr/bin/env bash
# Itens 3b e 4 (24/10) na NVIDIA, numa vez da placa: as fases do miolo e a lista dos pretos da W6d.
# Uso: PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Luz2D/ferramentas/item3_item4_nvidia.sh
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
t=(cargo test -p ph2d-app-light2d --profile smoke --test it)
"${t[@]}" medida_luz2d_miolo_por_placa -- --ignored --nocapture --test-threads=1 > target/item4/fases_nvidia.log 2>&1
echo "fases exit $?"
PH2D_W6D_LISTA=1 "${t[@]}" medida_luz2d_w6d_borda -- --ignored --nocapture --test-threads=1 > target/item3/w6d_lista_nvidia.log 2>&1
echo "lista exit $?"

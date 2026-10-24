#!/usr/bin/env bash
# Item 4 (24/10): os portões do miolo por nome, nas duas placas (logs em target/item4/portoes_*).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
gpu=(cargo test -p ph2d-light2d-gpu --profile smoke --test it -- o_custo_do_miolo o_miolo_na_placa --nocapture --test-threads=1)
app=(cargo test -p ph2d-app-light2d --profile smoke --test it -- o_miolo_das_formas o_interior_do_opaco a_vista_renasce bug7_o_mesmo_bit --nocapture --test-threads=1)
for placa in nvidia radeon; do
  if [ "$placa" = nvidia ]; then pre=(env PH2D_GPU=1 PH2D_PRAZO=2300 bash scripts/ph2d-run.sh)
  else pre=(env PH2D_PRAZO=2300 bash scripts/ph2d-run.sh bash docs/Luz2D/ferramentas/radeon.sh); fi
  "${pre[@]}" "${gpu[@]}" > "target/item4/portoes_gpu_$placa.log" 2>&1; echo "$placa gpu exit $?"
  "${pre[@]}" "${app[@]}" > "target/item4/portoes_app_$placa.log" 2>&1; echo "$placa app exit $?"
done

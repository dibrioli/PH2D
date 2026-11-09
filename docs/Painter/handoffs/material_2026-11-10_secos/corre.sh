#!/usr/bin/env bash
# Uma compilação smoke; 10 corridas (pressão 1 e rampa × 1–5 passadas).
set -uo pipefail
S=$1; R=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
cd "$R"
for n in 1 2 3 4 5; do
  for pr in "" rampa; do
    env PH2D_V3_SO="Pastel,Chalk,Crayon" PH2D_V3_VARIANTES="$S/var_secos.tsv" PH2D_CSP_CENA="$S/cena" PH2D_V3_PASSADAS=$n ${pr:+PH2D_V3_PRESSAO=$pr} \
      bash scripts/ph2d-run.sh cargo test -q -p ph2d-tool-painter --profile smoke --test it sonda_csp_v3_revisao -- --ignored --nocapture 2>&1 | grep -E "test result|error" | sed "s/^/[$n $pr] /"
  done
done

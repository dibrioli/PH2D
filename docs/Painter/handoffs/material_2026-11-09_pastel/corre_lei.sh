#!/usr/bin/env bash
# Rodada 2 do Pastel (09/11): as variantes de `var_pastel_lei.tsv` em pressão 1 (ida e volta), rampa 0,1→1 e rampa ×2.
# Uso: bash corre_lei.sh <scratch com cena/ e cena_volta/ da rodada 1>  (do worktree; UMA compilação --profile smoke)
set -euo pipefail
S=$1
R=$(git rev-parse --show-toplevel)
T=${T:-$R/docs/Painter/handoffs/material_2026-11-09_pastel/var_pastel_lei.tsv}
corre() { (cd "$R" && env PH2D_V3_SO=Pastel PH2D_V3_VARIANTES="$T" "$@" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter \
  --profile smoke --test it sonda_csp_v3_revisao -- --ignored --nocapture 2>&1 | grep -E '^\[v3\]|test result|error' ); }
corre PH2D_CSP_CENA="$S/cena"
corre PH2D_CSP_CENA="$S/cena_volta"
corre PH2D_CSP_CENA="$S/cena" PH2D_V3_PRESSAO=rampa
corre PH2D_CSP_CENA="$S/cena" PH2D_V3_PRESSAO=rampa PH2D_V3_PASSADAS=2

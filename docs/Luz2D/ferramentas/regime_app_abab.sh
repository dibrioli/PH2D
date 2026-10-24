#!/bin/bash
# ⭐ W6i item 2 — a app real cai no regime da vista `P`? (handoff W6h §13 b). Processos ALTERNADOS A B A B:
# A = `PH2D_REGIME_MODO=app`, B = `PH2D_REGIME_MODO=controlo` da régua `medida_luz2d_regime_app`.
#
# uso:
#   bash docs/Luz2D/ferramentas/regime_app_abab.sh <binário de teste it-*> <pares> <saida_dir> [VAR=valor ...]
#
# O binário constrói-se antes (`cargo test -p ph2d-app-light2d --profile smoke --test it --no-run`). Um log por
# processo em <saida_dir>/<i>_<modo>.log e o resumo das linhas O/C/D/F em <saida_dir>/resumo.txt.
set -euo pipefail

# Uma medição é dona da placa do princípio ao fim (docs/DevOps/A_VEZ_DA_PLACA.md).
if [ "${PH2D_NA_PORTA:-0}" != "1" ]; then
  exec env PH2D_GPU=1 bash "$(cd "$(dirname "$0")/../../.." && pwd)/scripts/ph2d-run.sh" bash "$0" "$@"
fi

BIN="${1:?binário}"
PARES="${2:?pares}"
SAIDA="${3:?saida_dir}"
shift 3
mkdir -p "$SAIDA"
: >"$SAIDA/resumo.txt"
for i in $(seq 1 "$PARES"); do
  for modo in app controlo; do
    log="$SAIDA/${i}_${modo}.log"
    env "$@" PH2D_REGIME_MODO="$modo" timeout 300 "$BIN" medida_luz2d_regime_app --ignored --nocapture \
      --test-threads=1 >"$log" 2>&1 || echo "✗ $i $modo: exit $?" >>"$SAIDA/resumo.txt"
    { echo "== $i $modo"; grep -E '^(P|O|C|D|F) |\[luz2d\]|panicked' "$log" || true; } >>"$SAIDA/resumo.txt"
  done
done

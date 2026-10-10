#!/usr/bin/env bash
# corre.sh <pasta_saida> [id_caso ...] — arranca a sala e o app (127.0.0.1), corre o harness, pára-os pelo PID.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLAB_HOME="${COLAB_HOME:-$HOME/Apps/excalidraw-colab-oracle}"
OUT="$(realpath -m "$1")"; shift
mkdir -p "$COLAB_HOME/harness"; cp -f "$HERE/corre_colab.mjs" "$COLAB_HOME/harness/corre_colab.mjs"   # o import «playwright» resolve-se a partir de lá
JA=0; ss -ltn 2>/dev/null | grep -qE '127\.0\.0\.1:3102 ' && ss -ltn 2>/dev/null | grep -qE '127\.0\.0\.1:5101 ' && JA=1
[[ $JA == 1 ]] || bash "$HERE/servidores.sh" start
trap '[[ $JA == 1 ]] || bash "$HERE/servidores.sh" stop' EXIT
CASOS_JSON="$HERE/../casos.json" timeout "${PRAZO_CORRIDA:-1800}" node "$COLAB_HOME/harness/corre_colab.mjs" "$OUT" "$@"

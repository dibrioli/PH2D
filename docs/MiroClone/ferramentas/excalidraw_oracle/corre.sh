#!/usr/bin/env bash
# Corre o oráculo Excalidraw: corre.sh [--sem-semente] <pasta_saida> <entrada.excalidraw>...
# (também aceita a forma do briefing: corre.sh <entrada.excalidraw> <pasta_saida>)
# Requer ./instala.sh corrido antes (instala em ${ORACLE_HOME:-~/Apps/excalidraw-oracle}).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$HOME/.local/bin:$PATH"
ORACLE_HOME="${ORACLE_HOME:-$HOME/Apps/excalidraw-oracle}"
[[ -f "$ORACLE_HOME/www/oracle.js" ]] || { echo "oráculo não instalado: corra $HERE/instala.sh" >&2; exit 2; }

flags=()
[[ "${1:-}" == "--sem-semente" ]] && { flags+=(--sem-semente); shift; }
if [[ $# -eq 2 && "$1" == *.excalidraw && "$2" != *.excalidraw ]]; then
  set -- "$2" "$1"
fi
exec timeout "${ORACLE_TIMEOUT:-600}" node "$HERE/corre.mjs" "${flags[@]}" "$@"

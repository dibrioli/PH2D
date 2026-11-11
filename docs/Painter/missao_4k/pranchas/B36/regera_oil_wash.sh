#!/usr/bin/env bash
# B36.1 (PARCIAL, 11/11): o Oil Wash está em `EM_ESTUDO` (familias/oleo_wash.rs), fora da fábrica — nada a regerar.
# Ao promovê-lo (mover para `WET`), troque a linha `exit 0` pela gravação abaixo e corra-o pela fila pesada.
set -euo pipefail
echo "B36.1: Oil Wash em estudo — nada regerado"
exit 0
# cd "$(git rev-parse --show-toplevel)"
# PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Oil Wash" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter \
#   --profile smoke --test it gera_os_presets_de_fabrica -- --ignored --nocapture

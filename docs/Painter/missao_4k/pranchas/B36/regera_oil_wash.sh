#!/usr/bin/env bash
# B36.1 (11/11): o Oil Wash está em `WET` (familias/oleo_wash.rs) — grava o ficheiro de fábrica dele (só ele).
# Corre pela fila pesada (o guarda recusa-o a um subagente). Depois: `a_largura_de_fabrica_e_a_aprovada` por nome
# acusa «Oil Wash: [a, b, c] (aprovada None)» — acrescente SÓ essa linha a
# `crates/ph2d-tool-painter/tests/it/presets_de_fabrica/larguras_aprovadas.tsv` (nunca regrave o ficheiro inteiro).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Oil Wash" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter \
  --profile smoke --test it gera_os_presets_de_fabrica -- --ignored --nocapture

#!/bin/bash
# Sonda do passo 0/alvo do ADR-0204 (n.º 7 item 2): 2.º pen-down WB 4K, `vidro_do_quadro`.
# Corre pela fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM2_sonda.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
PH2D_SONDA_SO="Watercolor Basic" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 \
  bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
cat /proc/loadavg

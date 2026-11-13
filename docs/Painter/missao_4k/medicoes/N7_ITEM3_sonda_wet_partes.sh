#!/bin/bash
# Sonda do n.º 7 item 3 (resto do «+»): Wet Wash 4K, as PARTES do «+» (ms e MB do fio principal, `sonda_partes`)
# e a chave `JANELA_DA_TINTA_PARALELA` ligada × desligada (PH2D_SONDA_JANELA=ab), 4 blocos rodados.
# Fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3_sonda_wet_partes.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
PH2D_SONDA_JANELA=ab PH2D_SONDA_REPETE=4 PH2D_SONDA_SO="Wet Wash" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 \
  bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
cat /proc/loadavg

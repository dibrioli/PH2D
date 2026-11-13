#!/bin/bash
# Sonda do n.º 7 item 3 (continuação 3): o fio da compressão da adormecida no «+» (chave `ENCOLHE_NO_SILENCIO`,
# PH2D_SONDA_ENCOLHE=ab, 4 blocos rodados) e o 1.º traço na camada nova por partes (ev0..2), Wet Wash e Watercolor Basic 4K.
# Fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3d_sonda.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
for so in "Wet Wash" "Watercolor Basic"; do
  PH2D_SONDA_ENCOLHE=ab PH2D_SONDA_REPETE=4 PH2D_SONDA_SO="$so" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 \
    bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
done
cat /proc/loadavg

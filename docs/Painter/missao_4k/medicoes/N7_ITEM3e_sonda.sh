#!/bin/bash
# Sonda do n.º 7 item 3e: o 1.º pen-down do Watercolor Basic 4K por dentro (`down.*`, PH2D_SONDA_PARTES_DOWN) na base e na
# camada nova, A×B da chave `FUNDO_AQUECIDO_NA_TROCA` (PH2D_SONDA_AQUECE=ab, 6 blocos rodados no MESMO processo), e o
# Wet Wash como vizinho (a chave não o toca: o Wet não usa o fundo da aguada).
# Fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3e_sonda.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
for so in "Watercolor Basic" "Wet Wash"; do
  PH2D_SONDA_AQUECE=ab PH2D_SONDA_REPETE=6 PH2D_SONDA_SO="$so" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 \
    PH2D_SONDA_PARTES=1 PH2D_SONDA_PARTES_DOWN=1 PH2D_SONDA_NOVA=1 \
    bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
done
cat /proc/loadavg

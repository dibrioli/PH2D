#!/bin/bash
# Sonda do n.º 7 item 3f: o aquecimento do fundo do toque num fio (fora do clique do «+»). A×B da chave
# `FUNDO_AQUECIDO_EM_FUNDO` (PH2D_SONDA_EM_FUNDO=ab, 6 blocos rodados no MESMO processo; `false` = a rota do clique do
# item 3e, o oráculo): o «+» (`mais.aquece_o_fundo`) e o 1.º pen-down na nova (`down.*`). Wet Wash como vizinho.
# Fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3f_sonda.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
for so in "Watercolor Basic" "Wet Wash"; do
  PH2D_SONDA_EM_FUNDO=ab PH2D_SONDA_REPETE=6 PH2D_SONDA_SO="$so" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 \
    PH2D_SONDA_PARTES=1 PH2D_SONDA_PARTES_DOWN=1 PH2D_SONDA_NOVA=1 \
    bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
done
cat /proc/loadavg

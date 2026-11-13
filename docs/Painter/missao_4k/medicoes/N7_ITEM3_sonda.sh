#!/bin/bash
# Sonda do n.º 7 item 3: o «+» a 4K e o 1.º traço da camada nova, chave MAIS_SEM_DIFF ligada × desligada,
# intercaladas em 4 blocos rodados no mesmo binário; WB e Wet Wash. Resumo = o MÍNIMO de cada rota
# (linhas «  «+» rota=true|false … ms · MB» e «TRACO … journal=true|false 1.º … 2.º …»; com PH2D_SONDA_MAIS o
# `journal=` da linha TRACO é a rota do «+»).
# Corre pela fila pesada (coordenador): bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3_sonda.sh'
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor || exit 1
cat /proc/loadavg
for so in "Watercolor Basic" "Wet Wash"; do
  PH2D_SONDA_MAIS=ab PH2D_SONDA_REPETE=4 PH2D_SONDA_SO="$so" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 \
    bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture
done
cat /proc/loadavg

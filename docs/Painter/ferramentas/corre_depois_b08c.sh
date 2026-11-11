#!/bin/bash
# B08c: o «depois» com o mesmo roteiro do ANTES.md. Corre DENTRO de um só ph2d-run.sh e segura a vez da placa do
# princípio ao fim (o mede_pen_up_4k.sh matriz pede-a): entre lançamentos outra árvore não entra na placa.
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
S=$W/target/mede-depois; T=$W/docs/Painter/ferramentas/mede_pen_up_4k.sh
mkdir -p $S
B=$S/ph2d-host-desktop
[ -e $B ] || cp --reflink=auto $W/target/smoke/ph2d-host-desktop $B
NUCLEO=("omissão:digital" "omissão:water" "omissão:impasto" "omissão:wet" "Watercolor Basic:water" "Filbert:wet" "Express Oil:wet")
grupo() { TELA=$1; FUNDO=$2; shift 2
  echo "GRUPO $TELA $FUNDO $(date +%T) $(cut -d' ' -f1-3 /proc/loadavg)" >> $S/carga.txt
  P=""; [ "$TELA" = 3840x2160 ] && P=1
  PNG=$P SAIDA=$S bash $T matriz $TELA $FUNDO $B "$@"
  echo "FIM $TELA $FUNDO $(date +%T) $(cut -d' ' -f1-3 /proc/loadavg)" >> $S/carga.txt; }
grupo 3840x2160 papel
grupo 3840x2160 branco "${NUCLEO[@]}"
grupo 1920x1080 papel "${NUCLEO[@]}"
grupo 3840x2160 blocos "${NUCLEO[@]}"
grupo 1024x1024 blocos "${NUCLEO[@]}"

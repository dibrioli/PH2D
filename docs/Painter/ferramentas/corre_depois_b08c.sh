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
cantos() {
# B17.5: Ctrl+Z de dois toques em cantos opostos do 4K papel (CANTOS: traço curto ao alto-esquerda, toque ao fundo-direita,
# dois Ctrl+Z seguidos). Saída à parte (target/mede-depois/cantos): as chaves pincel×tela×fundo são as do 4K papel.
echo "GRUPO cantos $(date +%T) $(cut -d' ' -f1-3 /proc/loadavg)" >> $S/carga.txt
CANTOS="390 140 1240 890" CURTO=1 SAIDA=$S/cantos bash $T matriz 3840x2160 papel $B "omissão:digital" "omissão:impasto" "omissão:wet"
echo "FIM cantos $(date +%T) $(cut -d' ' -f1-3 /proc/loadavg)" >> $S/carga.txt; }
# Por prioridade (o 4K papel é o ANTES inteiro). GRUPOS="1 6" corre só esses (1..5 = as telas, 6 = cantos).
for g in ${GRUPOS:-1 6 2 3 4 5}; do case $g in
  1) grupo 3840x2160 papel ;;
  2) grupo 3840x2160 branco "${NUCLEO[@]}" ;;
  3) grupo 1920x1080 papel "${NUCLEO[@]}" ;;
  4) grupo 3840x2160 blocos "${NUCLEO[@]}" ;;
  5) grupo 1024x1024 blocos "${NUCLEO[@]}" ;;
  6) cantos ;;
esac; done

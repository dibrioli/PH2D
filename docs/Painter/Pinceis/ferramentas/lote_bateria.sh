#!/bin/bash
# O LOTE LONGO das fotos da bateria no Rebelle, em SESSÕES: cada sessão abre a tela virtual pela porta da linha
# com a vez da placa (PH2D_GPU=1, prazo PH2D_PRAZO, 1800 s por omissão — entre sessões a placa volta à fila das
# outras linhas), passa o arranque (rebelle_pronto.sh), fotografa até ao prazo e fecha SÓ com gotas_fecha.sh.
# Retomável: as fotos que já existem saltam-se. Uma cena que derruba o Rebelle duas vezes vai para salta.txt.
# Passagens: 1.ª = a primeira cena de cada preset (PRIMEIRA=1); 2.ª = todas.
# uso: lote_bateria.sh <lista> <pasta de saída>     (corre da raiz da árvore)
set -u
L="$1"; S="$2"; F="$(cd "$(dirname "$0")" && pwd)"; R="$(git -C "$F" rev-parse --show-toplevel)"
W="$R/docs/Painter/NewFeaturesWetPaint/ferramentas"; O="$HOME/Apps/rebelle/oraculo"
P="${PH2D_PRAZO:-1800}"; mkdir -p "$S"
anterior=""
for passagem in 1 2; do
  while :; do
    echo "== $(date +%T) passagem $passagem: sessão nova"
    rm -f "$O/display.txt"
    PH2D_GPU=1 PH2D_PRAZO="$P" bash "$R/scripts/ph2d-run.sh" bash "$W/gotas_inicia.sh" > "$S/.sessao.log" 2>&1 &
    pid=$!; t0=$(date +%s)
    if ! bash "$F/rebelle_pronto.sh"; then
      echo "   não arrancou ($(tail -2 "$S/.sessao.log" | tr '\n' ' '))"
      bash "$W/gotas_fecha.sh"; wait "$pid"; sleep 30; continue
    fi
    if [ "$passagem" = 1 ]; then e=(env PRIMEIRA=1); else e=(env); fi
    "${e[@]}" ATE=$((t0 + P - 180)) bash "$F/fotos_bateria.sh" "$L" "$S" "$S/log.csv"; rc=$?
    bash "$W/gotas_fecha.sh"; wait "$pid"
    echo "   $(date +%T) fotos rc=$rc · $(find "$S" -name '*.png' | wc -l) PNG"
    if [ $rc -eq 11 ]; then
      atual=$(cat "$S/.em_curso" 2>/dev/null)
      if [ -n "$atual" ] && [ "$atual" = "$anterior" ]; then echo "$atual" >> "$S/salta.txt"; echo "   salta $atual"; anterior=""
      else anterior="$atual"; fi
      continue
    fi
    [ $rc -eq 10 ] && continue
    break
  done
done
echo "LOTE_BATERIA_FIM $(find "$S" -name '*.png' | wc -l) PNG"

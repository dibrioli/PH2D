#!/bin/bash
# ORIGEM: cópia de /home/enio/Apps/rebelle/oraculo/amostra_cpu.sh (o oráculo Rebelle), trazida 30/10
# para medir o Wet Paint com a MESMA régua (docs/Painter/49_realshader/B_nucleos.md).
# uso: bash scripts/amostra_cpu.sh <pid> <saida.csv> <segundos>
# Amostra a CPU do processo Rebelle por THREAD (1 Hz): por linha `ts utime stime`, com
# separador `ts ---` e o VmRSS no fim de cada amostra. clock ticks = USER_HZ (100).
# uso: amostra_cpu.sh <pid> <saida.csv> <segundos>
pid=$1; out=$2; secs=$3
end=$(( $(date +%s) + secs ))
while [ "$(date +%s)" -lt "$end" ] && [ -d "/proc/$pid" ]; do
  ts=$(date +%s%3N)
  n=0
  for t in "/proc/$pid/task"/*; do
    corpo=$(sed 's/.*) //' "$t/stat" 2>/dev/null) || continue
    set -- $corpo
    n=$((n + 1))
    echo "$ts ${12} ${13}" >> "$out"
  done
  rss=$(awk '/VmRSS/{print $2}' "/proc/$pid/status" 2>/dev/null)
  echo "$ts --- threads=$n rss_kb=$rss" >> "$out"
  sleep 1
done

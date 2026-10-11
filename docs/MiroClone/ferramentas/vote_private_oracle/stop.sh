#!/usr/bin/env bash
# Pára SÓ os processos que o start.sh lançou, pelo PID guardado (nunca por padrão de nome).
O="${SCRUMLR_ORACLE:-$HOME/Apps/scrumlr-oracle}"
[ -f "$O/run/pids" ] || { echo "sem pids"; exit 0; }
tac "$O/run/pids" | while read -r name pid; do
  if kill -0 "$pid" 2>/dev/null; then kill -TERM "$pid"; echo "TERM $name $pid"; fi
done
sleep 2
tac "$O/run/pids" | while read -r name pid; do
  kill -0 "$pid" 2>/dev/null && { kill -KILL "$pid"; echo "KILL $name $pid"; }
done
rm -f "$O/run/pids"

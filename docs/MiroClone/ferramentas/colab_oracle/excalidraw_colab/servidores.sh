#!/usr/bin/env bash
# servidores.sh start|stop — sala (3102) e app (5101) em 127.0.0.1, cada um sob `timeout` (prazo) e com os
# PIDs guardados (o do timeout e o do node, a folha). Parar = kill PELO PID, a folha primeiro; nunca pkill.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLAB_HOME="${COLAB_HOME:-$HOME/Apps/excalidraw-colab-oracle}"
RUN="$COLAB_HOME/run"; mkdir -p "$RUN"
PRAZO="${PRAZO:-3h}"
vivo() { [[ -f "$RUN/$1.pid" ]] && kill -0 "$(head -1 "$RUN/$1.pid")" 2>/dev/null; }
case "${1:-}" in
start)
  for n in room app; do vivo "$n" && { echo "$n já corre (pids $(tr '\n' ' ' <"$RUN/$n.pid"))"; exit 1; }; done
  if ss -ltn 2>/dev/null | grep -qE '127\.0\.0\.1:(3102|5101) '; then echo "porta 3102/5101 ocupada por outro processo"; exit 1; fi
  ( cd "$COLAB_HOME/excalidraw-room" && exec env PORT=3102 CORS_ORIGIN="http://127.0.0.1:5101" NODE_ENV=production \
      nohup timeout "$PRAZO" node -r "$HERE/so_local.cjs" dist/index.js >"$RUN/room.log" 2>&1 ) & echo $! >"$RUN/room.pid"
  ( exec nohup timeout "$PRAZO" node "$HERE/serve_app.mjs" "$COLAB_HOME/excalidraw/excalidraw-app/build" 5101 >"$RUN/app.log" 2>&1 ) & echo $! >"$RUN/app.pid"
  for i in $(seq 1 50); do
    if curl -sf -o /dev/null http://127.0.0.1:5101/ && curl -s -o /dev/null "http://127.0.0.1:3102/socket.io/?EIO=4&transport=polling"; then
      for n in room app; do t="$(head -1 "$RUN/$n.pid")"; c="$(pgrep -P "$t" | head -1 || true)"; printf '%s\n%s\n' "$t" "$c" >"$RUN/$n.pid"; done
      echo "ok: room $(tr '\n' ' ' <"$RUN/room.pid")· app $(tr '\n' ' ' <"$RUN/app.pid")"; ss -ltnp 2>/dev/null | grep -E ':(3102|5101) ' || true; exit 0; fi
    sleep 0.2; done
  echo "não arrancou"; cat "$RUN/room.log" "$RUN/app.log"; exit 1;;
stop)
  for n in room app; do
    [[ -f "$RUN/$n.pid" ]] || continue
    t="$(sed -n 1p "$RUN/$n.pid")"; c="$(sed -n 2p "$RUN/$n.pid")"
    [[ -n "$c" ]] && kill "$c" 2>/dev/null || true   # a folha (node) primeiro
    sleep 0.3; kill -0 "$t" 2>/dev/null && kill "$t" 2>/dev/null || true
    rm -f "$RUN/$n.pid"
  done; echo parado;;
*) echo "uso: $0 start|stop"; exit 2;;
esac

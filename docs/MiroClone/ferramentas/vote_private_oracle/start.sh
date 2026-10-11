#!/usr/bin/env bash
# Arranca Postgres portátil + nats-server + scrumlr-server em 127.0.0.1, sem root nem docker.
# Cada processo vai sob `timeout` (prazo de parede) e o PID fica em $O/run/pids.
set -euo pipefail
O="${SCRUMLR_ORACLE:-$HOME/Apps/scrumlr-oracle}"
PRAZO="${PRAZO:-7200}"
PG_PORT=55432 NATS_PORT=54222 HTTP_PORT=58080
mkdir -p "$O/run" "$O/logs"
: > "$O/run/pids"
[ -d "$O/run/pgdata" ] || "$O/pg/bin/initdb" -D "$O/run/pgdata" -U admin --auth=trust -E UTF8 --locale=C >/dev/null

setsid timeout "$PRAZO" "$O/pg/bin/postgres" -D "$O/run/pgdata" -p $PG_PORT -h 127.0.0.1 -k "$O/run" \
  >"$O/logs/postgres.log" 2>&1 < /dev/null & echo "postgres $!" >> "$O/run/pids"
for i in $(seq 1 50); do "$O/pg/bin/pg_ctl" -D "$O/run/pgdata" status >/dev/null 2>&1 && \
  (exec 3<>/dev/tcp/127.0.0.1/$PG_PORT) 2>/dev/null && break; sleep 0.2; done
# O bundle não traz psql: usa-se a base «postgres» (existe por omissão); o servidor migra o esquema.
setsid timeout "$PRAZO" "$O/nats-server-v2.14.4-linux-amd64/nats-server" -js -a 127.0.0.1 -p $NATS_PORT \
  -sd "$O/run/nats" >"$O/logs/nats.log" 2>&1 < /dev/null & echo "nats $!" >> "$O/run/pids"
sleep 0.5
setsid timeout "$PRAZO" "$O/scrumlr-server" --port $HTTP_PORT --address 127.0.0.1 \
  --nats "nats://127.0.0.1:$NATS_PORT" \
  --database "postgres://admin@127.0.0.1:$PG_PORT/postgres?sslmode=disable" \
  --insecure --disable-check-origin --enable-swagger --join-rate-limit 1000 \
  --allow-anonymous-board-creation --allow-anonymous-custom-templates \
  >"$O/logs/scrumlr.log" 2>&1 < /dev/null & echo "scrumlr $!" >> "$O/run/pids"
for i in $(seq 1 100); do curl -sf "http://127.0.0.1:$HTTP_PORT/health" >/dev/null 2>&1 && break; sleep 0.2; done
curl -s -o /dev/null -w "health %{http_code}\n" "http://127.0.0.1:$HTTP_PORT/health"
cat "$O/run/pids"

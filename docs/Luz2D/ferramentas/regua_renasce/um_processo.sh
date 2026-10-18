#!/bin/bash
# Um processo da régua (W6c §2–3): 60 s de placa livre antes (prazo 1200 s), o app a 2560×1080 High, 6 000 quadros,
# `CUSTO=1 SERIE=1`, e os alheios a cada 0,5 s. uso: um_processo.sh <VAR=valor|-> <saida_dir>
set -u
VAR="$1"; SAIDA="$2"; AQUI="$(dirname "$0")"
RAIZ="$(cd "$AQUI/../../../.." && pwd)"
mkdir -p "$SAIDA"
livre=0; t0=$SECONDS
while [ $livre -lt 60 ]; do
  [ $((SECONDS - t0)) -gt 1200 ] && { echo "sem placa livre em 1200 s: $(bash "$AQUI/alheios.sh" | tr '\n' ' ')" > "$SAIDA/espera.txt"; exit 3; }
  if [ -z "$(bash "$AQUI/alheios.sh")" ]; then livre=$((livre + 2)); else livre=0; fi
  sleep 2
done
echo "esperou $((SECONDS - t0)) s" > "$SAIDA/espera.txt"
( for _ in $(seq 1 1600); do [ -e "$SAIDA/.fim" ] && break; bash "$AQUI/alheios.sh" | sed "s/^/$(date +%s) /" >> "$SAIDA/alheios.txt"; sleep 0.5; done ) &
AMOSTRA=$!
EXTRA=""
[ "$VAR" != "-" ] && EXTRA="$VAR"
uptime > "$SAIDA/load.txt"
bash "$RAIZ/docs/Luz2D/ferramentas/mede_custo_no_app.sh" 2560 1080 high 6000 "$SAIDA" PH2D_LIGHT2D_SERIE=1 $EXTRA > "$SAIDA/roteiro.txt" 2>&1
uptime >> "$SAIDA/load.txt"
touch "$SAIDA/.fim"; wait $AMOSTRA

#!/bin/bash
# Um processo da régua (W6c §2–3): 60 s de placa livre antes (prazo 1200 s), o app a 2560×1080 High, 6 000 quadros,
# `CUSTO=1 SERIE=1`, e os alheios a cada 0,5 s. uso: um_processo.sh <VAR=valor|-> <saida_dir>
set -u
VAR="$1"; SAIDA="$2"; AQUI="$(dirname "$0")"
RAIZ="$(cd "$AQUI/../../../.." && pwd)"
mkdir -p "$SAIDA"
# ⛔ Um detector que FALHA lê-se «limpo» (o stdout sai vazio): até 18/10 este roteiro chamava um
# `alheios.sh` que nunca entrou no repo, e a régua versionada aprovava tudo. Ele corre uma vez aqui e pára ALTO.
python3 -I "$AQUI/alheios.py" > /dev/null || { echo "RECUSA: o detector alheios.py falhou" > "$SAIDA/espera.txt"; exit 4; }
livre=0; t0=$SECONDS
while [ $livre -lt 60 ]; do
  [ $((SECONDS - t0)) -gt 1200 ] && { echo "sem placa livre em 1200 s: $(python3 -I "$AQUI/alheios.py" | tr '\n' ' ')" > "$SAIDA/espera.txt"; exit 3; }
  if [ -z "$(python3 -I "$AQUI/alheios.py")" ]; then livre=$((livre + 2)); else livre=0; fi
  sleep 2
done
echo "esperou $((SECONDS - t0)) s" > "$SAIDA/espera.txt"
( for _ in $(seq 1 1600); do [ -e "$SAIDA/.fim" ] && break; python3 -I "$AQUI/alheios.py" | sed "s/^/$(date +%s) /" >> "$SAIDA/alheios.txt"; sleep 0.5; done ) &
AMOSTRA=$!
EXTRA=""
[ "$VAR" != "-" ] && EXTRA="$VAR"
uptime > "$SAIDA/load.txt"
bash "$RAIZ/docs/Luz2D/ferramentas/mede_custo_no_app.sh" 2560 1080 high 6000 "$SAIDA" PH2D_LIGHT2D_SERIE=1 $EXTRA > "$SAIDA/roteiro.txt" 2>&1
uptime >> "$SAIDA/load.txt"
touch "$SAIDA/.fim"; wait $AMOSTRA

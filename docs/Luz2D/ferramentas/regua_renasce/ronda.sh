#!/bin/bash
# ⭐ W6c — a RODADA da régua do renascimento (handoff W6c §2–3): as variantes intercaladas em blocos rodados (cada
# bloco = uma permutação), um processo SUJO volta ao fim da fila, esperar a placa livre não conta, para com N limpos
# de cada (ou 40 medidos). Cada variante é uma atribuição de ambiente (`-` = nenhuma), lida em execução pelo MESMO
# binário (`target/smoke`, que o roteiro do repo recusa se o código for mais novo).
#   uso: bash ronda.sh <saida_dir> <N> <VAR=a> <VAR=b> ...   ex.: ronda.sh /tmp/r 6 PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0 -
#   depois: python3 analisa.py <saida_dir>
# Corre cada processo com `PH2D_GPU=1` (a vez da placa). O detector é `alheios.py`: a tabela INTEIRA do nvidia-smi
# (C e G), aceites só a área de trabalho e o que descende de `um_processo.sh`.
set -u
AQUI="$(cd "$(dirname "$0")" && pwd)"; RAIZ="$(cd "$AQUI/../../../.." && pwd)"
SAIDA="$1"; N="$2"; shift 2; VARS=("$@"); mkdir -p "$SAIDA"; LOG="$SAIDA/ronda.log"
fila=(); k=${#VARS[@]}
for b in $(seq 0 $((N - 1))); do for j in $(seq 0 $((k - 1))); do
  if [ $((b % 2)) -eq 0 ]; then fila+=("${VARS[$(((j + b / 2) % k))]}"); else fila+=("${VARS[$(((k - 1 - j + b / 2) % k))]}"); fi
done; done
declare -A limpos; for v in "${VARS[@]}"; do limpos[$v]=0; done
i=0; n=0
while [ $i -lt ${#fila[@]} ] && [ $n -lt 40 ]; do
  v="${fila[$i]}"; i=$((i + 1))
  [ "${limpos[$v]}" -ge "$N" ] && continue
  n=$((n + 1)); d="$SAIDA/$(printf '%02d' $n)_$(echo "$v" | tr -c 'A-Za-z0-9=\n' '_' | tail -c 24)"
  echo "$(date +%T) ▸ $n $v" >> "$LOG"
  (cd "$RAIZ" && PH2D_GPU=1 PH2D_GPU_ESPERA=1800 bash scripts/ph2d-run.sh bash "$AQUI/um_processo.sh" "$v" "$d") >> "$LOG" 2>&1
  if ! grep -q "série" "$d/app.log" 2>/dev/null; then
    echo "$(date +%T)   não mediu ($(cut -c1-80 "$d/espera.txt" 2>/dev/null)) → volta à fila" >> "$LOG"; fila+=("$v"); n=$((n - 1)); rm -rf "$d"
  elif [ -s "$d/alheios.txt" ]; then
    echo "$(date +%T)   sujo: $(cut -d' ' -f2 "$d/alheios.txt" | sort -u | tr '\n' ' ' | cut -c1-100) → volta à fila" >> "$LOG"; fila+=("$v")
  else
    limpos[$v]=$((limpos[$v] + 1)); echo "$(date +%T)   limpo · $v ${limpos[$v]}" >> "$LOG"
  fi
done
python3 "$AQUI/analisa.py" "$SAIDA" > "$SAIDA/resumo.txt" 2>&1; echo "$(date +%T) FIM" >> "$LOG"

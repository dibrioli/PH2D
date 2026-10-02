#!/bin/bash
# mede_sonda_das_estrelas.sh — a régua do doc 121 §9.7+: `sonda_relogio_das_estrelas_grandes` nos três
# arranjos (grandes esticadas · grandes conformes · densas da `=127`), nas placas pedidas, só com a
# máquina CALMA e com a placa pela porta da casa (exclusão + prazo) SÓ durante a corrida.
#
# uso (dentro da worktree):
#   B=$(cargo test -p ph2d-app-motion --lib --release --no-run 2>&1 \
#         | grep -o 'target/release/deps/ph2d_app_motion-[0-9a-f]*' | tail -1)
#   cp "$B" <copia>        # ⚠️ COPIE: o próximo build do mesmo perfil sobrescreve o mesmo nome
#   bash "docs/Motion Nodes/ferramentas/mede_sonda_das_estrelas.sh" <copia> <dir-de-saida>
# Ambiente: PLACAS="igpu rtx" · CORRIDAS=2 · BARRA=4 · SEGUIDAS=3 · HORAS=6
set -u
BIN="${1:?binario-da-sonda}"
OUT="${2:?dir-de-saida}"
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
PLACAS="${PLACAS:-igpu rtx}"
CORRIDAS="${CORRIDAS:-2}"
BARRA="${BARRA:-4}"
SEGUIDAS="${SEGUIDAS:-3}"
FIM=$(( $(date +%s) + ${HORAS:-6} * 3600 ))
FILTRO=sonda_relogio_das_estrelas_grandes
mkdir -p "$OUT"
[ -x "$BIN" ] || { echo "binario ausente: $BIN" >&2; exit 2; }
# Controlo positivo: o filtro tem de casar um teste DESTE binário, senão cada corrida mede nada.
"$BIN" --list --ignored 2>/dev/null | grep -q "$FILTRO" \
  || { echo "o filtro '$FILTRO' nao casa nenhum teste deste binario" >&2; exit 2; }
calma() {
  local ok=0 l1
  while [ "$ok" -lt "$SEGUIDAS" ]; do
    if [ "$(date +%s)" -ge "$FIM" ]; then echo "DESISTIU: nao acalmou" >> "$OUT/progresso.log"; exit 3; fi
    l1=$(cut -d' ' -f1 /proc/loadavg)
    if awk -v l="$l1" -v b="$BARRA" 'BEGIN { exit !(l <= b) }'; then ok=$((ok + 1)); else ok=0; fi
    [ "$ok" -lt "$SEGUIDAS" ] && sleep 20
  done
}
for placa in $PLACAS; do
  for arranjo in esticadas conformes densas; do
    corrida=1
    while [ "$corrida" -le "$CORRIDAS" ]; do
      kv=()
      [ "$placa" = igpu ] && kv+=(VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json)
      [ "$arranjo" = conformes ] && kv+=(PH2D_SONDA_MODO=conforme)
      [ "$arranjo" = densas ] && kv+=(PH2D_SONDA_DENSO=2)
      f="$OUT/${placa}_${arranjo}_${corrida}.txt"
      calma
      {
        echo "ANTES: $(cat /proc/loadavg)"
        PH2D_GPU=1 PH2D_GPU_ESPERA=1500 PH2D_PRAZO=600 bash "$RAIZ/scripts/ph2d-run.sh" \
          env "${kv[@]}" "$BIN" --ignored --nocapture --test-threads=1 "$FILTRO" 2>&1
        echo "DEPOIS: $(cat /proc/loadavg)"
      } > "$f"
      echo "$(date +%H:%M:%S) $placa $arranjo $corrida" >> "$OUT/progresso.log"
      corrida=$((corrida + 1))
    done
  done
done
# A tabela: a linha da sonda de cada corrida.
grep -H 'copias: placa' "$OUT"/*.txt | sed "s|$OUT/||" > "$OUT/tabela.txt"
echo FEITO >> "$OUT/progresso.log"

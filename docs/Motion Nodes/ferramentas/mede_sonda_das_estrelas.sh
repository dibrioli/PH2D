#!/bin/bash
# ⛔ SUBSTITUÍDA para comparar variantes (doc 121 §9.16, 05/10): use `mede_intercalado.sh` — todas as variantes e
# cenas num só processo, sem espera de calma (`37 s` contra `4`–`5 h`). Esta fica para a PAREDE por célula.
# mede_sonda_das_estrelas.sh — a régua do doc 121 §9.7+: `sonda_relogio_das_estrelas_grandes` nos três
# arranjos (grandes esticadas · grandes conformes · densas da `=127`), nas placas pedidas, só com a
# máquina CALMA e com a placa pela porta da casa (exclusão + prazo) SÓ durante a corrida.
#
# uso (dentro da worktree):
#   B=$(cargo test -p ph2d-app-motion --lib --release --no-run 2>&1 \
#         | grep -o 'target/release/deps/ph2d_app_motion-[0-9a-f]*' | tail -1)
#   cp "$B" <copia>        # ⚠️ COPIE: o próximo build do mesmo perfil sobrescreve o mesmo nome
#   bash "docs/Motion Nodes/ferramentas/mede_sonda_das_estrelas.sh" <copia> <dir-de-saida> [<outra-copia>…]
# Com mais de uma cópia (o binário de ANTES e o de DEPOIS), cada célula corre-as INTERCALADAS na mesma
# janela calma — o rótulo de cada ficheiro é o nome da cópia.
# Ambiente: PLACAS="igpu rtx" · CORRIDAS=2 · BARRA=4 · SEGUIDAS=3 · HORAS=6 · REPETE=6 ·
#           PERFIL=1 (`PH2D_FLUID_PROFILE=1`: 250 quadros e o relógio da placa por passe) ·
#           TRACEJADOS="0 1" (doc 121 §9.15: o contínuo e o tracejado na MESMA rodada intercalada;
#           sem a variável, o `PH2D_SONDA_TRACEJADO` herdado e o rótulo de sempre)
set -u
BIN="${1:?binario-da-sonda}"
OUT="${2:?dir-de-saida}"
BINS=("$BIN" "${@:3}")
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
PLACAS="${PLACAS:-igpu rtx}"
CORRIDAS="${CORRIDAS:-2}"
BARRA="${BARRA:-4}"
SEGUIDAS="${SEGUIDAS:-3}"
FIM=$(( $(date +%s) + ${HORAS:-6} * 3600 ))
FILTRO=sonda_relogio_das_estrelas_grandes
mkdir -p "$OUT"
for b in "${BINS[@]}"; do
  [ -x "$b" ] || { echo "binario ausente: $b" >&2; exit 2; }
  # Controlo positivo: o filtro tem de casar um teste DESTE binário, senão cada corrida mede nada.
  "$b" --list --ignored 2>/dev/null | grep -q "$FILTRO" \
    || { echo "o filtro '$FILTRO' nao casa nenhum teste de $b" >&2; exit 2; }
done
calma() {
  local ok=0 l1
  while [ "$ok" -lt "$SEGUIDAS" ]; do
    if [ "$(date +%s)" -ge "$FIM" ]; then echo "DESISTIU: nao acalmou" >> "$OUT/progresso.log"; exit 3; fi
    l1=$(cut -d' ' -f1 /proc/loadavg)
    if awk -v l="$l1" -v b="$BARRA" 'BEGIN { exit !(l <= b) }'; then ok=$((ok + 1)); else ok=0; fi
    [ "$ok" -lt "$SEGUIDAS" ] && sleep 20
  done
}
TRS=(${TRACEJADOS:-herdado})
for placa in $PLACAS; do
  for arranjo in esticadas conformes densas; do
   for tr in "${TRS[@]}"; do
    corrida=1
    while [ "$corrida" -le "$CORRIDAS" ]; do
      kv=()
      [ "$placa" = igpu ] && kv+=(VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json)
      [ "$arranjo" = conformes ] && kv+=(PH2D_SONDA_MODO=conforme)
      [ "$arranjo" = densas ] && kv+=(PH2D_SONDA_DENSO=2)
      [ "${PERFIL:-0}" = 1 ] && kv+=(PH2D_FLUID_PROFILE=1)
      sufixo=""
      [ "$tr" != herdado ] && kv+=(PH2D_SONDA_TRACEJADO="$tr") && sufixo="_tr$tr"
      for b in "${BINS[@]}"; do
        rotulo=""
        [ "${#BINS[@]}" -gt 1 ] && rotulo="$(basename "$b")_"
        f="$OUT/${rotulo}${placa}_${arranjo}${sufixo}_${corrida}.txt"
        # ⛔ 04/10 (doc 121 §9.14): a placa presa por OUTRA linha (um arnês de mutação, 25 min) fazia a
        # porta desistir e a célula saía VAZIA — a rodada seguia com um buraco na intercalação. ⇒ uma
        # célula sem a linha da sonda repete-se (com nova calma), até `REPETE` vezes.
        tentativa=0
        while :; do
          calma
          {
            echo "ANTES: $(cat /proc/loadavg)"
            PH2D_GPU=1 PH2D_GPU_ESPERA=1500 PH2D_PRAZO=600 bash "$RAIZ/scripts/ph2d-run.sh" \
              env "${kv[@]}" "$b" --ignored --nocapture --test-threads=1 "$FILTRO" 2>&1
            echo "DEPOIS: $(cat /proc/loadavg)"
          } > "$f"
          grep -q 'copias: placa' "$f" && break
          tentativa=$((tentativa + 1))
          echo "$(date +%H:%M:%S) ${rotulo}$placa $arranjo$sufixo $corrida VAZIA (tentativa $tentativa)" >> "$OUT/progresso.log"
          [ "$tentativa" -ge "${REPETE:-6}" ] && break
        done
        echo "$(date +%H:%M:%S) ${rotulo}$placa $arranjo$sufixo $corrida" >> "$OUT/progresso.log"
      done
      corrida=$((corrida + 1))
    done
   done
  done
done
# A tabela: a linha da sonda de cada corrida.
grep -H 'copias: placa' "$OUT"/*.txt | sed "s|$OUT/||" > "$OUT/tabela.txt"
echo FEITO >> "$OUT/progresso.log"

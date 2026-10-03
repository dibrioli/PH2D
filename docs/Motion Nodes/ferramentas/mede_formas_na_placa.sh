#!/bin/bash
# mede_formas_na_placa.sh — a medição de FECHO do doc 121 (W5): a MESMA cena com e sem
# `PH2D_FORMAS_NA_PLACA`, nas duas placas, numa tela virtual e só com a máquina CALMA.
#
# PORQUÊ UM A/B NA MESMA BUILD: `PH2D_FORMAS_NA_PLACA=0` devolve as DUAS rotas (a do cozimento e a
# do desenho) ao caminho de antes da W1, byte a byte (doc 121 §7–§8). Logo a coluna «sem» é a
# partida (a W0 que a placa ocupada adiou) e a coluna «com» é o fecho — com o MESMO binário,
# a MESMA sessão e a MESMA carga, que é o único A/B que esta workstation deixa fazer.
#
# uso (dentro da worktree, com o `release` já construído):
#   bash "docs/Motion Nodes/ferramentas/mede_formas_na_placa.sh" <dir-de-saida>
# Ambiente: CELULAS (lista «placa:cena:n», ver abaixo) · BARRA=4 · SEGUIDAS=3 · ESPERA=40 · HORAS=6 ·
#           FORMAS="0 1" (só "1" re-mede o lado da placa contra um «sem» já medido na mesma sessão)
#
# ⚠️ Cada célula lê as TRÊS últimas janelas `[frame]` (120 quadros cada) depois de a população
# encher — a régua do doc 120 §3. ⚠️ A carga ANTES e DEPOIS vai ao lado de cada célula.
set -u
OUT="${1:?dir-de-saida}"
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
BARRA="${BARRA:-4}"
SEGUIDAS="${SEGUIDAS:-3}"
ESPERA="${ESPERA:-40}"
IGPU="VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json"
CELULAS="${CELULAS:-rtx:17:4096 rtx:17:16384 rtx:17:32768 igpu:17:4096 igpu:17:16384 igpu:17:32768 rtx:127:0 igpu:127:0}"
mkdir -p "$OUT"
FIM=$(( $(date +%s) + ${HORAS:-6} * 3600 ))
calma() {
  local ok=0
  while [ "$ok" -lt "$SEGUIDAS" ]; do
    if [ "$(date +%s)" -ge "$FIM" ]; then echo "DESISTIU: nao acalmou" >> "$OUT/progresso.log"; exit 3; fi
    l1=$(cut -d' ' -f1 /proc/loadavg)
    if awk -v l="$l1" -v b="$BARRA" 'BEGIN { exit !(l <= b) }'; then ok=$((ok + 1)); else ok=0; fi
    [ "$ok" -lt "$SEGUIDAS" ] && sleep 20
  done
}
for cel in $CELULAS; do
  IFS=: read -r placa cena n <<< "$cel"
  for formas in ${FORMAS:-0 1}; do
    kv="PH2D_FLUID_PROFILE=1 PH2D_FORMAS_NA_PLACA=$formas"
    if [ "$cena" = 17 ]; then
      kv="$kv PH2D_MOTION_OBJ_SMOKE=17 PH2D_TECTO_FORMA=1 PH2D_TECTO_N=$n"
    else
      kv="$kv PH2D_GPU_COOK_DEMO=$cena PH2D_TRACO_ESTICADO_DENSO=1"
    fi
    [ "$placa" = igpu ] && kv="$kv $IGPU"
    nome="$OUT/${placa}_c${cena}_n${n}_formas${formas}"
    calma
    antes=$(cat /proc/loadavg)
    # ⚠️ A PLACA passa pela porta da casa (exclusão + prazo) SÓ durante a foto — segurá-la durante
    # a espera pela calma bloquearia as outras linhas horas a fio.
    FOTO_PERFIL=release PH2D_GPU=1 PH2D_GPU_ESPERA=1500 PH2D_PRAZO=600 \
      bash "$RAIZ/scripts/ph2d-run.sh" bash "$RAIZ/docs/Components/ferramentas/fotografa_cena.sh" \
      "$kv" 1930 1040 "$nome.png" "$ESPERA" > "$nome.foto.txt" 2>&1
    depois=$(cat /proc/loadavg)
    {
      echo "celula=$cel formas=$formas"
      echo "ANTES: $antes"
      echo "DEPOIS: $depois"
      # a ROTA primeiro (sai uma vez, no arranque): sem ela a coluna podia medir outro caminho
      grep -E '^\[formas\]|^\[motion-route\]' "$nome.log" | sort -u
      grep -E '^\[frame\] total=|MOTION \(cozer' "$nome.log" | tail -6
    } > "$nome.txt"
    echo "$(date +%H:%M:%S) $cel formas=$formas" >> "$OUT/progresso.log"
  done
done
echo FEITO >> "$OUT/progresso.log"

#!/usr/bin/env bash
# medir_quando_calmo.sh — espera a máquina ACALMAR e só então corre uma sonda de preço.
#
# PORQUÊ: nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`
# (CLAUDE.md §5.0). Em 2026-09-10 a medição do ciclo 5 esperou 25 min À MÃO e desistiu; em
# 2026-09-13 a calma durou DOIS minutos entre builds de outras linhas — e só um vigia que
# amostra sozinho os apanha (doc 108 W5).
#
# USO (dentro da worktree):
#   B=$(cargo test -p ph2d-app-motion --lib --release --no-run 2>&1 \
#         | grep -o 'target/release/deps/ph2d_app_motion-[0-9a-f]*' | tail -1)
#   cp "$B" /tmp/sonda-release
#   bash "docs/Motion Nodes/ferramentas/medir_quando_calmo.sh" /tmp/sonda-release /tmp/saida measure_the_sim_group
#
# ⚠️ COPIE o binário antes de esperar: o nome dele sai de um hash de METADADOS, não do
# conteúdo, então o próximo build do mesmo perfil SOBRESCREVE o mesmo ficheiro — e a espera
# pode durar horas, com o código a mudar por baixo.
#
# Ambiente (todos opcionais):
#   LADOS="320 1000"  cada um vai à sonda como PH2D_LADO
#   CORRIDAS=2        corridas por lado (duas leituras que concordam valem mais que uma)
#   BARRA=4.5         teto do load 1-min, abaixo da barra de 5 do §5.0
#   SEGUIDAS=4        amostras consecutivas abaixo da barra
#   INTERVALO=30      segundos entre amostras
#   HORAS=6           desiste depois disto (e diz que desistiu)
set -u
if [ "$#" -lt 3 ]; then
  echo "uso: $0 <binario-de-teste-release> <dir-de-saida> <filtro-do-teste>" >&2
  exit 2
fi
BIN="$1"; OUT="$2"; FILTRO="$3"
LADOS="${LADOS:-320 1000}"
CORRIDAS="${CORRIDAS:-2}"
BARRA="${BARRA:-4.5}"
SEGUIDAS="${SEGUIDAS:-4}"
INTERVALO="${INTERVALO:-30}"
HORAS="${HORAS:-6}"

mkdir -p "$OUT"
[ -x "$BIN" ] || { echo "binario ausente ou nao executavel: $BIN" >&2; exit 2; }
# Controlo positivo: o filtro tem de casar um teste DENTRO deste binário, senão cada corrida
# imprime «0 passed» e a espera inteira mede nada.
if ! "$BIN" --list --ignored 2>/dev/null | grep -q "$FILTRO"; then
  echo "o filtro '$FILTRO' nao casa nenhum teste ignorado deste binario" >&2
  exit 2
fi

fim=$(( $(date +%s) + HORAS * 3600 ))
ok=0
while :; do
  l1=$(cut -d' ' -f1 /proc/loadavg)
  if awk -v l="$l1" -v b="$BARRA" 'BEGIN { exit !(l <= b) }'; then ok=$((ok + 1)); else ok=0; fi
  echo "$(date +%H:%M:%S) load1=$l1 seguidas=$ok" >> "$OUT/espera.log"
  [ "$ok" -ge "$SEGUIDAS" ] && break
  if [ "$(date +%s)" -ge "$fim" ]; then
    echo "DESISTIU: a maquina nao acalmou em ${HORAS} h" | tee -a "$OUT/espera.log"
    exit 3
  fi
  sleep "$INTERVALO"
done

for lado in $LADOS; do
  corrida=1
  while [ "$corrida" -le "$CORRIDAS" ]; do
    f="$OUT/lado${lado}_corrida${corrida}.txt"
    {
      echo "ANTES: $(cat /proc/loadavg)"
      PH2D_LADO="$lado" "$BIN" --ignored --nocapture --test-threads=1 "$FILTRO" 2>&1
      echo "DEPOIS: $(cat /proc/loadavg)"
    } > "$f"
    echo "$(date +%H:%M:%S) lado=$lado corrida=$corrida -> $f" >> "$OUT/espera.log"
    corrida=$((corrida + 1))
  done
done
echo "FEITO" >> "$OUT/espera.log"

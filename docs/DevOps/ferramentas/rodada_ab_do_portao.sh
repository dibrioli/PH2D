#!/usr/bin/env bash
# A rodada A × B do portão (W4, VELOCIDADE_DOS_PORTOES.md §2.6): o diff do Quadro d10a4ee617 (ph2d-app-board),
# 4 variantes por env sem recompilar, rondas em ordem rodada; resumo = o MÍNIMO (MEDIR_VELOCIDADE.md).
#   bash docs/DevOps/ferramentas/rodada_ab_do_portao.sh <pasta de saída> [rondas=3]
set -uo pipefail
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
OUT="$1"; RONDAS="${2:-3}"
LINHA="$(basename "$PWD" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9' '_' | sed 's/_*$//')"
mkdir -p "$OUT"
REDES=' + test(/^transform_determinism::/) + test(/^architecture_workspace_file_loc_cap::/) + test(/^file_loc_caps::/) + test(/^architecture_the_shell_only_shrinks::/) + test(/^architecture_no_orphan_source_file::/)'
EXPR_PACOTE="$(PH2D_IMPACTO=pacote python3 scripts/impacto-preciso.py ph2d-app-board)$REDES"
EXPR_PRECISO="$(python3 scripts/impacto-preciso.py ph2d-app-board)$REDES"
printf '%s\n' "$EXPR_PACOTE" > "$OUT/expr_pacote"; printf '%s\n' "$EXPR_PRECISO" > "$OUT/expr_preciso"
# variante: nome | env
declare -A ENVS=(
  [antes]="PH2D_IMPACTO=pacote PH2D_PLACA=principal PH2D_VEZ_ARVORE=1"
  [depois]=""
  [so_pacote]="PH2D_IMPACTO=pacote"
  [so_principal]="PH2D_PLACA=principal"
)
ORDEM=(antes depois so_pacote so_principal)
# aquecimento: a compilação ci-test (as duas listas, não conta)
cargo nextest list -E "$EXPR_PACOTE" --cargo-profile ci-test >/dev/null 2>"$OUT/aquece.err" || { echo "aquecimento falhou" ; tail "$OUT/aquece.err"; exit 2; }
echo "ronda variante segundos testes_corridos placa_principal_s fila_testes_s load_ini load_fim rc" > "$OUT/tabela.txt"
for r in $(seq 1 "$RONDAS"); do
  n=${#ORDEM[@]}
  for i in $(seq 0 $((n-1))); do
    v=${ORDEM[$(( (i + r - 1) % n ))]}
    e=${ENVS[$v]}
    case "$e" in *pacote*) X="$EXPR_PACOTE";; *) X="$EXPR_PRECISO";; esac
    l0=$(cut -d' ' -f1 /proc/loadavg)
    log="$OUT/r${r}_${v}.log"
    # amostrador: segundos em que ESTA árvore segura cada fechadura
    ( p=0; t=0; while :; do
        grep -qs "linha $LINHA " "${XDG_RUNTIME_DIR:-/tmp}/ph2d-gpu.lock.dono" && p=$((p+1))
        grep -qs "linha $LINHA " "${XDG_RUNTIME_DIR:-/tmp}/ph2d-gpu-testes.lock.dono" && t=$((t+1))
        echo "$p $t" > "$OUT/.amostra"; sleep 1; done ) &
    amo=$!
    t0=$(date +%s.%N)
    env $e PH2D_PRAZO=3600 bash scripts/ph2d-run.sh cargo nextest run --no-fail-fast -E "$X" --cargo-profile ci-test > "$log" 2>&1
    rc=$?
    t1=$(date +%s.%N)
    kill $amo 2>/dev/null; wait $amo 2>/dev/null
    read -r ps ts < "$OUT/.amostra"
    l1=$(cut -d' ' -f1 /proc/loadavg)
    nt=$(grep -oE 'Summary \[.*\] +[0-9]+ tests? run' "$log" | grep -oE '[0-9]+ tests? run' | grep -oE '^[0-9]+' | tail -1)
    LC_NUMERIC=C printf '%s %s %.1f %s %s %s %s %s %s\n' "$r" "$v" "$(echo "$t1 - $t0" | bc)" "${nt:-?}" "$ps" "$ts" "$l0" "$l1" "$rc" | tee -a "$OUT/tabela.txt"
  done
done
echo FIM

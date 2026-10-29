#!/usr/bin/env bash
# fila-pesada.sh — o pesado DE VERDADE de uma onda (regerar presets, bancada inteira) corre 1× no fim, numa
# fila desta LINHA, com a régua CONGELADA antes (item I da comissão 29/10; REGRAS_DE_CUSTO §METODOLOGIA 4).
# Medido na J6 (28/10): Blender/Smear regerados 2× porque a régua mudou a meio; 8 Oil/* = 2 h 45.
# ⛔ A fila é por LINHA (o `git-dir` do worktree), não global — a fila global de linhas pesadas (J) foi
# recusada pelo dono em 30/10.
#
#   bash scripts/fila-pesada.sh pôr '<comando>'      # qualquer agente (o hook do prazo manda para aqui)
#   bash scripts/fila-pesada.sh ver
#   bash scripts/fila-pesada.sh congelar <ficheiro…>  # o coordenador: a régua (testes/gerador) fica fixa
#   bash scripts/fila-pesada.sh correr               # o coordenador: recusa se a régua mudou; corre em série
set -uo pipefail
raiz="$(git rev-parse --show-toplevel)" || exit 2
d="${PH2D_FILA_DIR:-$(git rev-parse --git-dir)/ph2d-fila-pesada}"
mkdir -p "$d"
fila="$d/fila"; regua="$d/regua.sha256"

case "${1:-ver}" in
  pôr | por)
    shift; [ $# -gt 0 ] || { echo "uso: fila-pesada.sh pôr '<comando>'" >&2; exit 2; }
    grep -qxF -- "$*" "$fila" 2>/dev/null && { echo "· já na fila: $*"; exit 0; }
    printf '%s\n' "$*" >>"$fila"; echo "✓ na fila ($(wc -l <"$fila")): $*" ;;
  ver)
    if [ -s "$fila" ]; then nl -ba "$fila"; else echo "(fila vazia)"; fi
    if [ -f "$regua" ]; then echo "régua congelada:"; sed 's/^/  /' "$regua"; else echo "(régua por congelar)"; fi ;;
  congelar)
    shift; [ $# -gt 0 ] || { echo "uso: fila-pesada.sh congelar <ficheiro da régua…>" >&2; exit 2; }
    (cd "$raiz" && sha256sum -- "$@") >"$regua" || exit 2
    echo "✓ régua congelada ($# ficheiro(s))" ;;
  correr)
    [ -s "$fila" ] || { echo "(fila vazia)"; exit 0; }
    [ -f "$regua" ] || { echo "✗ congele a régua primeiro: fila-pesada.sh congelar <ficheiros>" >&2; exit 1; }
    (cd "$raiz" && sha256sum --quiet -c "$regua") || { echo "✗ a régua MUDOU depois de congelada — reveja e congele de novo antes de correr" >&2; exit 1; }
    n=0; falhou=0
    while IFS= read -r c; do
      n=$((n + 1)); echo "▶ [$n] $c"
      (cd "$raiz" && eval "$c") || { echo "✗ [$n] falhou"; falhou=1; }
    done <"$fila"
    [ "$falhou" = 0 ] && : >"$fila"
    exit "$falhou" ;;
  *) echo "uso: fila-pesada.sh pôr|ver|congelar|correr" >&2; exit 2 ;;
esac

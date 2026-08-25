#!/usr/bin/env bash
# cleanroom-sweep.sh — o instrumento da parede clean-room (SKILL_Cleanroom §7.1).
# Varre artefatos destinados aos olhos do Implementador contra a VASSOURA do alvo.
#
# Uso:
#   bash scripts/cleanroom-sweep.sh <VASSOURA_b64.txt> <path> [path...]
#   bash scripts/cleanroom-sweep.sh <VASSOURA_b64.txt> --git-history [<pathspec>]
#
# A vassoura tem UMA entrada por linha, codificada em base64 — decodificada SÓ
# aqui, EM MEMÓRIA (nunca em arquivo): um grep acidental do Implementador não
# casa por construção, e nada do alvo toca /tmp.
#
# O que varre:
#   modo paths:        (1) conteúdo de arquivos de texto  (2) strings de binários
#                      (3) NOMES de arquivos e pastas
#   modo --git-history (4) mensagens de commit (--all)    (5) patches (git log -p)
#
# Exit: 0 = limpo · 1 = achado (cada hit impresso) · 2 = uso errado.
set -u

usage() {
  sed -n '2,14p' "$0" >&2
  exit 2
}

[ $# -ge 2 ] || usage
VAS="$1"
shift
[ -f "$VAS" ] || { echo "✗ vassoura não encontrada: $VAS" >&2; exit 2; }

# Decodifica a vassoura em memória. Entradas vazias ou não-base64 são recusadas
# alto (uma linha de padrão vazia faria o grep casar TUDO — pior que falhar).
PATTERNS=""
lineno=0
while IFS= read -r line || [ -n "$line" ]; do
  lineno=$((lineno + 1))
  [ -n "$line" ] || continue
  decoded="$(printf '%s' "$line" | base64 -d 2>/dev/null)" || decoded=""
  if [ -z "$decoded" ]; then
    echo "✗ vassoura linha $lineno: não é base64 válido (ou decodifica vazio)" >&2
    exit 2
  fi
  PATTERNS="${PATTERNS}${decoded}
"
done < "$VAS"
[ -n "$PATTERNS" ] || { echo "✗ vassoura vazia" >&2; exit 2; }

hits=0
patfile() { printf '%s' "$PATTERNS"; }

# ⚠️ NUNCA `comando | report`: hits=1 num subshell de pipeline se perde e o
# sweep sairia verde com achado impresso (a doença "pipe mascara exit code").
# Capture SEMPRE em variável e chame report com o texto como argumento.
report() { # $1 = rótulo · $2 = hits já filtrados ("" = limpo)
  if [ -n "$2" ]; then
    hits=1
    printf '✗ %s:\n%s\n' "$1" "$2"
  fi
}

if [ "${1:-}" = "--git-history" ]; then
  shift
  # pathspec opcional; ⚠️ sem argumento, NENHUM `-- .` default: um pathspec
  # filtra por commits que TOCAM o path, e mensagens (e commits vazios)
  # escapariam do sweep — o repo inteiro é o default correto.
  spec=()
  [ $# -gt 0 ] && spec=(-- "$@")
  out="$(git log --all --format='%H %B' ${spec[@]+"${spec[@]}"} 2>/dev/null \
    | grep -F -f <(patfile))" || true
  report "mensagens de commit (histórico)" "$out"
  out="$(git log -p --all ${spec[@]+"${spec[@]}"} 2>/dev/null \
    | grep -F -f <(patfile))" || true
  report "patches do histórico" "$out"
else
  for p in "$@"; do
    [ -e "$p" ] || { echo "✗ path não existe: $p" >&2; exit 2; }
    # (3) nomes de arquivos/pastas
    out="$(find "$p" -name .git -prune -o -print 2>/dev/null \
      | grep -F -f <(patfile))" || true
    report "NOMES sob $p" "$out"
    # (1)+(2) conteúdo, arquivo a arquivo
    while IFS= read -r -d '' f; do
      # a própria vassoura é base64 — o decode nunca casa nela; pular por clareza
      [ "$(readlink -f "$f")" = "$(readlink -f "$VAS")" ] && continue
      if grep -qI . "$f" 2>/dev/null; then
        out="$(grep -HnF -f <(patfile) -- "$f" 2>/dev/null)" || true
        report "conteúdo" "$out"
      else
        if strings -n 6 -- "$f" 2>/dev/null | grep -qF -f <(patfile); then
          hits=1
          printf '✗ binário contém entrada da vassoura: %s\n' "$f"
        fi
      fi
    done < <(find "$p" -name .git -prune -o -type f -print0 2>/dev/null)
  done
fi

if [ "$hits" -eq 0 ]; then
  echo "✓ sweep limpo (vassoura: $(printf '%s' "$PATTERNS" | grep -c .) entradas)"
  exit 0
fi
exit 1

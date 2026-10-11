#!/bin/bash
# the_other_media_are_untouched — a metade da cerca que lê FICHEIROS (a outra metade é o teste
# `os_outros_meios_nao_mudam`, que lê PIXELS). Decisão do dono 07/10: Digital, Watercolor e Impasto
# não mudam um byte; só o Wet Paint evolui.
#
# Contra o merge-base com `main`, os ficheiros dos outros meios e do carimbo partilhado só podem
# GANHAR linhas (campo/porta append-only usado só pelo Wet Paint): uma linha removida ou alterada
# reprova. As linhas acrescentadas imprimem-se para quem fecha a linha as ler.
#
# uso: bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh   (da raiz da worktree)
# exit 0 = verde · 1 = um ficheiro protegido perdeu/alterou linhas
set -u
base=$(git merge-base main HEAD) || { echo "sem merge-base com main" >&2; exit 2; }
padrao='^crates/(ph2d-tool-painter|ph2d-painter-brush|ph2d-render|ph2d-app-painter|ph2d-panel-painter-layers)/.*(/(watercolor|impasto|substrate)_[^/]*|/(watercolor|impasto|substrate)_[^/]*/.*|/dynamics\.rs|/spec\.rs|/dab\.rs|/stamp_route\.rs)$'
falha=0
n=0
# As ISENÇÕES (uma por linha: caminho, o número EXATO de linhas removidas/alteradas, a razão): uma linha a
# mais ou a menos volta a reprovar. Só para linhas que testam o Wet Paint e moram num ficheiro protegido —
# ou, por ordem do dono (11/10), para mudar só a FORMA DE CORRER um teste de outro meio (as mesmas contas e
# asserções; nenhum byte do produto).
isencoes="$(dirname "$0")/cerca_isencoes.tsv"
while IFS=$'\t' read -r mais menos f; do
  [[ "$f" =~ $padrao ]] || continue
  n=$((n + 1))
  isento=$(awk -F'\t' -v f="$f" -v n="$menos" '$1 == f && $2 == n { print $3 }' "$isencoes" 2>/dev/null)
  if [ "$menos" != "0" ] && [ -n "$isento" ]; then
    echo "· $f: $menos linha(s) alterada(s) ISENTAS (+$mais) — $isento — leia:"
    git diff "$base" -- "$f" | grep '^[-+][^-+]' | sed 's/^/    /'
  elif [ "$menos" != "0" ]; then
    echo "✗ $f: $menos linha(s) removida(s)/alterada(s) (+$mais)"
    falha=1
  else
    echo "· $f: só acrescenta (+$mais) — leia:"
    git diff "$base" -- "$f" | grep '^+[^+]' | sed 's/^/    /'
  fi
done < <(git diff --numstat "$base")
echo "the_other_media_are_untouched: base ${base:0:9}, $n ficheiro(s) protegido(s) no diff — $([ $falha = 0 ] && echo VERDE || echo VERMELHO)"
exit $falha

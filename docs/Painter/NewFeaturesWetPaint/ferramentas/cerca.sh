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
while IFS=$'\t' read -r mais menos f; do
  [[ "$f" =~ $padrao ]] || continue
  n=$((n + 1))
  if [ "$menos" != "0" ]; then
    echo "✗ $f: $menos linha(s) removida(s)/alterada(s) (+$mais)"
    falha=1
  else
    echo "· $f: só acrescenta (+$mais) — leia:"
    git diff "$base" -- "$f" | grep '^+[^+]' | sed 's/^/    /'
  fi
done < <(git diff --numstat "$base")
echo "the_other_media_are_untouched: base ${base:0:9}, $n ficheiro(s) protegido(s) no diff — $([ $falha = 0 ] && echo VERDE || echo VERMELHO)"
exit $falha

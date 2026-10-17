#!/usr/bin/env bash
# Corre o oráculo da lista de completação (CodeEdit headless) sobre completar.txt e grava a fixtura
# COM CABEÇALHO que o gate `completar::a_ordem_da_lista_e_a_do_godot` de ph2d-app-code lê.
# Uso: bash corre_completar.sh [saída]
set -euo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/crates/ph2d-app-code/tests/it/fixtures/godot_completar.txt}
PROJ=$(mktemp -d)
trap 'rm -rf "$PROJ"' EXIT
touch "$PROJ/project.godot"
cp "$AQUI/completar.gd" "$PROJ/"
if ! timeout 60 godot --headless --path "$PROJ" -s completar.gd -- "$AQUI/completar.txt" "$PROJ/out.txt" \
    >"$PROJ/log.txt" 2>&1; then
    cat "$PROJ/log.txt"
    exit 1
fi
LICENCA=$(LC_ALL=C pacman -Qi godot | grep -E '^(Name|Version|Licenses) ' | sed 's/ *: /: /' | paste -sd ';' -)
{
    echo "# Saída do oráculo Godot sobre docs/CodeEditor/ferramentas/godot_oraculo/completar.txt (entrada NOSSA)."
    echo "# NÃO edite à mão — regrave: bash docs/CodeEditor/ferramentas/godot_oraculo/corre_completar.sh"
    echo "# godot --version: $(godot --version)"
    echo "# pacman -Qi godot: $LICENCA"
    echo "# data: $(date -u +%Y-%m-%dT%H:%MZ)"
    echo "# comando: timeout 60 godot --headless --path <projeto temporário> -s completar.gd -- completar.txt <saída>"
    echo "# CodeEdit: code_completion_enabled=true; texto \"ph2d.<escrito>\", cursor no fim; todos os candidatos"
    echo "#   KIND_FUNCTION (o mesmo lugar), na ordem de completar.txt."
    echo "# nomes: $(grep '^nomes:' "$AQUI/completar.txt" | cut -d: -f2- | sed 's/^ *//')"
    echo "# Formato: <escrito>\\t<mostrados pela ordem, separados por espaço>"
    cat "$PROJ/out.txt"
} >"$SAIDA"
echo "gravado: $SAIDA"

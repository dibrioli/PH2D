#!/usr/bin/env bash
# Corre o oráculo Godot (CodeEdit headless) sobre casos.txt e grava a fixtura COM CABEÇALHO que o
# gate `autoedit::cada_caso_do_godot` de ph2d-code-syntax lê. Uso: bash corre.sh [saída]
set -euo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/crates/ph2d-code-syntax/tests/it/fixtures/godot_autoedit.txt}
PROJ=$(mktemp -d)
trap 'rm -rf "$PROJ"' EXIT
touch "$PROJ/project.godot"
cp "$AQUI/oraculo.gd" "$PROJ/"
if ! timeout 60 godot --headless --path "$PROJ" -s oraculo.gd -- "$AQUI/casos.txt" "$PROJ/out.txt" \
    >"$PROJ/log.txt" 2>&1; then
    cat "$PROJ/log.txt"
    exit 1
fi
LICENCA=$(LC_ALL=C pacman -Qi godot | grep -E '^(Name|Version|Licenses) ' | sed 's/ *: /: /' | paste -sd ';' -)
{
    echo "# Saída do oráculo Godot sobre docs/CodeEditor/ferramentas/godot_oraculo/casos.txt (entrada NOSSA)."
    echo "# NÃO edite à mão — regrave: bash docs/CodeEditor/ferramentas/godot_oraculo/corre.sh"
    echo "# godot --version: $(godot --version)"
    echo "# pacman -Qi godot: $LICENCA"
    echo "# data: $(date -u +%Y-%m-%dT%H:%MZ)"
    echo "# comando: timeout 60 godot --headless --path <projeto temporário> -s oraculo.gd -- casos.txt <saída>"
    echo "# CodeEdit: auto_brace_completion_enabled=true, indent_automatic=true (prefixos de fábrica \":\" \"{\" \"[\" \"(\");"
    echo "#   delimitadores do Luau: strings \" e ', comentário -- (de linha)."
    echo "# map: o mapa abridor GDScript→Luau é NOSSO — o \":\" do GDScript vira \" then\" / \" do\" / \"function f()\";"
    echo "#   o teste aplica-o ao texto de entrada E à saída do Godot antes de correr a nossa lei."
    echo "# Marcadores: | = cursor, ^ = âncora da seleção; escapes \\n \\t \\\\."
    cat "$PROJ/out.txt"
} >"$SAIDA"
echo "gravado: $SAIDA ($(grep -c '^== ' "$SAIDA") casos)"

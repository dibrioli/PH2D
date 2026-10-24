#!/usr/bin/env bash
# Oráculo «uma referência a um nó no TOPO do script, fora do @export/@onready, avisa?» (CodeEditor §10 do
# handoff D5_8B_8C_1B, 24/10). O Godot 4.7.2 (MIT, `pacman -Qi godot`) CORRIDO em `--headless --check-only`
# sobre cada caso de `casos/`; grava em saida.txt o que ele diz de cada um.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_objeto_no_topo/corre.sh
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_objeto.XXXX)
P=$T/proj
mkdir -p "$P" "$T/home" "$T/cfg" "$T/data" "$T/cache"
printf 'config_version=5\n\n[application]\nconfig/name="oraculo_objeto"\n' >"$P/project.godot"
cp "$AQUI"/casos/*.gd "$P/"
G=(env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 XDG_CONFIG_HOME="$T/cfg"
  XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache")
"${G[@]}" godot --headless --path "$P" --import >/dev/null 2>&1
{
  echo "# Oráculo: Godot $("${G[@]}" godot --version 2>/dev/null | head -1) · godot --headless --check-only --script <caso>"
  echo "# Gerado por corre.sh em $(date -I). Um bloco por caso: o texto do caso, depois o que o Godot disse."
  for f in "$AQUI"/casos/*.gd; do
    n=$(basename "$f")
    echo
    echo "== $n"
    sed 's/^/   | /' "$f"
    "${G[@]}" timeout 60 godot --headless --path "$P" --check-only --script "res://$n" 2>&1 \
      | grep -v -E '^(Godot Engine|$)' | sed "s#$P#res:/#g"
    echo "   exit=${PIPESTATUS[0]}"
  done
} >"$AQUI/saida.txt"
rm -rf "$T"

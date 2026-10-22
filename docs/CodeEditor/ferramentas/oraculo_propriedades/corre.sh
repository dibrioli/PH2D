#!/usr/bin/env bash
# Oráculo «propriedades por nome» (CodeEditor): o Godot 4.7 (MIT, `pacman -Qi godot`) CORRIDO sem janela
# (`--headless --script`), nunca lido (CLAUDE.md §0.9). Pergunta a Sprite2D, Camera2D e a um Node2D com script
# (`heroi.gd`, @export) o que get/set/get_indexed/get_property_list/property_get_revert respondem, e o que o
# motor imprime em stderr. Corre DUAS vezes e grava o diff entre as corridas no cabeçalho.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_propriedades/corre.sh
# Grava saida.jsonl (cabeçalho «#» + um caso por linha) ao lado.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_propriedades.XXXX)
trap 'rm -rf "$T"' EXIT
P=$T/proj
mkdir -p "$P" "$T/home" "$T/cfg" "$T/data" "$T/cache"
cp "$AQUI/probe.gd" "$AQUI/heroi.gd" "$P/"
printf 'config_version=5\n\n[application]\nconfig/name="oraculo_propriedades"\nconfig/features=PackedStringArray("4.7")\n' >"$P/project.godot"
printf '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#4a8"/></svg>\n' >"$P/icon.svg"
G=(env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 XDG_CONFIG_HOME="$T/cfg" XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache")
# importa o icon.svg (sem isto load("res://icon.svg") não acha o recurso importado)
timeout 120 "${G[@]}" godot --headless --editor --quit --path "$P" >"$T/import.log" 2>&1 || { tail -20 "$T/import.log" >&2; exit 1; }
CMD='godot --headless --path <proj> --script res://probe.gd'
for i in 1 2; do
  timeout 120 "${G[@]}" godot --headless --path "$P" --script res://probe.gd >"$T/cru$i.txt" 2>&1
  echo "$?" >"$T/rc$i"
  python3 -I "$AQUI/resume.py" "$T/cru$i.txt" >"$T/casos$i.jsonl"
done
# (b2) o RÓTULO que o Inspector mostra: um EditorPlugin (`rotulos.gd`) no editor headless lê cada EditorProperty.
P2=$T/proj_rot
mkdir -p "$P2/addons/rotulos"
cp "$AQUI/rotulos.gd" "$P2/addons/rotulos/"; cp "$AQUI/heroi.gd" "$P2/"
printf '[plugin]\nname="rotulos"\ndescription=""\nauthor=""\nversion="1"\nscript="rotulos.gd"\n' >"$P2/addons/rotulos/plugin.cfg"
printf 'config_version=5\n\n[application]\nconfig/name="rot"\nconfig/features=PackedStringArray("4.7")\n\n[editor_plugins]\nenabled=PackedStringArray("res://addons/rotulos/plugin.cfg")\n' >"$P2/project.godot"
for i in 1 2; do
  timeout 120 "${G[@]}" godot --headless --editor --path "$P2" >"$T/rot$i.txt" 2>&1
  grep -E '^@@' "$T/rot$i.txt" >"$T/rotf$i.txt"
  python3 -I "$AQUI/resume.py" "$T/rotf$i.txt" >>"$T/casos$i.jsonl"
done
if diff -q "$T/casos1.jsonl" "$T/casos2.jsonl" >/dev/null; then IG="idênticas (diff vazio)"; else IG="DIFEREM:"; fi
{
  echo "# oráculo «propriedades por nome» — $(date -u +%Y-%m-%dT%H:%MZ)"
  echo "# godot --version: $("${G[@]}" godot --version 2>/dev/null | tail -1)"
  echo "# pacote: $(LC_ALL=C pacman -Q godot) ; licença: $(LC_ALL=C pacman -Qi godot | sed -n 's/^Licenses *: //p')"
  echo "# b2: godot --headless --editor --path <proj_rot> com o plugin rotulos.gd (só as linhas @@ do editor)"
  echo "# comando: $CMD (2×; exit $(cat "$T/rc1") e $(cat "$T/rc2")); corridas $IG"
  [ "$IG" = "DIFEREM:" ] && diff "$T/casos1.jsonl" "$T/casos2.jsonl" | head -20 | sed 's/^/#   /'
  echo "# formato: {caso, r (resposta observada), stderr (o que o motor imprimiu durante o caso)}"
  cat "$T/casos1.jsonl"
} >"$AQUI/saida.jsonl"
echo "gravado em $AQUI/saida.jsonl ($IG)"

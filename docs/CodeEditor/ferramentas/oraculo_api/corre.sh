#!/usr/bin/env bash
# Oráculo «a API que funciona» (CodeEditor item 6): o motor do Godot 4.7.2 (MIT, `pacman -Qi godot`) CORRIDO sem
# janela (`--headless`), a 60 quadros fixos, sobre os casos de `api.gd`. Nunca lido (CLAUDE.md §0.9).
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_api/corre.sh
# Grava docs/CodeEditor/ferramentas/oraculo_api/saida.txt (cabeçalho + uma linha por facto).
set -euo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_api.XXXX)
trap 'rm -rf "$T"' EXIT
mkdir -p "$T/proj" "$T/home" "$T/cfg" "$T/data" "$T/cache"
printf 'config_version=5\n\n[application]\nconfig/name="oraculo_api"\n' >"$T/proj/project.godot"
cp "$AQUI/api.gd" "$T/proj/api.gd"
SAI="$AQUI/saida.txt"
{
  echo "# oráculo «a API que funciona» — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# godot: $(LC_ALL=C pacman -Q godot) ; $(LC_ALL=C pacman -Qi godot | grep -E '^Licenses' | sed 's/ *: /: /')"
  echo "# corrida: godot --headless --fixed-fps 60 -s api.gd (casos: $AQUI/api.gd)"
} >"$SAI"
env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 \
  XDG_CONFIG_HOME="$T/cfg" XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache" \
  timeout 120 godot --headless --fixed-fps 60 --path "$T/proj" -s api.gd -- "$T/saida.txt" >"$T/godot.log" 2>&1 || {
    echo "godot falhou:" >&2; tail -30 "$T/godot.log" >&2; exit 1; }
grep -iE 'error|warning' "$T/godot.log" | sed 's/^/# godot.log: /' >>"$SAI" || true
cat "$T/saida.txt" >>"$SAI"
echo "gravado em $SAI"

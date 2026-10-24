#!/usr/bin/env bash
# Oráculo «como o Inspector do Godot mostra um nó que junta várias partes?» (CodeEditor, pergunta (c) do dono de
# 22/10, «busque o padrão-ouro»). O EDITOR do Godot 4.7.2 (MIT, `pacman -Qi godot`) CORRIDO numa Xvfb, com a cena
# Camera2D + Parallax2D (os equivalentes dos nossos cartões Câmera e Paralaxe). A sonda (`sonda.gd`) selecciona cada
# nó, abre as secções e grava a estrutura; o corre.sh fotografa cada fase.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_cartao_composto/corre.sh
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_cartao.XXXX)
P=$T/proj
mkdir -p "$P/addons/sonda" "$T/home" "$T/cfg" "$T/data" "$T/cache" "$T/sonda"
cp "$AQUI/sonda.gd" "$P/addons/sonda/sonda.gd"
printf '[plugin]\nname="sonda"\ndescription=""\nauthor=""\nversion="1"\nscript="sonda.gd"\n' >"$P/addons/sonda/plugin.cfg"
cat >"$P/project.godot" <<'EOP'
config_version=5

[application]
config/name="oraculo_cartao"
run/main_scene="res://main.tscn"
config/features=PackedStringArray("4.7", "GL Compatibility")

[editor_plugins]
enabled=PackedStringArray("res://addons/sonda/plugin.cfg")

[rendering]
renderer/rendering_method="gl_compatibility"
renderer/rendering_method.mobile="gl_compatibility"
EOP
cat >"$P/main.tscn" <<'EOP'
[gd_scene format=3]

[node name="Main" type="Node2D"]

[node name="Camera2D" type="Camera2D" parent="."]

[node name="Parallax2D" type="Parallax2D" parent="."]
EOP
DSP=:78
G=(env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 DISPLAY=$DSP XDG_CONFIG_HOME="$T/cfg"
  XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache" SONDA_DIR="$T/sonda")
export DISPLAY=$DSP
Xvfb $DSP -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
ED=
fim() { [ -n "$ED" ] && kill "$ED" 2>/dev/null; kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
sleep 1
timeout 120 "${G[@]}" godot --headless --editor --quit --path "$P" >"$T/import.log" 2>&1
"${G[@]}" timeout 300 godot --editor --path "$P" res://main.tscn --position 0,0 --resolution 1600x1000 >"$T/editor.log" 2>&1 &
ED=$!
fotos=(foto_01_camera_dobrada foto_02_camera_aberta foto_03_parallax_aberto)
feito=0
for _ in $(seq 1 600); do
  f=$(cat "$T/sonda/fase" 2>/dev/null || echo 0)
  if [ "$f" -gt "$feito" ]; then
    sleep 1; import -window root "$AQUI/${fotos[$((f - 1))]}.png" 2>/dev/null; feito=$f
    [ "$f" -ge 3 ] && break
  fi
  sleep 0.2
done
{
  echo "# Oráculo: $(LC_ALL=C pacman -Q godot) · editor numa Xvfb 1600x1000 · $(date -I) · load $(cut -d' ' -f1-3 /proc/loadavg)"
  echo "# As secções VISÍVEIS do Inspector por fase (prof = quantas secções por cima; y/h em px do ecrã)."
  for i in 1 2 3; do echo "== fase $i"; cat "$T/sonda/fase_$i.json" 2>/dev/null || echo "(sem dados)"; echo; done
} >"$AQUI/saida.txt"
[ "$feito" -ge 3 ] || { echo "só $feito fases" >&2; grep -iE 'error' "$T/editor.log" | head >&2; exit 1; }

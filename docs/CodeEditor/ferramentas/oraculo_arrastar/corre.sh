#!/usr/bin/env bash
# Oráculo «arrastar para o código» (CodeEditor item 8): o EDITOR do Godot 4.7.2 (MIT, `pacman -Qi godot`) CORRIDO numa
# Xvfb; cada caso é um arrasto REAL do rato (xdotool/XTest) da árvore da cena, do FileSystem ou do Inspector para dentro
# do script aberto, com e sem modificador na soltura. Nunca lido (CLAUDE.md §0.9). A sonda (`sonda.gd`, um EditorPlugin)
# só diz onde estão as coisas no ecrã e lê o texto; o resumo diz, por caso, o que a soltura ESCREVEU e onde ficou o cursor.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_arrastar/corre.sh
# Grava docs/CodeEditor/ferramentas/oraculo_arrastar/saida.txt (cabeçalho + um caso por linha) e as fotos ao lado.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_arrastar.XXXX)
P=$T/proj
mkdir -p "$P/addons/sonda" "$T/home" "$T/cfg" "$T/data" "$T/cache" "$T/sonda"
cp "$AQUI/sonda.gd" "$P/addons/sonda/sonda.gd"
printf '[plugin]\nname="sonda"\ndescription=""\nauthor=""\nversion="1"\nscript="sonda.gd"\n' >"$P/addons/sonda/plugin.cfg"
cat >"$P/project.godot" <<'EOF'
config_version=5

[application]
config/name="oraculo_arrastar"
run/main_scene="res://main.tscn"
config/features=PackedStringArray("4.7", "GL Compatibility")

[editor_plugins]
enabled=PackedStringArray("res://addons/sonda/plugin.cfg")

[rendering]
renderer/rendering_method="gl_compatibility"
renderer/rendering_method.mobile="gl_compatibility"
EOF
cat >"$P/icon.svg" <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="#4a8"/></svg>
EOF
printf 'extends Node2D\n\nfunc _ready() -> void:\n\t\n\tpass\n' >"$P/main.gd"
printf 'extends CharacterBody2D\n\nfunc _physics_process(_delta: float) -> void:\n\t\n\tpass\n' >"$P/jogador.gd"
printf 'extends Node\n\nfunc f() -> void:\n\t\n\tpass\n' >"$P/solto.gd"
printf '[gd_scene format=3]\n\n[node name="Outra" type="Node2D"]\n' >"$P/outra.tscn"
printf '[gd_resource type="Resource" format=3]\n\n[resource]\n' >"$P/dados.tres"
cat >"$P/main.tscn" <<'EOF'
[gd_scene load_steps=4 format=3]

[ext_resource type="Script" path="res://main.gd" id="1"]
[ext_resource type="Script" path="res://jogador.gd" id="2"]
[ext_resource type="Texture2D" path="res://icon.svg" id="3"]

[node name="Main" type="Node2D"]
script = ExtResource("1")

[node name="Porta" type="Sprite2D" parent="."]
texture = ExtResource("3")

[node name="Jogador" type="CharacterBody2D" parent="."]
script = ExtResource("2")

[node name="Arma" type="Node2D" parent="Jogador"]

[node name="Caixa Grande" type="Node2D" parent="."]

[node name="Placar" type="Label" parent="."]
unique_name_in_owner = true
text = "0"
EOF
DSP=:78
G=(env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 DISPLAY=$DSP XDG_CONFIG_HOME="$T/cfg"
  XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache" SONDA_DIR="$T/sonda")
export DISPLAY=$DSP
Xvfb $DSP -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
ED=
fim() { [ -n "$ED" ] && kill "$ED" 2>/dev/null; kill "$XV" 2>/dev/null; [ -z "${GUARDA:-}" ] && rm -rf "$T"; }
trap fim EXIT
sleep 1
timeout 120 "${G[@]}" godot --headless --editor --quit --path "$P" >"$T/import.log" 2>&1 || {
  echo "import falhou" >&2; tail -20 "$T/import.log" >&2; exit 1; }
"${G[@]}" timeout 900 godot --editor --path "$P" --position 0,0 --resolution 1600x1000 >"$T/editor.log" 2>&1 &
ED=$!
for _ in $(seq 1 60); do W=$(xdotool search --pid "$ED" --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 1; done
[ -n "$W" ] || { echo "o editor não abriu" >&2; exit 1; }
sleep 8
xdotool windowfocus --sync "$W"

N=0
# pede <cmd> <args...> → imprime o JSON da resposta
pede() {
  # o contador vive num ficheiro: `pede` corre dentro de $(…), onde uma variável do pai não sobe.
  N=$(($(cat "$T/sonda/n" 2>/dev/null || echo 0) + 1)); echo "$N" >"$T/sonda/n"
  (IFS=$'\t'; echo "$N$IFS$*") >"$T/sonda/cmd"
  for _ in $(seq 1 100); do
    if [ -f "$T/sonda/out" ] && [ "$(cut -d' ' -f1 "$T/sonda/out")" = "$N" ]; then cut -d' ' -f2- "$T/sonda/out"; return 0; fi
    sleep 0.1
  done
  echo '{"erro":"a sonda não respondeu"}'
}
# A 1.ª abertura do editor demora (importa, compila os shaders no llvmpipe): espera-se a sonda, nunca um relógio fixo.
for _ in $(seq 1 30); do pede nome . | grep -q erro || break; done
pede nome . | grep -q erro && { echo "a sonda não carregou:" >&2; grep -A3 'SCRIPT ERROR' "$T/editor.log" >&2; exit 1; }
xy() { pede "$@" | python3 -I -c 'import json,sys; v=json.load(sys.stdin)["xy"]; print(*v) if v else print("")'; }
debaixo() { pede debaixo | tr -d '\n'; }

R=$T/casos.jsonl
# caso <rótulo> <modificador|-> <origem: no:<caminho>|arquivo:<res>|prop:<nome>> <linha> <coluna> [foto]
caso() {
  local rot=$1 mod=$2 org=$3 ln=$4 col=$5 foto=${6:-}
  local tipo=${org%%:*} alvo=${org#*:} o d
  o=$(xy "$tipo" "$alvo"); d=$(xy pos "$ln" "$col")
  if [ -z "$o" ] || [ -z "$d" ]; then echo "{\"caso\":\"$rot\",\"erro\":\"sem pixel origem=[$o] destino=[$d]\"}" >>"$R"; return; fi
  set -- $o; local x0=$1 y0=$2; set -- $d; local x1=$1 y1=$2
  xdotool mousemove "$x0" "$y0"; sleep 0.25
  local sob_o; sob_o=$(debaixo)
  xdotool mousedown 1; sleep 0.15
  for k in 1 2 3 4 5 6 7 8 9 10 11 12; do
    xdotool mousemove $((x0 + (x1 - x0) * k / 12)) $((y0 + (y1 - y0) * k / 12)); sleep 0.04
  done
  sleep 0.35
  local sob_d; sob_d=$(debaixo)
  [ "$mod" != - ] && { xdotool keydown "$mod"; sleep 0.2; }
  [ -n "$foto" ] && import -window root "$AQUI/$foto.png" 2>/dev/null
  xdotool mouseup 1; sleep 0.2
  [ "$mod" != - ] && xdotool keyup "$mod"
  sleep 0.6
  echo "{\"caso\":\"$rot\",\"mod\":\"$mod\",\"origem\":\"$org\",\"destino\":[$ln,$col],\"sob_origem\":$sob_o,\"sob_destino\":$sob_d,\"depois\":$(pede texto)}" >>"$R"
}
abre() { pede abre res://main.tscn "res://$1" >>"$T/abre.log"; echo >>"$T/abre.log"; sleep 1.5; }
base() { echo "{\"base\":\"$1\",\"depois\":$(pede texto)}" >>"$R"; }

# 1 — main.gd (no nó raiz): cada nó × modificador × (topo do ficheiro | dentro da função).
abre main.gd; base res://main.gd
for org in Porta Jogador/Arma "Caixa Grande" Placar; do
  for mod in - ctrl shift alt; do
    for alvo in "1 0" "3 1"; do
      pede repoe res://main.gd >/dev/null; pede sel "$org" >/dev/null; sleep 0.3
      caso "main.gd ← nó $org" "$mod" "no:$org" $alvo
    done
  done
done
# 2 — vários nós de uma vez.
for mod in - ctrl; do
  for alvo in "1 0" "3 1"; do
    # a seleção múltipla faz-se com o RATO (clique + Ctrl+clique): a feita pela API cai no premir do arrasto.
    pede repoe res://main.gd >/dev/null; pede sel Porta >/dev/null; sleep 0.3
    set -- $(xy no Porta); xdotool mousemove "$1" "$2" click 1; sleep 0.3
    set -- $(xy no Placar); xdotool mousemove "$1" "$2" keydown ctrl click 1 keyup ctrl; sleep 0.4
    echo "{\"selecao_antes\":$(pede selecao)}" >>"$R"
    caso "main.gd ← nós Porta+Placar" "$mod" "no:Placar" $alvo
  done
done
# 3 — ficheiros do FileSystem.
for f in res://icon.svg res://outra.tscn res://dados.tres res://jogador.gd; do
  for mod in - ctrl shift; do
    for alvo in "1 0" "3 1"; do
      pede repoe res://main.gd >/dev/null; sleep 0.2
      caso "main.gd ← ficheiro $f" "$mod" "arquivo:$f" $alvo
    done
  done
done
# 4 — propriedades do Inspector.
for np in "Porta texture" "Placar text"; do
  set -- $np
  for mod in - ctrl shift alt; do
    pede repoe res://main.gd >/dev/null; pede sel "$1" >/dev/null; sleep 0.6
    caso "main.gd ← propriedade $1.$2" "$mod" "prop:$2" 3 1
  done
done
# 5 — jogador.gd (script de um nó filho): caminhos relativos.
abre jogador.gd; base res://jogador.gd
for org in Jogador/Arma Porta . Placar; do
  for mod in - ctrl; do
    pede repoe res://jogador.gd >/dev/null; pede sel "$org" >/dev/null; sleep 0.3
    caso "jogador.gd ← nó $org" "$mod" "no:$org" 3 1
  done
done
# 6 — solto.gd (de nenhum nó da cena aberta).
abre solto.gd; base res://solto.gd
for mod in - ctrl; do
  pede repoe res://solto.gd >/dev/null; pede sel Porta >/dev/null; sleep 0.3
  caso "solto.gd ← nó Porta" "$mod" "no:Porta" 3 1
done
echo "{\"avisos_solto\":$(pede avisos)}" >>"$R"
import -window root "$AQUI/foto_03_solto.png" 2>/dev/null

# 7 — undo: um Ctrl+Z depois da soltura.
abre main.gd; pede repoe res://main.gd >/dev/null; base res://main.gd
pede sel Porta >/dev/null; sleep 0.3
caso "main.gd ← nó Porta (foto a meio do arrasto, sem modificador)" - "no:Porta" 3 1 foto_01_a_meio_do_arrasto
pede repoe res://main.gd >/dev/null; pede sel Porta >/dev/null; sleep 0.3
caso "main.gd ← nó Porta (foto a meio do arrasto, Ctrl)" ctrl "no:Porta" 1 0 foto_04_a_meio_do_arrasto_ctrl
xdotool key ctrl+z; sleep 0.6
echo "{\"caso\":\"undo: 1 Ctrl+Z depois da soltura\",\"depois\":$(pede texto)}" >>"$R"

# 8 — renomear depois de soltar: o texto acompanha? o editor avisa?
pede repoe res://main.gd >/dev/null; pede sel Porta >/dev/null; sleep 0.3
caso "main.gd ← nó Porta (antes de renomear)" - "no:Porta" 3 1
pede sel Placar >/dev/null; sleep 0.3
caso "main.gd ← nó Placar (antes de renomear)" ctrl "no:Placar" 1 0
for par in "Porta Portao" "Placar Pontos"; do
  set -- $par
  o=$(xy no "$1"); set -- $o "$2"
  xdotool mousemove "$1" "$2" click 1; sleep 0.4; xdotool key F2; sleep 0.4; xdotool key ctrl+a; xdotool type "$3"
  xdotool key Return; sleep 1.0
done
echo "{\"caso\":\"depois de renomear Porta→Portao e Placar→Pontos\",\"nomes\":[$(pede nome Portao),$(pede nome Pontos)],\"depois\":$(pede texto)}" >>"$R"
sleep 2
echo "{\"avisos_depois_de_renomear\":$(pede avisos)}" >>"$R"
import -window root "$AQUI/foto_02_depois_de_renomear.png" 2>/dev/null
for par in "Portao Porta" "Pontos Placar"; do
  set -- $par
  o=$(xy no "$1"); set -- $o "$2"
  xdotool mousemove "$1" "$2" click 1; sleep 0.4; xdotool key F2; sleep 0.4; xdotool key ctrl+a; xdotool type "$3"
  xdotool key Return; sleep 1.0
done

# 9 — o que o editor sugere ao escrever um caminho de nó (o gémeo do arrasto pelo teclado).
for gatilho in '$' '%' 'get_node("'; do
  pede repoe res://main.gd >/dev/null; sleep 0.2
  set -- $(xy pos 3 1); xdotool mousemove "$1" "$2" click 1; sleep 0.3
  xdotool type --delay 80 "$gatilho"; sleep 1.5
  echo "{\"completar\":\"$(printf %s "$gatilho" | sed 's/"/\\"/g')\",\"r\":$(pede completar)}" >>"$R"
  xdotool key Escape; sleep 0.3
done
# 10 — Ctrl+clique sobre `$Porta` no texto: aonde leva?
pede repoe res://main.gd >/dev/null; pede sel Porta >/dev/null; sleep 0.3
caso "main.gd ← nó Porta (para o Ctrl+clique)" - "no:Porta" 3 1
pede sel Placar >/dev/null; sleep 0.3
set -- $(xy pos 3 4); xdotool mousemove "$1" "$2"; sleep 0.3; xdotool keydown ctrl; sleep 0.3; xdotool click 1; sleep 0.2
xdotool keyup ctrl; sleep 1.2
echo "{\"ctrl_clique\":{\"selecao\":$(pede selecao),\"ecra\":$(pede ecra),\"texto\":$(pede texto)}}" >>"$R"
import -window root "$AQUI/foto_05_ctrl_clique.png" 2>/dev/null

python3 -I "$AQUI/resume.py" "$R" "$(date -u +%Y-%m-%dT%H:%MZ)" "$(cat /proc/loadavg)" \
  "$(LC_ALL=C pacman -Q godot)" "$(cat "$T/abre.log" | head -1)" >"$AQUI/saida.txt"
cp "$R" "$AQUI/casos.jsonl"
grep -iE 'error' "$T/editor.log" | grep -viE 'alsa|audio' | sed 's/^/# editor.log: /' | head -10 >>"$AQUI/saida.txt"
echo "gravado em $AQUI/saida.txt"

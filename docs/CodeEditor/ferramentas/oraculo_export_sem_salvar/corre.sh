#!/usr/bin/env bash
# Oráculo «o @export aparece no Inspector SEM salvar?» (CodeEditor 1b, ordem do dono 23/10). O EDITOR do Godot 4.7.2
# (MIT, `pacman -Qi godot`) CORRIDO numa Xvfb; a escrita são teclas reais (xdotool) no script aberto, o nó do script
# seleccionado no Inspector, e NUNCA se salva até ao passo 9. A sonda (`sonda.gd`) regista a cada 50 ms o tamanho do
# texto e as propriedades de script que o Inspector mostra; o resumo diz, por passo, quando cada uma apareceu.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_export_sem_salvar/corre.sh
# Grava saida.txt (cabeçalho + um passo por bloco), passos.jsonl e as fotos ao lado.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_export.XXXX)
P=$T/proj
mkdir -p "$P/addons/sonda" "$T/home" "$T/cfg" "$T/data" "$T/cache" "$T/sonda"
cp "$AQUI/sonda.gd" "$P/addons/sonda/sonda.gd"
printf '[plugin]\nname="sonda"\ndescription=""\nauthor=""\nversion="1"\nscript="sonda.gd"\n' >"$P/addons/sonda/plugin.cfg"
cat >"$P/project.godot" <<'EOF'
config_version=5

[application]
config/name="oraculo_export"
run/main_scene="res://main.tscn"
config/features=PackedStringArray("4.7", "GL Compatibility")

[editor_plugins]
enabled=PackedStringArray("res://addons/sonda/plugin.cfg")

[rendering]
renderer/rendering_method="gl_compatibility"
renderer/rendering_method.mobile="gl_compatibility"
EOF
printf 'extends Node2D\n\n@export var vida := 10\n\nfunc _process(_delta: float) -> void:\n\tpass\n' >"$P/jogador.gd"
cp "$P/jogador.gd" "$T/jogador.original"
cat >"$P/main.tscn" <<'EOF'
[gd_scene load_steps=2 format=3]

[ext_resource type="Script" path="res://jogador.gd" id="1"]

[node name="Main" type="Node2D"]

[node name="Jogador" type="Node2D" parent="."]
script = ExtResource("1")
EOF
DSP=:79
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
"${G[@]}" timeout 600 godot --editor --path "$P" --position 0,0 --resolution 1600x1000 >"$T/editor.log" 2>&1 &
ED=$!
for _ in $(seq 1 60); do W=$(xdotool search --pid "$ED" --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 1; done
[ -n "$W" ] || { echo "o editor não abriu" >&2; exit 1; }
sleep 8
xdotool windowfocus --sync "$W"

pede() {
  N=$(($(cat "$T/sonda/n" 2>/dev/null || echo 0) + 1)); echo "$N" >"$T/sonda/n"
  (IFS=$'\t'; echo "$N$IFS$*") >"$T/sonda/cmd"
  for _ in $(seq 1 100); do
    if [ -f "$T/sonda/out" ] && [ "$(cut -d' ' -f1 "$T/sonda/out")" = "$N" ]; then cut -d' ' -f2- "$T/sonda/out"; return 0; fi
    sleep 0.1
  done
  echo '{"erro":"a sonda não respondeu"}'
}
for _ in $(seq 1 30); do pede nome | grep -q erro || break; done
pede nome | grep -q erro && { echo "a sonda não carregou:" >&2; grep -A3 'SCRIPT ERROR' "$T/editor.log" >&2; exit 1; }

R=$T/passos.jsonl
CFG=$(pede abre res://main.tscn res://jogador.gd); sleep 2
pede sel Jogador >/dev/null; sleep 1.5
# escreve <texto>: tecla a tecla (120 ms entre teclas, mais rápido que o idle do editor), Escape fecha a lista de sugestões
escreve() { xdotool type --delay 120 "$1"; xdotool key Escape; }
# passo <rótulo> <espera s> <foto|-> — corre DEPOIS das teclas; grava o registo da sonda desde o «vigia»
passo() {
  pede marca "fim das teclas" >/dev/null
  sleep "$2"
  [ "$3" != - ] && import -window root "$AQUI/$3.png" 2>/dev/null
  echo "{\"passo\":\"$1\",\"lido\":$(pede lido),\"avisos\":$(pede avisos),\"abas\":$(pede abas),\"texto\":$(pede texto)}" >>"$R"
}
inicio() { pede vigia >/dev/null; }

echo "{\"inicial\":{\"cfg\":$CFG,\"lido\":$(pede lido)}}" >>"$R"

# 1 — uma linha @export nova, escrita tecla a tecla: aparece a cada tecla, ao parar, ou nunca (sem salvar)?
inicio; pede caret_em "vida := 10" >/dev/null; xdotool key End Return; escreve "@export var velocidade := 3.0"
passo "1 escrever @export var velocidade := 3.0" 6 foto_01_sem_salvar

# 2 — o artista pôs 7 no campo (valor próprio); mudar o PADRÃO no texto (3.0 → 5.0): o 7 fica?
pede setv Jogador velocidade 7 >/dev/null; sleep 0.5
inicio; pede caret_em "velocidade := 3.0" >/dev/null; xdotool key BackSpace BackSpace BackSpace; escreve "5.0"
passo "2 mudar o padrão 3.0→5.0 com valor próprio 7" 5 -

# 3 — erro de sintaxe numa linha nova: o Inspector esvazia, mantém o último válido, ou mostra outra coisa?
inicio; pede caret_em "velocidade := 5.0" >/dev/null; xdotool key End Return; escreve "var = ="
passo "3 erro de sintaxe (var = =)" 6 foto_02_com_erro

# 4 — com o erro lá: uma propriedade nova ACIMA do erro
inicio; pede caret_em "vida := 10" >/dev/null; xdotool key End Return; escreve "@export var escudo := 1"
passo "4 nova @export escudo com o erro presente" 6 -

# 5 — reseleccionar o nó com o script partido (o Inspector reconstrói-se do zero)
inicio; pede sel Main >/dev/null; sleep 1; pede sel Jogador >/dev/null
passo "5 reseleccionar o nó com o script partido" 3 foto_03_reseleccionado_com_erro

# 6 — apagar a linha do erro
inicio; pede caret_em "var = =" >/dev/null; xdotool key End shift+Home shift+Home Delete BackSpace
passo "6 apagar a linha do erro" 6 foto_04_erro_apagado

# 7 — renomear velocidade → rapidez (sem salvar): o valor próprio 7 viaja, perde-se, ou fica órfão?
inicio; pede caret_em "var velocidade" >/dev/null
for _ in 1 2 3 4 5 6 7 8 9 10; do xdotool key BackSpace; done; escreve "rapidez"
passo "7 renomear velocidade→rapidez" 6 -

# 8 — apagar a linha do escudo
inicio; pede caret_em "escudo := 1" >/dev/null; xdotool key End shift+Home shift+Home Delete BackSpace
passo "8 apagar a linha @export escudo" 6 -

# 9 — o disco nunca foi tocado? Depois Ctrl+S.
echo "{\"disco_igual_ao_original_antes_do_ctrl_s\":$(cmp -s "$P/jogador.gd" "$T/jogador.original" && echo true || echo false)}" >>"$R"
inicio; xdotool key ctrl+s
passo "9 Ctrl+S" 4 foto_05_salvo

python3 -I "$AQUI/resume.py" "$R" "$(date -u +%Y-%m-%dT%H:%MZ)" "$(cat /proc/loadavg)" "$(LC_ALL=C pacman -Q godot)" >"$AQUI/saida.txt"
cp "$R" "$AQUI/passos.jsonl"
grep -iE 'error' "$T/editor.log" | grep -viE 'alsa|audio' | sed 's/^/# editor.log: /' | head -10 >>"$AQUI/saida.txt"
echo "gravado em $AQUI/saida.txt"

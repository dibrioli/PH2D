#!/usr/bin/env bash
# Oráculo «de quem é o teclado com o jogo a correr» (CodeEditor item 7): o EDITOR do Godot 4.7.2 (MIT, `pacman -Qi
# godot`) CORRIDO numa Xvfb (onde o XTest chega — na Xwayland do `kwin --virtual` não chega, fotografa_cena.sh), com o
# jogo `jogo.gd` a registar cada tecla que lhe chega e um batimento por 100 ms. Nunca lido (CLAUDE.md §0.9).
# Cada passo do roteiro escreve uma marca; o resumo diz, por passo: que teclas o JOGO recebeu, se o jogo ANDOU
# (batimentos), e se ainda vive.
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_teclado/corre.sh
# Grava docs/CodeEditor/ferramentas/oraculo_teclado/saida.txt (cabeçalho + uma linha por passo) e as fotos ao lado.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d /tmp/oraculo_teclado.XXXX)
mkdir -p "$T/proj" "$T/home" "$T/cfg" "$T/data" "$T/cache"
cat >"$T/proj/project.godot" <<'EOF'
config_version=5

[application]
config/name="oraculo_teclado"
run/main_scene="res://main.tscn"
config/features=PackedStringArray("4.7", "GL Compatibility")

[rendering]
renderer/rendering_method="gl_compatibility"
renderer/rendering_method.mobile="gl_compatibility"
EOF
cp "$AQUI/jogo.gd" "$T/proj/jogo.gd"
cat >"$T/proj/main.tscn" <<'EOF'
[gd_scene load_steps=2 format=3]

[ext_resource type="Script" path="res://jogo.gd" id="1"]

[node name="Main" type="Node2D"]
script = ExtResource("1")
EOF
DSP=:77
G=(env -i HOME="$T/home" PATH=/usr/bin:/bin LANG=en_US.UTF-8 DISPLAY=$DSP XDG_CONFIG_HOME="$T/cfg"
  XDG_DATA_HOME="$T/data" XDG_CACHE_HOME="$T/cache" ORACULO_LOG="$T/jogo.log")
export DISPLAY=$DSP
Xvfb $DSP -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
ED=
fim() { [ -n "$ED" ] && kill "$ED" 2>/dev/null; pkill -f "oraculo_teclado.*godot" 2>/dev/null; kill "$XV" 2>/dev/null
  [ -z "${GUARDA:-}" ] && rm -rf "$T"; }
trap fim EXIT
sleep 1
timeout 120 "${G[@]}" godot --headless --editor --quit --path "$T/proj" >"$T/import.log" 2>&1 || {
  echo "import falhou" >&2; tail -20 "$T/import.log" >&2; exit 1; }

agora() { date +%s%3N; }
marca() { echo "$(agora) PASSO $*" >>"$T/marcas.log"; }
foto() { import -window root "$AQUI/$1.png" 2>/dev/null; }
vivo() { [ -n "${JOGO:-}" ] && kill -0 "$JOGO" 2>/dev/null && echo sim || echo nao; }

"${G[@]}" timeout 400 godot --editor --path "$T/proj" --position 0,0 --resolution 1600x1000 >"$T/editor.log" 2>&1 &
ED=$!
for _ in $(seq 1 60); do W=$(xdotool search --pid "$ED" --onlyvisible --name . 2>/dev/null | head -1); [ -n "$W" ] && break; sleep 1; done
[ -n "$W" ] || { echo "o editor não abriu" >&2; exit 1; }
sleep 8

# 1 — F5 com o editor em foco lança o jogo.
xdotool windowfocus --sync "$W"
marca "editor-em-foco F5 (correr)"
xdotool key F5
for _ in $(seq 1 40); do grep -q pronto "$T/jogo.log" 2>/dev/null && break; sleep 0.5; done
JOGO=$(grep -o 'pronto pid=[0-9]*' "$T/jogo.log" | head -1 | cut -d= -f2)
sleep 2
foto foto_01_jogo_a_correr
# A janela do jogo embutido é a «(DEBUG)» MAIS PEQUENA (a outra é a moldura com a barra Input/2D/3D); a busca por
# `--pid` do jogo devolve a janela principal do editor (medido na 1.ª corrida, 20/10).
JW=$(for w in $(xdotool search --onlyvisible --name 'DEBUG' 2>/dev/null); do
  eval "$(xdotool getwindowgeometry --shell "$w")"; echo "$((WIDTH * HEIGHT)) $w"; done | sort -n | head -1 | cut -d' ' -f2)
eval "$(xdotool getwindowgeometry --shell "$JW")"   # X Y WIDTH HEIGHT do jogo (no ecrã)
JX=$((X + WIDTH / 2)); JY=$((Y + HEIGHT / 2))
echo "$(agora) GEOMETRIA jogo pid=$JOGO janela=$JW x=$X y=$Y l=$WIDTH a=$HEIGHT" >>"$T/marcas.log"
for w in $(xdotool search --pid "$ED" --onlyvisible --name . 2>/dev/null); do
  eval "$(xdotool getwindowgeometry --shell "$w")"
  echo "$(agora) GEOMETRIA editor janela=$w nome=$(xdotool getwindowname "$w" | tr ' ' _) x=$X y=$Y l=$WIDTH a=$HEIGHT" >>"$T/marcas.log"
done
eval "$(xdotool getwindowgeometry --shell "$JW")"
GX=$X; GY=$Y

passo() { local nome=$1; shift; marca "$nome"; "$@"; sleep 1.2; }
tecla() { xdotool key "$@"; }

# 2 — o jogo tem o foco depois de F5 (sem clicar): as teclas nuas que um jogo usa.
passo "jogo-em-foco space" tecla space
passo "jogo-em-foco Left" tecla Left
passo "jogo-em-foco a" tecla a
passo "jogo-em-foco Tab" tecla Tab
passo "jogo-em-foco Escape" tecla Escape
passo "jogo-em-foco Delete" tecla Delete
passo "jogo-em-foco 1" tecla 1
passo "jogo-em-foco ctrl+s" tecla ctrl+s
passo "jogo-em-foco ctrl+z" tecla ctrl+z
# 3 — os atalhos do transporte com o jogo em foco.
passo "jogo-em-foco F7 (pausa)" tecla F7
sleep 1
foto foto_02_pausa_f7
passo "jogo-em-foco F7 (de novo)" tecla F7
passo "jogo-em-foco F9 (suspender)" tecla F9
sleep 1
passo "jogo-em-foco F10 (quadro seguinte?)" tecla F10
passo "jogo-em-foco F9 (de novo)" tecla F9
# 4 — clicar no editor (fora do jogo) e premir as mesmas teclas.
passo "clique no editor (100,900)" xdotool mousemove 100 900 click 1
passo "editor-em-foco space" tecla space
passo "editor-em-foco Left" tecla Left
passo "editor-em-foco F7 (pausa)" tecla F7
passo "editor-em-foco F7 (de novo)" tecla F7
# 5 — clicar DENTRO do jogo devolve-lhe o teclado?
passo "clique no jogo (centro)" xdotool mousemove "$JX" "$JY" click 1
passo "jogo-clicado space" tecla space
# 6 — o modo de escolha (2D) da barra do jogo: o jogo ainda recebe teclas e cliques?
passo "clique em 2D na barra" xdotool mousemove $((GX + 266)) $((GY - 23)) click 1
foto foto_03_modo_2d
passo "modo-2d clique no jogo" xdotool mousemove "$JX" "$JY" click 1
passo "modo-2d space" tecla space
passo "clique em Input na barra" xdotool mousemove $((GX + 202)) $((GY - 23)) click 1
passo "modo-input clique no jogo" xdotool mousemove "$JX" "$JY" click 1
passo "modo-input space" tecla space
# 7 — F8 com o jogo em foco pára o jogo?
passo "jogo-em-foco F8 (parar)" tecla F8
sleep 2
marca "FIM vivo=$(vivo)"

# Resumo por passo: o que o jogo recebeu (teclas/rato), quantos batimentos (o jogo andou?), e se vive.
{
  echo "# oráculo «de quem é o teclado com o jogo a correr» — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# godot: $(LC_ALL=C pacman -Q godot) ; $(LC_ALL=C pacman -Qi godot | grep -E '^Licenses' | sed 's/ *: /: /')"
  echo "# corrida: o EDITOR (godot --editor) numa Xvfb 1600x1000, teclas e cliques por xdotool (XTest); jogo: jogo.gd"
  echo "# coluna «jogo recebeu» = o _input do jogo; «batimentos» = linhas de 100 ms no passo (≈ 12 a correr, 0 parado)"
  sort -n "$T/marcas.log" "$T/jogo.log" | awk '
    function fecha() { if (p != "") printf "%-44s | jogo recebeu: %-60s | batimentos: %d\n", p, (r == "" ? "-" : r), b }
    / PASSO / { fecha(); p = substr($0, index($0, "PASSO ") + 6); r = ""; b = 0; next }
    / GEOMETRIA / { print "# " substr($0, index($0, "GEOMETRIA")); next }
    / batimento / { b++; next }
    / tecla | rato | foco / { sub(/^[0-9]+ /, ""); r = (r == "" ? $0 : r "; " $0); next }
    / fim / { sub(/^[0-9]+ /, ""); r = (r == "" ? $0 : r "; " $0); next }
    END { fecha() }'
  grep -iE 'error' "$T/editor.log" | grep -viE 'alsa|audio' | sed 's/^/# editor.log: /' | head -10
} >"$AQUI/saida.txt"
echo "gravado em $AQUI/saida.txt"

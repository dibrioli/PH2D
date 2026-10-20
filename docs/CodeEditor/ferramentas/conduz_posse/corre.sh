#!/usr/bin/env bash
# Roteiro CONDUZIDO «de quem é o teclado com a cena a correr» (CodeEditor item 7): o binário de smoke do PH2D numa
# Xvfb (onde o XTest chega — na Xwayland do `kwin --virtual` não chega, fotografa_cena.sh), na cena 6
# (`PH2D_CODE_SMOKE=6`, que arranca a correr), com teclas e cliques REAIS por `xdotool` — os mesmos passos do oráculo
# do Godot (`../oraculo_teclado/`). A régua é o `PH2D_POSSE_LOG`, que só escreve o que a porta decidiu.
# Cada passo tem o que TEM de aparecer no registo (e o que não pode); o fim diz ✓/✗ por passo e sai 1 se algum falhou.
# Uso (da raiz da worktree, com o binário de smoke feito):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/conduz_posse/corre.sh
# Grava docs/CodeEditor/ferramentas/conduz_posse/saida.txt e as fotos ao lado.
set -uo pipefail
AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/conduz_posse.XXXX)
mkdir -p "$T/home"
DSP=:79
Xvfb $DSP -screen 0 1280x800x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XV=$!
AP=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; kill "$XV" 2>/dev/null; [ -z "${GUARDA:-}" ] && rm -rf "$T"; }
trap fim EXIT
sleep 1
export DISPLAY=$DSP
LOG="$T/posse.log"
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY=$DSP HOME="$T/home" XDG_SESSION_TYPE=x11 PH2D_CODE_SMOKE=6 \
  PH2D_POSSE_LOG="$LOG" timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
for _ in $(seq 1 90); do grep -q 'jogo_tem=true' "$LOG" 2>/dev/null && break; sleep 0.5; done
grep -q 'jogo_tem=true' "$LOG" 2>/dev/null || { echo "a cena 6 não deu o teclado ao jogo no arranque" >&2; tail -20 "$T/app.log" >&2; exit 1; }
sleep 2

agora() { date +%s%3N; }
marca() { echo "$(agora) PASSO $*" >>"$LOG"; }
foto() { import -window root "$AQUI/$1.png" 2>/dev/null; }
# A janela abre a 1024x768 em (0,0) (sem gestor de janelas): a cena, um painel e o alvo da cena 6.
CENA="520 300"; PAINEL="100 420"; ALVO="735 325"
passo() { local nome=$1; shift; marca "$nome"; "$@"; sleep 1.0; }
tecla() { xdotool key "$@"; }
clique() { xdotool mousemove $1 $2 click 1; }

foto foto_01_a_correr
passo "jogo-em-foco space" tecla space
passo "jogo-em-foco Left" tecla Left
passo "jogo-em-foco a" tecla a
passo "jogo-em-foco Tab" tecla Tab
passo "jogo-em-foco Delete" tecla Delete
passo "jogo-em-foco F5 (pausa)" tecla F5
foto foto_02_pausa_f5
passo "F5 (continua)" tecla F5
passo "clique num painel" clique $PAINEL
passo "editor-em-foco space (pausa)" tecla space
passo "editor-em-foco space (continua)" tecla space
passo "clique num painel de novo" clique $PAINEL
passo "editor-em-foco Left" tecla Left
passo "clique no alvo da cena" clique $ALVO
foto foto_03_clique_no_alvo
passo "jogo-clicado space" tecla space
foto foto_04_estrela
passo "shift+F5 (para)" tecla shift+F5
foto foto_05_parado
marca "FIM"

# O que cada passo TEM de mostrar no registo (|-separados) e o que NÃO pode ('!' à frente).
declare -A ESPERA=(
  ["jogo-em-foco space"]="tecla Space jogo=true consumida=true"
  ["jogo-em-foco Left"]="tecla ArrowLeft jogo=true consumida=true"
  ["jogo-em-foco a"]="tecla KeyA jogo=true consumida=true"
  ["jogo-em-foco Tab"]="tecla Tab jogo=true consumida=true"
  ["jogo-em-foco Delete"]="tecla Delete jogo=true consumida=true"
  ["jogo-em-foco F5 (pausa)"]="tecla F5 jogo=false consumida=true transporte=Some(Alterna)|posse a_correr=false"
  ["F5 (continua)"]="posse a_correr=true a_vista=true jogo_tem=true"
  ["clique num painel"]="na_cena=false so_do_jogo=false|jogo_tem=false"
  ["editor-em-foco space (pausa)"]="tecla Space jogo=false consumida=false|posse a_correr=false"
  ["editor-em-foco space (continua)"]="posse a_correr=true a_vista=true jogo_tem=true"
  ["clique num painel de novo"]="jogo_tem=false"
  ["editor-em-foco Left"]="tecla ArrowLeft jogo=false consumida=false"
  ["clique no alvo da cena"]="na_cena=true so_do_jogo=true|jogo_tem=true|!selecao"
  ["jogo-clicado space"]="tecla Space jogo=true consumida=true"
  ["shift+F5 (para)"]="transporte=Some(Para)|posse a_correr=false"
)
FALHAS=0
{
  echo "# roteiro conduzido «de quem é o teclado» — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# binário: $APP ($(stat -c %y "$APP" | cut -d. -f1)) · HEAD $(git -C "$RAIZ" rev-parse --short HEAD)"
  echo "# corrida: Xvfb 1280x800, PH2D_CODE_SMOKE=6, teclas e cliques por xdotool; régua PH2D_POSSE_LOG"
  echo "# os mesmos passos do oráculo do Godot: ../oraculo_teclado/saida.txt"
  sort -n "$LOG" | awk '/ PASSO /{p=substr($0, index($0,"PASSO ")+6); print "## " p; next} {sub(/^[0-9]+ /,""); print "   " $0}'
  echo "# veredito por passo"
} >"$AQUI/saida.txt"
while IFS= read -r nome; do
  bloco=$(sort -n "$LOG" | awk -v p="$nome" '/ PASSO /{dentro=(substr($0, index($0,"PASSO ")+6)==p); next} dentro')
  ok=1
  IFS='|' read -ra conds <<<"${ESPERA[$nome]}"
  for c in "${conds[@]}"; do
    if [[ $c == !* ]]; then grep -qF -- "${c:1}" <<<"$bloco" && ok=0
    else grep -qF -- "$c" <<<"$bloco" || ok=0; fi
  done
  [ $ok = 1 ] && v="✓" || { v="✗"; FALHAS=$((FALHAS + 1)); }
  echo "$v $nome   (espera: ${ESPERA[$nome]})" >>"$AQUI/saida.txt"
done < <(grep -o 'PASSO .*' "$LOG" | sed 's/^PASSO //' | grep -v '^FIM$')
echo "# falhas: $FALHAS" >>"$AQUI/saida.txt"
grep -E '^(✓|✗)|falhas' "$AQUI/saida.txt"
[ "$FALHAS" = 0 ]

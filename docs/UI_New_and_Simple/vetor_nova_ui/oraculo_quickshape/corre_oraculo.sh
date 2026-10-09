#!/bin/bash
# ⭐ CORRE O ORÁCULO DO QUICKSHAPE — o Xournal++ (GPL-2, caixa preta: só a SAÍDA é gravada) numa tela
# INVISÍVEL (Xvfb :99), com o «dedo» sintético (xdotool) a desenhar os traços NOSSOS de `entradas/`.
# Cada traço vira um ficheiro em `fixtures/` com cabeçalho: a entrada, o que varia, a calibração
# tela→página, os pontos de entrada (já em unidades de página) e os pontos que o oráculo devolveu.
#
# uso:  bash corre_oraculo.sh [entradas_dir] [fixtures_dir]
# requer: xournalpp, xorg-server-xvfb, xdotool, imagemagick (import), python3.
#
# ⚠️ MEDIDO (08/10): o reconhecedor de formas do Xournal++ liga-se por `Ctrl+1` (Tools ▸ Drawing
# Type ▸ Shape Recognizer) e corre ao SOLTAR; deixa a forma SELECCIONADA (Esc tira a selecção); o
# `Ctrl+Z` a seguir desfaz a RECONHECIMENTO (volta ao traço cru) e só o segundo desfaz o traço; o
# 2.º `Ctrl+S` grava sem diálogo (por isso é sempre `Ctrl+Shift+S`); `xdotool --window` é ignorado
# pelo GTK (XSendEvent) — as teclas vão ao foco, que sem gestor de janelas é a janela sob o rato; e
# um traço com 4 pontos por lado NÃO é reconhecido — a amostragem tem de ser a de uma mão (~4 px).
set -u
AQUI="$(cd "$(dirname "$0")" && pwd)"
ENT="${1:-$AQUI/entradas}"
FIX="${2:-$AQUI/fixtures}"
TMP="${TMPDIR:-/tmp}/oraculo_quickshape.$$"
mkdir -p "$FIX" "$TMP/home" "$TMP/out"
export DISPLAY=:99

# ⚠️ Mata-se por PID, nunca por `pkill -f <texto>`: o padrão apanha a shell que o lançou (lição da
# memória, 05/10) — a 1.ª corrida deste roteiro matou-se a si própria assim.
XPID=""; APID=""
limpa() { [ -n "$APID" ] && kill "$APID" 2>/dev/null; [ -n "$XPID" ] && kill "$XPID" 2>/dev/null; rm -rf "$TMP"; }
trap limpa EXIT

Xvfb :99 -screen 0 1600x1000x24 -nolisten tcp >/dev/null 2>&1 &
XPID=$!
sleep 1
env -i HOME="$TMP/home" DISPLAY=:99 PATH=/usr/bin XDG_CONFIG_HOME="$TMP/home/.config" LANG=C.UTF-8 \
    xournalpp --disable-audio >"$TMP/xpp.log" 2>&1 &
APID=$!
MAIN=""
for _ in $(seq 1 100); do MAIN=$(xdotool search --name ' - Xournal' 2>/dev/null | head -1); [ -n "$MAIN" ] && break; sleep 0.2; done
[ -z "$MAIN" ] && { echo "✗ o Xournal++ não abriu (ver $TMP/xpp.log)"; exit 1; }
sleep 1
xdotool windowmove "$MAIN" 0 0; xdotool windowsize "$MAIN" 1600 1000; sleep 1
xdotool mousemove 800 500 click 1; sleep 0.3; xdotool key ctrl+z; sleep 0.2   # o foco (e o ponto que o clique deixa, desfeito)

desenha() {  # desenha <ficheiro de pontos em px>
  local first=1 x y
  while read -r x y; do
    case "$x" in '#'|'') continue;; esac
    if [ $first = 1 ]; then xdotool mousemove "${x%.*}" "${y%.*}"; xdotool mousedown 1; first=0
    else xdotool mousemove "${x%.*}" "${y%.*}"; sleep 0.012; fi
  done < "$1"
  xdotool mouseup 1
  sleep 0.8
}

grava() {  # grava <saida.xopp>  (Guardar como; devolve 0 se o ficheiro apareceu)
  local out="$1" dlg="" tenta
  rm -f "$out"
  xdotool key Escape; sleep 0.3
  for tenta in 1 2 3; do
    xdotool key ctrl+shift+s
    for _ in $(seq 1 30); do dlg=$(xdotool search --name '^Save' 2>/dev/null | head -1); [ -n "$dlg" ] && break; sleep 0.1; done
    [ -n "$dlg" ] && break
    xdotool key Escape; sleep 0.5
  done
  [ -z "$dlg" ] && return 1
  sleep 0.5
  xdotool key ctrl+a
  xdotool type --delay 20 "$out"
  sleep 0.3
  xdotool key Return
  for _ in $(seq 1 50); do [ -f "$out" ] && break; sleep 0.1; done
  sleep 0.3
  [ -f "$out" ]
}

# o ÚLTIMO traço do ficheiro .xopp, como linhas "x y"
ultimo_traco() {
  zcat "$1" | python3 -c '
import re,sys
s=sys.stdin.read()
m=re.findall(r"<stroke[^>]*>([^<]*)</stroke>", s)
if not m: sys.exit(0)
v=m[-1].split()
for i in range(0,len(v)-1,2): print(v[i], v[i+1])'
}

# ── calibração: reconhecedor DESLIGADO (é o estado de origem), uma linha de 2 pontos ─────────────
printf '500 300\n1300 800\n' > "$TMP/calib.txt"
desenha "$TMP/calib.txt"
grava "$TMP/out/calib.xopp" || {
  import -window root "$FIX/_falha_calibracao.png"
  echo "✗ a gravação da calibração falhou (foto em $FIX/_falha_calibracao.png; janelas: $(xdotool search --name '' | while read -r w; do xdotool getwindowname "$w" 2>/dev/null; done | tr '\n' '|'))"
  exit 1
}
read -r AX AY BX BY < <(ultimo_traco "$TMP/out/calib.xopp" | tr '\n' ' ')
SX=$(python3 -c "print(($BX-$AX)/800.0)"); SY=$(python3 -c "print(($BY-$AY)/500.0)")
OX=$(python3 -c "print($AX-500*$SX)");    OY=$(python3 -c "print($AY-300*$SY)")
echo "calibração: página = tela × ($SX, $SY) + ($OX, $OY)"
xdotool key ctrl+z; sleep 0.2
xdotool key ctrl+1; sleep 0.3   # o reconhecedor LIGADO daqui em diante

n=0; rec=0
for ent in "$ENT"/*.txt; do
  nome=$(basename "$ent" .txt)
  desenha "$ent"
  if ! grava "$TMP/out/$nome.xopp"; then
    echo "✗ $nome: sem gravação"; xdotool key Escape; sleep 0.3
    xdotool key ctrl+z; sleep 0.15; xdotool key ctrl+z; sleep 0.15   # senão o traço contamina o seguinte
    continue
  fi
  ultimo_traco "$TMP/out/$nome.xopp" > "$TMP/saida.txt"
  nin=$(grep -cv '^#' "$ent"); nout=$(wc -l < "$TMP/saida.txt")
  # ⚠️ «Reconhecido» NÃO é «contagem diferente»: o app reamostra o traço cru (327 → 326, 850 → 845).
  # O vale MEDIDO (08/10): crus ≥ 99 % dos pontos; círculos 54–67 %; polígonos e linhas ≤ 2 %.
  reconhecido=$(( nout * 100 <= nin * 80 ))
  {
    grep '^#' "$ent"
    echo "# oraculo: xournalpp $(xournalpp --version 2>/dev/null | head -1 | tr -d '\n') · Shape Recognizer (Ctrl+1) · strokeRecognizerMinSize=40 (origem)"
    echo "# calibracao: pagina = tela * ($SX, $SY) + ($OX, $OY)"
    echo "# pontos_saida: $nout"
    [ "$reconhecido" = 1 ] && echo "# reconhecido: sim (saida <= 80 % dos pontos da entrada)" || echo "# reconhecido: nao (saida com $nout de $nin pontos: o traco cru reamostrado)"
    echo "# --- entrada (unidades de pagina) ---"
    grep -v '^#' "$ent" | python3 -c "
import sys
for l in sys.stdin:
    x,y=map(float,l.split()); print(f'in {x*$SX+$OX:.3f} {y*$SY+$OY:.3f}')"
    echo "# --- saida do oraculo (unidades de pagina) ---"
    sed 's/^/out /' "$TMP/saida.txt"
  } > "$FIX/$nome.txt"
  n=$((n+1)); [ "$reconhecido" = 1 ] && rec=$((rec+1))
  printf '%-28s in %4d → out %4d %s\n' "$nome" "$nin" "$nout" "$([ "$reconhecido" = 1 ] && echo RECONHECIDO || echo cru)"
  xdotool key ctrl+z; sleep 0.15; xdotool key ctrl+z; sleep 0.15   # desfaz o reconhecimento E o traço
done
echo "$n traços · $rec reconhecidos · fixtures em $FIX"

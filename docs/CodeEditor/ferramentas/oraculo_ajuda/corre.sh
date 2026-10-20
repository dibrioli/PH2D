#!/usr/bin/env bash
# Oráculo «página de ajuda» (CodeEditor): Godot 4.7.2 — o Ctrl+clique num nome da API abre a referência da classe numa
# aba do editor de script. Mede: (A) que página abre e onde pousa para cada espécie de nome (função global, função de
# outra tabela, método de um valor, classe, método HERDADO, constante, nome nosso, palavra-chave); (B) a estrutura da
# página (título, descrição, tabela de métodos, descrições, índice de secções); (C) a navegação (link na mesma página,
# link para outra classe, ‹ › do histórico, a mesma página reaproveitada, Ctrl+W); (D) o ponteiro sobre link × texto
# e o balão de um tipo; (E) o teclado (PgDn, Ctrl+F). O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9).
#
# Licença: Godot = MIT (pacman -Qi godot).
#
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_ajuda/corre.sh <saida>
#   Grava em <saida>: amostras.txt (o veredito em texto, por OCR) e *_c.png (recortes). Fotos de ecrã inteiro em
#   /tmp/oraculo_ajuda_raw.XXXX (dito no fim) — não entram no repo.
#
# ⛔ Regras (as do oraculo_acabamento/corre.sh): Xvfb PRIVADO :118 (recusa :0); foto = `import -window root` (NUNCA
#   `spectacle`); dbus privado sem pastas de serviços; Godot com XDG_* próprios, x11 + opengl3; cada processo sob
#   `timeout`, parado pelo PID que ESTE script lançou.
# ⚠️ O Godot só reconhece o nome do Ctrl+clique se o rato MEXER com o Ctrl já carregado (medido 19/10: keydown com o
#   rato parado e clique = só o cursor de texto). Por isso `ctrl_clique` carrega o Ctrl, mexe 1 px e espera 0,5 s.
#
# Coordenadas: ecrã 1600x1000, janela em (0,0), leiaute de fábrica. Script: linha n em y = 134 + 25(n-1), coluna c
# em x = 604 + 8c (tab = 4). Lista de scripts: item i (0..) em y = 166 + 22i. Histórico ‹ › em (1231,96) e (1255,96).
set -euo pipefail

D="${AJUDA_DISPLAY:-:118}"
[ "$D" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
AQUI=$(cd "$(dirname "$0")" && pwd)

if [ -z "${AJUDA_DENTRO:-}" ]; then
  OUT="${1:-$(mktemp -d /tmp/oraculo_ajuda.XXXX)}"; mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
  RAW=$(mktemp -d /tmp/oraculo_ajuda_raw.XXXX)
  T=$(mktemp -d /tmp/ajd.XXXX); chmod 700 "$T"
  cat >"$T/bus.conf" <<'BC'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig><type>session</type><listen>unix:tmpdir=/tmp</listen><auth>EXTERNAL</auth>
<policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy></busconfig>
BC
  rc=0
  AJUDA_DENTRO=1 AJUDA_T="$T" AJUDA_RAW="$RAW" timeout 900 \
    dbus-run-session --config-file="$T/bus.conf" -- bash "$0" "$OUT" || rc=$?
  rm -rf "$T"
  echo "gravado em $OUT ; fotos de ecrã inteiro em $RAW (rc=$rc)"
  exit "$rc"
fi

OUT="$1"; T="$AJUDA_T"; RAW="$AJUDA_RAW"
export DISPLAY="$D" XDG_RUNTIME_DIR="$T"
unset WAYLAND_DISPLAY
A="$OUT/amostras.txt"
PIDS=()
limpa() { local i; for ((i=${#PIDS[@]}-1; i>=0; i--)); do kill "${PIDS[$i]}" 2>/dev/null || true; done; }
trap limpa EXIT

gcc -O2 -o "$T/cursor_nome" "$AQUI/../oraculo_acabamento/cursor_nome.c" -lX11 -lXfixes
shot() { import -window root "$RAW/$1.png"; }
crop() { magick "$RAW/$1.png" -crop "$2" +repage "$OUT/$1_c.png"; }
fato() { echo "$*" >>"$A"; }
hash_cursor() { "$T/cursor_nome" | sed 's/.*hash=//'; }
longe() { xdotool mousemove 1450 700; sleep "${1:-0.5}"; }
# texto de uma região da foto (OCR, invertido e 2×), numa linha só
ler() { # foto WxH+X+Y
  magick "$RAW/$1.png" -crop "$2" +repage -negate -resize 200% "$T/ocr.png"
  tesseract "$T/ocr.png" - --psm 6 2>/dev/null | tr -s ' \n' ' ' | sed 's/^ //; s/ $//'
}
# o centro (x y no ecrã) da 1.ª palavra que COMEÇA por $2 na zona da página; vazio se não houver
acha() { # foto prefixo
  magick "$RAW/$1.png" -crop 820x800+495+120 +repage -negate -resize 200% "$T/ocr.png"
  tesseract "$T/ocr.png" - --psm 11 tsv 2>/dev/null |
    awk -F'\t' -v w="$2" 'index($12, w) == 1 {print int(495 + ($7 + $9 / 2) / 2), int(120 + ($8 + $10 / 2) / 2); exit}'
}
titulo()  { ler "$1" 560x30+520+82; }
lista()   { ler "$1" 190x140+295+155; }
topo()    { ler "$1" 790x34+505+150; }
estado()  { # foto rótulo
  fato "$2: título «$(titulo "$1")» · lista «$(lista "$1")» · topo «$(topo "$1")» — prova: $1_c.png"
  crop "$1" 1020x520+290+80
}
ao_script() { xdotool mousemove 350 166 click 1; sleep 1.2; }
ctrl_clique() { # x y
  xdotool mousemove "$1" "$2"; sleep 0.3
  xdotool keydown ctrl; sleep 0.2; xdotool mousemove_relative 1 0; sleep 0.5
  MAO=$(hash_cursor)
  xdotool click 1; xdotool keyup ctrl; sleep 2.5; longe 0.4
}
# alvo (A): volta ao script, Ctrl+clique no meio da palavra (linha, coluna, comprimento)
alvo() { # rótulo linha coluna comprimento
  local x y; x=$((604 + 8 * $3 + 4 * $4)); y=$((134 + 25 * ($2 - 1)))
  ao_script; ctrl_clique "$x" "$y"; shot "a_$1"
  fato "A $1 (linha $2, col $3): cursor com Ctrl = ${NOME[$MAO]:-outro:$MAO}"
  estado "a_$1" "A $1"
}

{
  echo "# oráculo página de ajuda (Ctrl+clique → referência da classe) — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# godot: $(LC_ALL=C pacman -Q godot) ; $(LC_ALL=C pacman -Qi godot | grep -E '^Licenses' | sed 's/ *: /: /')"
  echo "# display: Xvfb $D 1600x1000x24 ; dbus privado sem serviços; fotos de ecrã inteiro em $RAW"
  echo "# script: $AQUI/ajuda.gd ; textos por OCR (tesseract) — o recorte nomeado é a prova"
} >"$A"

timeout 850 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
PIDS+=($!); sleep 1.5
G="$T/gd"; mkdir -p "$G/proj" "$G/home" "$G/cfg" "$G/data" "$G/cache"
printf 'config_version=5\n\n[application]\nconfig/name="oraculo_ajuda"\n\n[rendering]\nrenderer/rendering_method="gl_compatibility"\n' >"$G/proj/project.godot"
cp "$AQUI/ajuda.gd" "$G/proj/ajuda.gd"
env -i HOME="$G/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
  DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
  XDG_CONFIG_HOME="$G/cfg" XDG_DATA_HOME="$G/data" XDG_CACHE_HOME="$G/cache" \
  timeout 800 godot --display-driver x11 --rendering-driver opengl3 -e --path "$G/proj" >"$T/godot.log" 2>&1 &
GODOT_PID=$!; PIDS+=("$GODOT_PID")
GW=""
for _ in $(seq 40); do GW=$(xdotool search --class godot 2>/dev/null | tail -1 || true); [ -n "$GW" ] && break; sleep 1; done
[ -n "$GW" ] || { echo "Godot não abriu" >&2; tail -30 "$T/godot.log" >&2; exit 1; }
sleep 10
xdotool windowmove "$GW" 0 0 windowsize "$GW" 1600 1000; sleep 4
xdotool mousemove 80 690 click --repeat 2 --delay 80 1; sleep 4      # ajuda.gd no FileSystem (único ficheiro)
xdotool mousemove 1100 650 click 1; sleep 0.5
longe; shot s0_script; crop s0_script 1020x300+290+80

# a régua do ponteiro (o nome XFixes vem vazio): seta fora do texto, I-beam no texto, mão com Ctrl sobre `sin`
declare -A NOME=()
xdotool mousemove 1410 22; sleep 0.7; NOME[$(hash_cursor)]=seta
xdotool mousemove 720 384; sleep 0.7; NOME[$(hash_cursor)]=ibeam
xdotool mousemove 712 259; sleep 0.3; xdotool keydown ctrl; sleep 0.2; xdotool mousemove_relative 1 0; sleep 0.6
NOME[$(hash_cursor)]=mao; xdotool keyup ctrl; sleep 0.5
fato "# cursores de referência: $(for h in "${!NOME[@]}"; do printf '%s=%s ' "${NOME[$h]}" "$h"; done)"

# (A) cada espécie de nome
alvo sin_global          6 12 3
alvo clamp_mesma_pagina  8 12 5
alvo length_metodo       5 12 6
alvo Vector2_classe      4 12 7
alvo add_child_herdado   7 4 9
alvo Node_classe         7 14 4
alvo print_debug_gdscript 9 4 11
alvo PI_constante        9 16 2
alvo print_global        5 4 5
alvo amplitude_nosso     6 16 9
alvo var_palavra_chave   6 4 3

# (B) estrutura: o topo, a tabela de métodos, o índice de secções (página @GlobalScope)
xdotool mousemove 350 210 click 1; sleep 1.5                         # @GlobalScope na lista (3.º depois do script)
xdotool mousemove 312 555 click 1; sleep 1.5; longe; shot b_topo; crop b_topo 1020x880+290+80
fato "B topo: «$(ler b_topo 790x500+500+120)»"
fato "B índice de secções: «$(ler b_topo 190x200+295+545)»"
xdotool mousemove 330 643 click 1; sleep 1.5; longe; shot b_metodos; crop b_metodos 1020x880+290+80
fato "B tabela de métodos (início): «$(ler b_metodos 790x330+500+120)»"

# (C) navegação, a partir do `print`
alvo print_de_novo 5 4 5
read -r LX LY <<<"$(acha a_print_de_novo push_warning)" || true
if [ -n "${LX:-}" ]; then
  xdotool mousemove "$LX" "$LY"; sleep 0.8; fato "D ponteiro sobre o link push_warning(): ${NOME[$(hash_cursor)]:-outro:$(hash_cursor)}"
  xdotool mousemove "$LX" $((LY + 60)); sleep 0.8; fato "D ponteiro sobre texto da página: ${NOME[$(hash_cursor)]:-outro:$(hash_cursor)}"
  xdotool mousemove "$LX" "$LY" click 1; sleep 2; longe; shot c1_link_mesma_pagina; estado c1_link_mesma_pagina "C link push_warning() (mesma página)"
  xdotool mousemove 1231 96 click 1; sleep 2; longe; shot c2_volta; estado c2_volta "C ‹ depois do link na mesma página"
else
  fato "C: OCR não achou push_warning em a_print_de_novo"
fi
alvo print_outra_vez 5 4 5
read -r EX EY <<<"$(acha a_print_outra_vez Engine.print_to_stdout)" || true
if [ -n "${EX:-}" ]; then
  xdotool mousemove "$EX" "$EY" click 1; sleep 2.5; longe; shot c3_outra_classe; estado c3_outra_classe "C link Engine.print_to_stdout (outra classe)"
  xdotool mousemove 1231 96 click 1; sleep 2; longe; shot c4_volta; estado c4_volta "C ‹ depois do link para outra classe"
  xdotool mousemove 1255 96 click 1; sleep 2; longe; shot c5_avanca; estado c5_avanca "C › a seguir"
  xdotool mousemove 1231 96 click 1; sleep 2
else
  fato "C: OCR não achou Engine.print_to_stdout em a_print_outra_vez"
fi
# (D) o balão de um tipo na assinatura (`void` do print, no topo)
read -r VX VY <<<"$(acha c4_volta void)" || true
if [ -n "${VX:-}" ]; then
  xdotool mousemove "$VX" "$VY"; sleep 1.5; shot d_balao_void; crop d_balao_void 700x200+490+$((VY - 40))
  fato "D balão sobre «void»: «$(ler d_balao_void 400x60+$((VX - 40))+$((VY + 8)))» — prova: d_balao_void_c.png"
fi
# (E) teclado: PgDn, Ctrl+F (barra de busca no fundo), uma busca, Esc, Ctrl+W
xdotool mousemove 800 500 click 1; sleep 0.5; xdotool key Next; sleep 1; longe; shot e1_pgdn; estado e1_pgdn "E PgDn com o foco na página"
xdotool key ctrl+f; sleep 1; shot e2_ctrl_f; crop e2_ctrl_f 1020x60+290+890
fato "E Ctrl+F: barra «$(ler e2_ctrl_f 1000x40+295+895)» — prova: e2_ctrl_f_c.png"
xdotool type --delay 50 smoothstep; xdotool key Return; sleep 1.5; shot e3_busca; estado e3_busca "E busca «smoothstep» + Enter"
fato "E busca: barra «$(ler e3_busca 1000x40+295+895)»"
xdotool key Escape; sleep 0.5
xdotool key ctrl+w; sleep 2; longe; shot e4_ctrl_w; estado e4_ctrl_w "E Ctrl+W na página"

kill "$GODOT_PID" 2>/dev/null || true; sleep 2
echo "fim — $(date -u +%H:%MZ) loadavg $(cat /proc/loadavg)" >>"$A"

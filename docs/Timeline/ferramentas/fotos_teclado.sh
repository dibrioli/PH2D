#!/usr/bin/env bash
# Os três abertos que precisam de TECLADO (16/11): Ctrl+Z dos pontos da Custom, o Auto Key na sessão «Animating»
# (clip novo sem Done; regravar a chave) e o valor DIGITADO no Inspector. Xvfb 1600×900, cliques e teclas reais.
# A 1.ª tentativa (15/11) não fez o texto entrar: `xdotool --window` manda eventos SINTÉTICOS (XSendEvent), que o app
# ignora; aqui a janela recebe o foco e as teclas vão pelo XTEST, como as de um teclado.
# Uso (da raiz da worktree): PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Timeline/ferramentas/fotos_teclado.sh [saida]
# `PARA=<passo>` pára com a tela inteira (`tela_<passo>.png`).
RAIZ=$(git rev-parse --show-toplevel)
SAIDA=${1:-$RAIZ/docs/Timeline/fotos}
APP="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$APP" ] || { echo "falta o binário de smoke" >&2; exit 2; }
T=$(mktemp -d /tmp/fotos_teclado.XXXX)
mkdir -p "$T/home/.ph2d" "$SAIDA"
AP= XV=
fim() { [ -n "$AP" ] && kill "$AP" 2>/dev/null; [ -n "$XV" ] && kill "$XV" 2>/dev/null; rm -rf "$T"; }
trap fim EXIT
. "$RAIZ/docs/CodeEditor/ferramentas/xvfb_livre.sh"
xvfb_livre 1600x900x24 "$T/xvfb.log" 600 || exit 2
(cd "$RAIZ" && env -u WAYLAND_DISPLAY DISPLAY="$DSP" HOME="$T/home" XDG_SESSION_TYPE=x11 \
  PH2D_ANIMACAO_SMOKE=1 timeout 300 "$APP" >"$T/app.log" 2>&1) &
AP=$!
timeout 60 bash -c 'until xdotool search --onlyvisible --name . >/dev/null 2>&1; do sleep 1; done' \
  || { echo "a janela não apareceu" >&2; exit 2; }
W=$(xdotool search --onlyvisible --name . | head -1)
xdotool windowsize "$W" 1600 900 windowmove "$W" 0 0
sleep 12
xdotool windowfocus --sync "$W" 2>/dev/null || xdotool windowfocus "$W"
c() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool click "${3:-1}"; sleep 0.8; }
arrasta() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool mousedown 1
  for i in $(seq 1 12); do xdotool mousemove $(($1 + ($3 - $1) * i / 12)) $(($2 + ($4 - $2) * i / 12)); sleep 0.05; done
  sleep 0.4; xdotool mouseup 1; sleep 0.8; }
tecla() { xdotool key "$@"; sleep 0.8; }
escreve() { xdotool type --delay 60 "$1"; sleep 0.6; }
tela() { import -window root +repage "$SAIDA/tela_$1.png"; echo "tela: $SAIDA/tela_$1.png"; }
para() { [ "${PARA:-}" = "$1" ] && { tela "$1"; exit 0; }; true; }
# (1) Ctrl+Z dos pontos — o caminho do `fotos_pontos_retimado.sh` até ao r1 (o toque cria o ponto, medido 15/11),
# depois Ctrl+Z tira-o e Ctrl+Shift+Z repõe-o.
if [ "${PONTOS:-}" = 1 ]; then
  c 475 679; c 365 815; c 1211 687; c 981 684; xdotool key Escape; sleep 0.6; c 475 651
  arrasta 800 666 800 330
  c 567 479; c 567 479 3; c 675 560; c 869 837
  c 503 548; c 567 479
  arrasta 665 576 640 470
  arrasta 762 537 820 600
  c 1211 351; c 981 619; xdotool key Escape; sleep 0.6; c 567 479
  para p0
  c 710 553
  para p1
  tecla ctrl+z
  para p2
  tecla ctrl+shift+z
  para p3
  exit 0
fi
# Pausa e volta ao 0 (a cena toca em laço).
c 447 651; c 391 651
para 0
# Sonda: o teclado chega? Espaço toca (o tempo tem de andar); outro espaço pára.
if [ "${SONDA:-}" = 1 ]; then c 800 300; tecla space; sleep 1; tela sonda; exit 0; fi
# Abre o Transform do Inspector.
c 1366 760
para 1
# (3) O valor DIGITADO no Inspector com o Auto Key ligado regrava a chave do instante 0 (−300 → −150).
c 363 651
c 1385 815
para 2
tecla minus 1 5 0
para 2b
tecla Return
para 3
# A CHAVE mudou, não só o mundo: um quadro à frente e de volta ao 0 — sem chave regravada voltava −300.
c 503 651; c 419 651
para 3b
# (2B) Auto Key na sessão: arrastar o Herói (em x≈665 depois do −150) no quadro 0 regrava a chave (Position Y 0 → ~70); um quadro e volta confere.
arrasta 665 450 665 380
c 503 651; c 419 651
para 4
# (2A) «+» cria um clip novo SEM o Done no anterior; arrastar o Herói grava a chave no clip novo.
c 685 720
para 5a
arrasta 665 380 735 380
para 5

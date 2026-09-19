#!/bin/bash
# ⭐ FOTOGRAFA UMA CENA DE SMOKE NUMA TELA VIRTUAL — sem tocar no ecrã nem no rato do dono.
#
# Nasceu no TOP-20 #16 (2026-09-16), pela regra da casa de que cada passo de um smoke é conduzido e
# FOTOGRAFADO antes de ir ao dono. A foto apanhou três defeitos que nenhum gate via (botões lidos
# `…`, dois pontos numa linha, a cena a sair do ecrã) — ver o handoff do #16.
#
# uso:
#   bash docs/Components/ferramentas/fotografa_cena.sh <VAR=valor> <largura> <altura> <saida.png> [espera_s]
#   ex.: bash docs/Components/ferramentas/fotografa_cena.sh PH2D_SCRIPT_SMOKE=1 1930 1040 /tmp/x/cena.png
#
# COMO FUNCIONA
#   1. `kwin_wayland --virtual` abre uma sessão KWin INVISÍVEL, com uma configuração isolada
#      (`XDG_CONFIG_HOME` temporário) cuja única regra é MAXIMIZAR toda janela — assim a janela do
#      app (1024x768 por omissão) toma o tamanho pedido.
#   2. O app corre em **X11** na Xwayland DESSA sessão (`WAYLAND_DISPLAY` removido) e a foto é o
#      `import -window <id>` dessa janela (o id vem do `_NET_CLIENT_LIST`).
#
# ⛔⛔ AS TRES ARMADILHAS MEDIDAS
#   - O `spectacle` fala com o KWin pelo D-Bus da sessão — e a sessão é a do DONO: a 1.ª tentativa
#     fotografou o ecrã REAL dele. ⇒ nunca o `spectacle`; e este roteiro RECUSA correr se o
#     `DISPLAY` de dentro for `:0` ou o `WAYLAND_DISPLAY` for `wayland-0`.
#   - O `import -window root` falha na Xwayland (raiz *rootless*: «missing an image filename»).
#     ⇒ fotografa-se a JANELA, pelo id.
#   ⚠️ E os eventos SINTÉTICOS não chegam: o XTest na Xwayland desta sessão é ignorado (medido com
#   `XTestFakeButtonEvent` — a roda não rolou). ⛔ E o `ydotool` move o rato REAL do dono: proibido.
#   ⇒ um clique prova-se num gate de costura (`MockPanelHost`), não aqui.
#   - O `$HOME` do app e' o do DONO, e o app RE-ESCREVE `~/.ph2d/layout.txt` (a arrumacao dos
#     paineis, que vive fora do repo). ⇒ o roteiro corre com **HOME isolado** desde 19/09.
set -euo pipefail

ENV_KV="${1:?VAR=valor}"
LARG="${2:?largura}"
ALT="${3:?altura}"
SAIDA="${4:?saida.png}"
ESPERA="${5:-9}"
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$RAIZ/target/smoke/ph2d-host-desktop"
[ -x "$BIN" ] || { echo "falta o binario: cargo build -p ph2d-host-desktop --profile smoke" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg"
cat > "$TMP/cfg/kwinrulesrc" <<'EOF'
[General]
count=1
rules=r1

[r1]
Description=maximiza tudo
wmclassmatch=0
types=1
maximizehoriz=true
maximizehorizrule=3
maximizevert=true
maximizevertrule=3
EOF

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a tela virtual (DISPLAY=\$DISPLAY WAYLAND_DISPLAY=\$WAYLAND_DISPLAY)" > "$TMP/log"
  exit 3
fi
cd "$RAIZ"
# ⛔⛔ **HOME ISOLADO, e isto e' uma cerca e nao um detalhe.** A arrumacao dos paineis vive em
# `~/.ph2d/layout.txt`, FORA do repo: e' um ficheiro DO DONO, e o app RE-ESCREVE-O. Sem esta
# linha, cada fotografia mexia na arrumacao dele — a mesma familia do `spectacle` a fotografar
# o ecra real, e o mesmo defeito que ja' apanhou uma fotografia a abrir com o painel errado
# porque OUTRA arvore a correr em paralelo tinha reescrito o ficheiro.
# ⭐ E de graca ela torna a foto REPRODUTIVEL: a arrumacao passa a ser a de fabrica.
#    Quem quiser a do dono passa `HOME=$REAL_HOME` no primeiro argumento.
mkdir -p "$TMP/home"
env -u WAYLAND_DISPLAY HOME="$TMP/home" $ENV_KV PH2D_EXIT_AFTER_FRAMES=100000 "$BIN" > "$TMP/app.log" 2>&1 &
APP=\$!
sleep $ESPERA
WIN=\$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*' | tail -1)
import -display "\$DISPLAY" -window "\$WIN" "$SAIDA" >> "$TMP/log" 2>&1
kill -9 \$APP 2>/dev/null
EOF
chmod +x "$TMP/sessao.sh"

XDG_CONFIG_HOME="$TMP/cfg" timeout 120 kwin_wayland --virtual --width "$LARG" --height "$ALT" \
  --xwayland --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
cat "$TMP/log" 2>/dev/null || true
if [ -s "$SAIDA" ]; then
  echo "foto: $SAIDA"
  grep -m3 -i 'smoke\]' "$TMP/app.log" || true
else
  echo "sem foto — log do app:" >&2
  tail -20 "$TMP/app.log" >&2 || true
  exit 1
fi

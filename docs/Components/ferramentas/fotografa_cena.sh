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

# ⛔⛔⛔ **ELE NÃO CONSTRÓI, LOGO SEM ISTO FOTOGRAFA O PROGRAMA ANTERIOR** (medido 2026-09-19).
# A cena do golpe foi curada, fotografada DUAS vezes com o binário de antes da cura, e as duas fotos
# mostraram o defeito já curado — uma delas com o log a reproduzi-lo à letra. *Uma sonda que mede
# outro programa é pior do que nenhuma: ela não fica em silêncio, ela CONFIRMA.*
# ⚠️ **A recusa é ALTA e não um aviso:** o modo de falha que ela substitui é mudo por construção
# (a foto sai, é bonita, e é de outra build), e este roteiro é lido por quem já está a caçar um
# defeito. ⭐ E a régua é o PRÓPRIO ficheiro mais novo, com o nome dele na mensagem — nunca um
# relógio: `find -newer` compara os dois mtimes e não tem calibração para envelhecer.
NOVO="$(find "$RAIZ/crates" "$RAIZ/shells" -name '*.rs' -newer "$BIN" -print -quit 2>/dev/null || true)"
if [ -n "$NOVO" ]; then
  echo "RECUSA: ha' codigo mais novo que o binario (ex.: ${NOVO#"$RAIZ/"})" >&2
  echo "  a foto seria do programa ANTERIOR. Corra primeiro:" >&2
  echo "  bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke" >&2
  exit 2
fi

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

# ⛔⛔ HOME ISOLADO, e isto e' uma cerca e nao um detalhe. A arrumacao dos paineis vive em
# ~/.ph2d/layout.txt, FORA do repo: e' um ficheiro DO DONO, e o app RE-ESCREVE-O. Sem o HOME
# isolado, cada fotografia mexia na arrumacao dele. E de graca ela torna a foto REPRODUTIVEL:
# a arrumacao passa a ser a de fabrica. Quem quiser a do dono passa HOME=... no 1.o argumento.
#
# ⛔⛔⛔ E ATENCAO AO QUE SE ESCREVE DENTRO DESTE HEREDOC: ele e' <<EOF SEM ASPAS, logo toda CRASE
# ali dentro e' SUBSTITUICAO DE COMANDO. Em 2026-09-19 um comentario meu com a palavra spectacle
# entre crases EXECUTOU o spectacle — o unico programa que este roteiro proibe, porque ele
# fotografa o ecra REAL do dono — e o roteiro ficou pendurado 449 s a' espera dele.
# ⇒ prosa fica FORA do heredoc; dentro dele so' codigo, e o guarda abaixo recusa o resto.
cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a tela virtual (DISPLAY=\$DISPLAY WAYLAND_DISPLAY=\$WAYLAND_DISPLAY)" > "$TMP/log"
  exit 3
fi
cd "$RAIZ"
mkdir -p "$TMP/home"
env -u WAYLAND_DISPLAY HOME="$TMP/home" $ENV_KV PH2D_EXIT_AFTER_FRAMES=100000 "$BIN" > "$TMP/app.log" 2>&1 &
APP=\$!
sleep $ESPERA
WIN=\$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*' | tail -1)
import -display "\$DISPLAY" -window "\$WIN" "$SAIDA" >> "$TMP/log" 2>&1
kill -9 \$APP 2>/dev/null
EOF
chmod +x "$TMP/sessao.sh"

# ⭐⭐ O GUARDA da armadilha acima: o roteiro gerado nao pode conter uma crase nem um $(, porque o
# heredoc que o escreve e' sem aspas e os dois JA' correram na geracao. Se algum sobreviveu ate' o
# ficheiro, e' porque escapou — e o que vem a seguir corre-o outra vez.
if grep -q '`' "$TMP/sessao.sh"; then
  echo "RECUSA: o roteiro gerado tem uma CRASE — ela e' substituicao de comando no heredoc <<EOF" >&2
  exit 4
fi

XDG_CONFIG_HOME="$TMP/cfg" timeout 120 kwin_wayland --virtual --width "$LARG" --height "$ALT" \
  --xwayland --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
cat "$TMP/log" 2>/dev/null || true
if [ -s "$SAIDA" ]; then
  echo "foto: $SAIDA"
  # ⚠️ **`-m12` e não `-m3`** (2026-09-17): com três linhas, a 1.ª cena que imprime um
  # diagnóstico a seguir ao anúncio perde-o — e um diagnóstico que não se vê lê-se como um
  # motor que não corre. Foi o que custou uma volta ao HUD (TOP-20 #20).
  # ⛔⛔ **E o filtro é por LINHA, logo um anúncio MULTI-LINHA perde tudo menos a 1.ª** (2026-09-18):
  #    um roteiro em passos numerados (a lei do `CLAUDE.md` §0.8) imprime `[x-smoke]` só no
  #    cabeçalho. ⇒ imprime-se do 1.º acerto até à linha antes do acerto SEGUINTE, com tecto.
  awk '/[Ss]moke\]/ { n++ } n >= 1 && n <= 2 && linhas < 24 { print; linhas++ }' "$TMP/app.log" \
    || true
  # ⭐ **E o log INTEIRO por pedido** (`FOTO_LOG=<ficheiro>`): a janela do `awk` acima mostra do 1.º
  # anúncio ao 2.º, logo **todo diagnóstico posterior é invisível** — e este roteiro é lido por quem
  # está a caçar um defeito. *Foi essa janela que escondeu a auto-conferência do HUD, e é a 3.ª vez
  # que ela custa uma volta.* ⛔ Ela FICA como está: encher o terminal com centenas de linhas é o
  # defeito oposto, e o dono lê este resumo.
  if [ -n "${FOTO_LOG:-}" ]; then
    cp "$TMP/app.log" "$FOTO_LOG" && echo "log: $FOTO_LOG"
  fi
else
  echo "sem foto — log do app:" >&2
  tail -20 "$TMP/app.log" >&2 || true
  exit 1
fi

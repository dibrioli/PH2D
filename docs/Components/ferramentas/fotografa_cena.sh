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
#   ⛔⛔ E o HOME isolado levava o CACHE DE SHADERS do driver (`$HOME/.cache/nvidia`) VAZIO a cada
#     foto (medido 15/10, cena 142, espera 12, 1930x1040): o 1.º quadro so' saía depois de o NVVM
#     compilar todos os pipelines — 10,9 a 33,7 s com load 14–146 —, e ate' la' a janela X11 devolve
#     ZEROS ao `import`: **9 de 12 fotos PRETAS**, do tamanho certo e da janela certa (`_NET_WM_PID` =
#     o app). Com um cache PERSISTENTE so' das fotos o 1.º quadro sai aos 0,68–0,76 s (3/3, ate' load
#     115); o do dono (`~/.cache/nvidia`) nao serve: outros processos aparam-no (frio 3 de 4 vezes).
#     ⇒ cache proprio (`FOTO_CACHE_SHADERS`), e a espera conta-se do 1.º quadro ACESO, nunca do
#     arranque; a janela e' a do PID do app; foto toda a zeros e' RECUSA, nunca entregue.
#     `FOTO_GUARDA_TMP=1` guarda a pasta da sessao (logs, kwin.log, sessao.sh) para diagnostico.
set -euo pipefail

ENV_KV="${1:?VAR=valor}"
LARG="${2:?largura}"
ALT="${3:?altura}"
SAIDA="${4:?saida.png}"
ESPERA="${5:-9}"
# Rajada: FOTO_RAJADA=<n> fotos na MESMA sessao, uma a cada FOTO_INTERVALO=<s> (1 por omissao): a
# 1.a e' <saida.png>, as seguintes <saida>_02.png, … — dois instantes de uma cena que corre sem
# que o arranque (que varia com a carga) mude a distancia entre eles.
RAJADA="${FOTO_RAJADA:-1}"
INTERVALO="${FOTO_INTERVALO:-1}"
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
# ⚠️ O PERFIL, e ele é `smoke` por omissão — a lei do `CLAUDE.md` §5 (o `release` optimiza a shell
# num só thread e cada correcção pós-smoke custava 161 s contra 3).
#
# ⛔⛔ **Mas um smoke de PERFORMANCE não pode ser fotografado em `smoke`**, e a mesma linha do §5
# di-lo: *«o `smoke` não tem LTO e corre mais devagar»* ⇒ ali as duas colunas de um A/B mediriam o
# perfil de build e não a lei. ⇒ `FOTO_PERFIL=release` para essas, e para mais nenhuma.
PERFIL="${FOTO_PERFIL:-smoke}"
BIN="$RAIZ/target/$PERFIL/ph2d-host-desktop"

# ⛔⛔⛔ ELE CONSTRÓI, e a razão é um defeito MEDIDO (2026-09-18, `line/Vector`).
#
# Até aqui este roteiro só EXIGIA um binário (`[ -x "$BIN" ] || exit 2`) — logo ele fotografava, em
# silêncio, a build de ontem. O modo de falha é o pior que há: uma linha editou a cena, fotografou
# "antes" e "depois" de um corte, viu **as duas fotos IGUAIS** e escreveu na mensagem do commit que
# a cura estava verificada. *Duas fotos do MESMO binário são sempre iguais, e lêem-se exactamente
# como «a mudança não estragou nada».*
#
# ⚠️ O custo é o do `cargo` a não fazer nada: `2,5 s` incrementais com a árvore quente (medido) —
# contra uma foto que afirma sobre código que não corre. E ele fica **por dentro**, e não num passo
# que quem chama tem de se lembrar de escrever: *uma ferramenta que depende de um passo lembrado
# tem o defeito de volta no dia em que alguém a chamar à pressa.*
echo "[foto] a construir o binario ($PERFIL) — senao esta foto e' da build de ontem…" >&2
( cd "$RAIZ" && cargo build -q -p ph2d-host-desktop --profile "$PERFIL" ) \
  || { echo "[foto] a build falhou — nao fotografo uma arvore que nao compila" >&2; exit 2; }
[ -x "$BIN" ] || { echo "falta o binario: cargo build -p ph2d-host-desktop --profile $PERFIL" >&2; exit 2; }

# ⛔⛔⛔ **ELE NÃO CONSTRÓI, LOGO SEM ISTO FOTOGRAFA O PROGRAMA ANTERIOR** (medido 2026-09-19).
# A cena do golpe foi curada, fotografada DUAS vezes com o binário de antes da cura, e as duas fotos
# mostraram o defeito já curado — uma delas com o log a reproduzi-lo à letra. *Uma sonda que mede
# outro programa é pior do que nenhuma: ela não fica em silêncio, ela CONFIRMA.*
# ⚠️ **A recusa é ALTA e não um aviso:** o modo de falha que ela substitui é mudo por construção
# (a foto sai, é bonita, e é de outra build), e este roteiro é lido por quem já está a caçar um
# defeito. ⭐ E a régua é o PRÓPRIO ficheiro mais novo, com o nome dele na mensagem — nunca um
# relógio: compara os dois mtimes e não tem calibração para envelhecer.
#
# ⛔⛔ A POPULAÇÃO é a que ENTRA no binário: a dep-info que o cargo escreve ao lado dele
# (`$BIN.d`, `alvo: dep1 dep2 …`, espaço escapado `\ `; reescrita a cada build, mesmo no-op).
# A régua antiga (`find crates shells -name '*.rs' -newer`) recusou duas vezes em 15–16/10 por um
# `*_tests.rs` (fora do binário, build no-op CORRECTA) — e a «cura» de um agente foi `touch` no
# binário, que desarma o guarda inteiro. Ela também era cega aos `.wgsl`/`include_str!`, que a
# dep-info lista. Sem dep-info (ou mais velha que o binário) não há régua: RECUSA.
DEP="$BIN.d"
if [ ! -s "$DEP" ] || [ "$DEP" -ot "$BIN" ]; then
  echo "RECUSA: sem regua — a dep-info $DEP falta ou e' mais velha que o binario" >&2
  exit 2
fi
mapfile -t DEPS < <(awk -v alvo="$BIN" '
  BEGIN { gsub(/ /, "\001", alvo); alvo = alvo ":" }
  { gsub(/\\ /, "\001") }
  $1 == alvo { for (i = 2; i <= NF; i++) { f = $i; gsub(/\001/, " ", f); print f } }' "$DEP")
if [ "${#DEPS[@]}" -eq 0 ]; then
  echo "RECUSA: sem regua — a dep-info $DEP nao lista nenhum ficheiro para $BIN" >&2
  exit 2
fi
NOVO=""
for f in "${DEPS[@]}"; do
  case "$f" in /*) ;; *) f="$RAIZ/$f" ;; esac
  if [ ! -e "$f" ]; then NOVO="$f (APAGADO)"; break; fi
  if [ "$f" -nt "$BIN" ]; then NOVO="$f"; break; fi
done
if [ -n "$NOVO" ]; then
  echo "RECUSA: ha' codigo do binario mais novo que ele (ex.: ${NOVO#"$RAIZ/"})" >&2
  echo "  a foto seria do programa ANTERIOR. Corra primeiro:" >&2
  echo "  bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile $PERFIL" >&2
  exit 2
fi

TMP="$(mktemp -d)"
if [ "${FOTO_GUARDA_TMP:-0}" = 1 ]; then
  trap 'echo "[foto] sessao guardada em $TMP" >&2' EXIT
else
  trap 'rm -rf "$TMP"' EXIT
fi
mkdir -p "$TMP/cfg"
# Cache de shaders PERSISTENTE e so' das fotos (ver «AS TRES ARMADILHAS»): no HOME REAL de quem chama,
# lido ANTES do isolamento. E a saida apaga-se ja': uma foto VELHA no mesmo caminho nunca se le' como esta.
CACHE="${FOTO_CACHE_SHADERS:-${XDG_CACHE_HOME:-$HOME/.cache}/ph2d-fotografa}"
mkdir -p "$CACHE"
PRAZO_QUADRO="${FOTO_PRAZO_QUADRO:-90}"
PRAZO_KWIN="$(awk -v a="$PRAZO_QUADRO" -v b="$ESPERA" 'BEGIN { print int(a + b + 30) }')"
rm -f "$SAIDA"
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
#
# O binário corre SEM o cargo: o tecto do cache de shaders do driver é o do [env] do .cargo/config.toml
# (16 GiB) — com o de fábrica (1 GiB) o driver APARA o cache partilhado e despeja a luz dos outros (W5p).
# ⚠️ O cache é o `$CACHE` das fotos (o HOME isolado deixava-o VAZIO: as fotos pretas de 15/10).
#
# ⭐⭐ O GUARDA lê o FONTE do heredoc, ANTES de o gerar: no ficheiro gerado a crase já correu e sumiu
# (14/10: o comentário acima vivia lá dentro, executava «[env]: comando não encontrado» e o guarda
# antigo, que lia o gerado, nunca o via). Recusa crase e «$(» sem barra antes.
if awk -v alvo='cat > "$TMP/sessao.sh" <<EOF' '
  $0 == alvo { dentro = 1; next }
  dentro && $0 == "EOF" { dentro = 0 }
  dentro && /(^|[^\\])(`|\$\()/ { print FILENAME ":" FNR ": " $0; achou = 1 }
  END { exit !achou }' "$0" >&2; then
  echo "RECUSA: crase ou \$( sem barra no heredoc <<EOF do roteiro — corre na GERACAO" >&2
  exit 4
fi
cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a tela virtual (DISPLAY=\$DISPLAY WAYLAND_DISPLAY=\$WAYLAND_DISPLAY)" > "$TMP/log"
  exit 3
fi
cd "$RAIZ"
mkdir -p "$TMP/home"
env -u WAYLAND_DISPLAY HOME="$TMP/home" __GL_SHADER_DISK_CACHE_SIZE=17179869184 \
  __GL_SHADER_DISK_CACHE_PATH="$CACHE" MESA_SHADER_CACHE_DIR="$CACHE/mesa" $ENV_KV \
  PH2D_EXIT_AFTER_FRAMES=100000 "$BIN" > "$TMP/app.log" 2>&1 &
APP=\$!
janela() {
  for w in \$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do
    xprop -display "\$DISPLAY" -id "\$w" _NET_WM_PID | grep -qx "_NET_WM_PID(CARDINAL) = \$APP" && echo "\$w"
  done | tail -1
}
aceso() {
  [ "\$(magick "\$1" -format '%[fx:maxima>0]' info: 2>/dev/null)" = 1 ]
}
WIN=
T0=\$(date +%s%N)
while :; do
  if ! kill -0 \$APP 2>/dev/null; then
    echo "RECUSA: o app morreu antes do 1.o quadro (ver o log do app)" >> "$TMP/log"; exit 5
  fi
  if [ \$SECONDS -ge $PRAZO_QUADRO ]; then
    echo "RECUSA: em ${PRAZO_QUADRO}s a janela do app (pid \$APP, id \${WIN:-nenhuma}) nunca apresentou" \
      "um quadro - o import so' devolveria ZEROS. FOTO_PRAZO_QUADRO sobe o prazo." >> "$TMP/log"
    kill -9 \$APP 2>/dev/null; exit 6
  fi
  sleep 0.25
  WIN=\$(janela)
  [ -n "\$WIN" ] || continue
  import -display "\$DISPLAY" -window "\$WIN" "$TMP/sonda.png" 2>/dev/null || continue
  aceso "$TMP/sonda.png" && break
done
MS=\$(( (\$(date +%s%N) - T0) / 1000000 ))
echo "[foto] 1.o quadro aceso aos \${MS} ms (janela \$WIN do pid \$APP); espero ${ESPERA}s" >> "$TMP/log"
sleep $ESPERA
import -display "\$DISPLAY" -window "\$WIN" "$SAIDA" >> "$TMP/log" 2>&1
if ! aceso "$SAIDA"; then
  echo "RECUSA: a foto saiu toda a ZEROS depois de a janela ter acendido - nao a entrego" >> "$TMP/log"
  rm -f "$SAIDA"
fi
for i in \$(seq 2 $RAJADA); do
  sleep $INTERVALO
  import -display "\$DISPLAY" -window "\$WIN" "${SAIDA%.png}_\$(printf %02d \$i).png" >> "$TMP/log" 2>&1
done
kill -9 \$APP 2>/dev/null
EOF
chmod +x "$TMP/sessao.sh"

# E o guarda do GERADO fica: uma crase escapada (\`) sobrevive até o ficheiro, e o kwin corre-o.
if grep -q '`' "$TMP/sessao.sh"; then
  echo "RECUSA: o roteiro gerado tem uma CRASE — ela e' substituicao de comando no heredoc <<EOF" >&2
  exit 4
fi

XDG_CONFIG_HOME="$TMP/cfg" timeout "$PRAZO_KWIN" kwin_wayland --virtual --width "$LARG" --height "$ALT" \
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
  # ⭐⭐⭐ **E o log INTEIRO fica SEMPRE ao lado da foto** — nunca por pedido.
  #
  # ⚠️⚠️ **As duas linhas desta rodada curaram a MESMA janela de três linhas, cada uma à sua
  # maneira, e a integração de 20/09 ficou com o melhor das duas:** o resumo do `awk` acima é da
  # `line/components` (encher o terminal com centenas de linhas é o defeito OPOSTO, e o dono lê
  # este resumo), e *escrever o log ao lado da foto* é da `line/Vector` — ⭐ ela é estritamente
  # melhor que o `FOTO_LOG=<ficheiro>` que aqui estava: **um opt-in só serve quem já sabe que
  # precisa dele**, e quem está a caçar um defeito descobre isso depois de a foto já ter saído.
  #
  # ⚠️ E o ficheiro leva o log **inteiro**, não só o que casa `smoke]`: uma RECUSA do app não traz
  # essa etiqueta, e é exactamente ela que explica uma foto vazia.
  cp "$TMP/app.log" "${FOTO_LOG:-${SAIDA%.png}.log}" 2>/dev/null \
    && echo "roteiro: ${FOTO_LOG:-${SAIDA%.png}.log}"
else
  echo "sem foto — log do app:" >&2
  tail -20 "$TMP/app.log" >&2 || true
  exit 1
fi

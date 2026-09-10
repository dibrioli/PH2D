#!/usr/bin/env bash
# cargo-test-narrow.sh <crate> [extra cargo args] — o irmão que faltava do
# `cargo-check-narrow.sh`, com a PORTA DO CHECK na frente.
#
# ## WHY this exists (medido em 2026-08-18 sobre 101 sessões, 4,01 GB de transcript)
#
# `cargo test` responde por **80,5% de todo o relógio de Bash do repo** — 59.215
# invocações, **340,6 h**. Nenhum outro comando chega a 3%. Dentro desse número há
# três defeitos, e nenhum deles é «o teste demora»:
#
# 1. ⭐ **4.414 corridas (44 por sessão, 6,3 h) NUNCA CHEGARAM A RODAR UM TESTE** —
#    o crate não compilou. A mesma pergunta custa **1,08 s** de mediana no
#    `cargo check -p` contra **5,17 s** no `cargo test -p`. Por isso este script
#    roda `check` PRIMEIRO e sai cedo: economia medida de ~5,0 h no corpus.
#    (É a §2 do CLAUDE.md executável: *`test -p` não responde «minha edição entrou?»*.)
#
# 2. **98,9% das invocações carregam um filtro escrito à mão** — `| grep -E "^error"`,
#    `| grep "test result"`, `| tail -20`. São **177.063 usos em 19.760 formas
#    distintas**: cada forma foi reescrita ~9 vezes, e sozinhas são **42% de todos
#    os padrões de busca do repo** (728 por sessão).
#
# 3. **O filtro é onde a resposta se perde.** 3.252 corridas (32/sessão) pagaram o
#    relógio inteiro e não devolveram veredito: 1.847 trouxeram só o panic com o
#    `test result:` cortado, e **797 devolveram literalmente NADA** (1,6 h de
#    máquina por zero informação). ⚠️ Um filtro que come o agregado transforma
#    «falhou» em «não sei», e «não sei» custa outra corrida.
#
# ## O contrato
#
# Saída ≤ ~20 linhas. **Exit code distinto por desfecho**, que é o que um `| head`
# destrói (o `cargo-check-narrow.sh` já registra essa armadilha):
#
#   0  verde — todos passaram
#   1  vermelho de TESTE — compilou, algum teste falhou
#   2  vermelho de COMPILAÇÃO — nenhum teste chegou a correr
#   3  PENDUROU — bateu no tecto de tempo (ver `PH2D_TEST_TIMEOUT` abaixo)
#
# ⚠️ **Isto NÃO é o gate de fechamento.** O gate batched continua sendo
# `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria, 1× sobre o
# diff acumulado (CLAUDE.md §2). Este script é para a corrida dirigida — o gate
# red-first, a prova de mutação, o «este teste passou?».
#
# USO
#   bash scripts/cargo-test-narrow.sh ph2d-timeline
#   bash scripts/cargo-test-narrow.sh ph2d-sculpt3d --release -- --ignored
#   PH2D_SKIP_CHECK=1 bash scripts/cargo-test-narrow.sh ph2d-ecs   # pula a porta
set -uo pipefail

crate="${1:?uso: cargo-test-narrow.sh <crate> [args extra do cargo]}"
shift || true

command -v jq >/dev/null 2>&1 || { echo "✗ jq não encontrado (o script depende dele)"; exit 2; }

# --- A PORTA: compila? ---------------------------------------------------------
# ⚠️ Só é honesta porque devolve o exit code REAL. Um `cargo check | head` reporta
# o exit do `head` e um vermelho passa por verde.
if [ "${PH2D_SKIP_CHECK:-0}" != "1" ]; then
  chk="$(mktemp)"; trap 'rm -f "$chk"' EXIT
  # `--all-targets` de propósito: o teste vive em `tests/`, e um `check -p` sem
  # isto compila só a lib — daria verde com o teste quebrado, que é o modo de
  # falha exato que esta porta existe para impedir.
  cargo check -p "$crate" --all-targets --message-format=json "$@" >"$chk" 2>/dev/null
  nerr=$(jq -rs '[ .[] | select(.reason=="compiler-message") | .message
                       | select(.level=="error") ] | length' "$chk" 2>/dev/null || echo 0)
  if [ "${nerr:-0}" -gt 0 ]; then
    echo "✗ NÃO COMPILA — nenhum teste correu ($nerr erro(s)):"
    jq -rs '[ .[] | select(.reason=="compiler-message") | .message
                  | select(.level=="error") | .rendered ] | .[0:3] | .[]' "$chk"
    [ "$nerr" -gt 3 ] && echo "... (${nerr} erros no total; mostrando os 3 primeiros)"
    exit 2
  fi
fi

# --- ORFÃOS de uma corrida anterior --------------------------------------------
#
# ⛔⛔ **Matar quem lançou NÃO mata o teste** (medido 2026-09-10): o binário de teste é
# reparentado ao `systemd --user` e continua a girar, invisível ao cargo e a quem o lançou. Dois
# binários de `ph2d-poly2d` ficaram vivos **1h55m** a **299,7% de CPU cada** — ~6 dos 32 núcleos —
# depois de os processos que os lançaram terem morrido. Ninguém ia colher o resultado.
#
# ⚠️ E o `rm -rf target/*/incremental` do fecho de linha (DIRETRIZ §1.5.9) **apaga ficheiros e não
# mata processo nenhum**.
#
# ⚠️⚠️ **A classe de caracteres no padrão NÃO é enfeite: sem ela isto MATA-SE A SI MESMO.** O
# padrão aparece na linha de comando deste próprio script, o `pkill -f` casa-a, e o script morre
# com exit 144 — medido, e duas vezes.
#
# ⛔⛔⛔ **E ele só mata o REPARENTADO** — medido na 1.ª redacção, que matou **27** binários de uma
# corrida de `nextest` VIVA que estava a decorrer ao lado. *Um binário de teste desta árvore não é
# um órfão: um órfão é aquele cujo LANÇADOR morreu.* Numa máquina onde seis linhas correm em
# paralelo, matar por caminho é sabotar o vizinho.
#
# ⇒ o discriminador é o PAI: `PPID 1` ou `systemd` (para onde o kernel reparenta quem ficou sem
# dono). Com o pai vivo, a corrida é de alguém — e fica em paz.
raiz="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
orfaos=""
for pid in $(pgrep -f "${raiz}/target/[^/]*/deps/[a-z_]" 2>/dev/null || true); do
  ppid="$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')"
  pai="$(ps -o comm= -p "${ppid:-0}" 2>/dev/null | tr -d ' ')"
  case "${ppid:-0}:${pai:-}" in
    1:* | *:systemd) orfaos="$orfaos $pid" ;;
  esac
done
if [ -n "${orfaos// /}" ]; then
  echo "⚠ binário(s) de teste ÓRFÃOS desta árvore (o lançador morreu) — a matar:"
  # shellcheck disable=SC2086
  ps -o pid=,etime=,pcpu=,args= -p $orfaos 2>/dev/null | cut -c1-140 | sed 's/^/  /' | head -5
  # shellcheck disable=SC2086
  kill -9 $orfaos 2>/dev/null || true
fi

# --- os testes -----------------------------------------------------------------
#
# ⛔⛔⛔ **O TECTO DE TEMPO, e ele é do RECURSO «a máquina»** (medido 2026-09-10): antes disto
# **nenhuma corrida de teste deste repo tinha tecto**, excepto uma crate. O `slow-timeout` do
# `.config/nextest.toml` vive dentro de um `[[profile.default.overrides]]` com
# `filter = 'package(ph2d-asset-cooker)'` — o próprio ficheiro escreve a lei (*«a hang never returns
# to trigger a retry»*) e depois cerca-a com o nome de uma crate.
#
# ⚠️ Ele NÃO é um limite de lentidão: é a conversão de uma **pendura** (infinita) numa **falha**.
# Por isso é largo — quem aperta o relógio é o gate, não esta porta.
#
# `PH2D_TEST_TIMEOUT` afina-o (segundos); `0` desliga.
tempo="${PH2D_TEST_TIMEOUT:-900}"
out="$(mktemp)"
trap 'rm -f "${chk:-}" "$out"' EXIT
if [ "$tempo" = "0" ]; then
  cargo test -p "$crate" "$@" >"$out" 2>&1
  rc=$?
else
  timeout --kill-after=10s "$tempo" cargo test -p "$crate" "$@" >"$out" 2>&1
  rc=$?
fi
if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
  echo "✗ $crate — PENDUROU (nenhum teste terminou em ${tempo}s)"
  echo
  echo "  Um teste de algoritmo geométrico/topológico que passa de 60 s em debug é uma PENDURA,"
  echo "  não lentidão. Os últimos nomes que o libtest imprimiu:"
  grep -E "^test [a-z]" "$out" | tail -5 | sed 's/^/    /'
  echo
  printf '  %s\n' "⚠ Confira que não sobrou binário vivo:"
  printf '      pgrep -af %s\n' "$raiz/target/[^/]*/deps/"
  exit 3
fi

# O agregado do libtest, todas as suítes (`cargo test -p` roda lib + cada
# arquivo de tests/ como binário próprio, então há VÁRIAS linhas `test result:`).
pass=$(grep -oE '^test result: ok\. [0-9]+ passed' "$out" | grep -oE '[0-9]+' | awk '{s+=$1} END{print s+0}')
fail=$(grep -oE '[0-9]+ failed' "$out" | grep -oE '[0-9]+' | awk '{s+=$1} END{print s+0}')
ign=$(grep -oE '[0-9]+ ignored' "$out" | grep -oE '[0-9]+' | awk '{s+=$1} END{print s+0}')

if [ "$rc" -eq 0 ]; then
  echo "✓ $crate — ${pass} passaram · ${ign} ignorados"
  exit 0
fi

echo "✗ $crate — ${fail} falharam · ${pass} passaram · ${ign} ignorados"
echo
# Cada falha com o seu panic e o file:line — é isto que os filtros à mão perdiam.
grep -E "^---- .* stdout ----|panicked at |^assertion|^  left:|^ right:" "$out" \
  | sed -E 's/^---- (.*) stdout ----/\n▸ \1/' | head -40
nomes=$(sed -n '/^failures:$/,/^$/p' "$out" | grep -E '^    ' | head -12)
if [ -n "$nomes" ]; then
  echo
  echo "falharam:"
  echo "$nomes"
fi
echo
echo "(saída inteira: cargo test -p $crate $*)"
exit 1

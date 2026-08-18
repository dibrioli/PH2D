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

# --- os testes -----------------------------------------------------------------
out="$(mktemp)"
trap 'rm -f "${chk:-}" "$out"' EXIT
cargo test -p "$crate" "$@" >"$out" 2>&1
rc=$?

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

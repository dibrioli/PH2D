#!/usr/bin/env bash
# nextest-impacted.sh — run nextest ONLY for crates changed vs BASE plus their
# reverse-dependents, ALWAYS including the HR-5 determinism golden. For the fast
# module-close loop. NOT a replacement for the final gate:
#   ./scripts/ship.sh (cargo nextest run --workspace --cargo-profile ci-test)
# stays the unconditional pre-push gate, and the CI cross-OS determinism job
# (spike.yml `determinism`/`determinism-compare`) cannot be reproduced locally.
#
# Usage:  ./scripts/nextest-impacted.sh            # vs origin/main
#         BASE=<ref> ./scripts/nextest-impacted.sh # vs custom ref
set -uo pipefail

# `ci-test` só roda em BATCH (uma ou duas vezes por jornada, sobre a workspace
# inteira), então compilação incremental não colhe nada ali e paga 11 GB — o
# número que o CLAUDE.md §2 nomeia, MEDIDO em 2026-08-16. Esta é a única coisa
# que este script executa, então o `export` é seguro: o `cargo check -p` do
# inner loop (perfil `dev`, outro processo) fica em paz, de propósito.
#
# ⚠️ A regra vivia SÓ no CLAUDE.md, e o integrador de 16/08 a seguiu à mão e
# viu os 11 GB aparecerem na mesma — porque quem compila é ESTE script. Uma
# regra fora do caminho de quem a executa é uma regra que não existe.
# ⚠️ E nesta workstation o `target/` é symlink para /dev/shm: o diretório de
# build É RAM, e o número sai do mesmo orçamento em que o app corre.
export CARGO_INCREMENTAL=0

BASE="${BASE:-origin/main}"

# ⛔⛔ **UMA ÁRVORE SUJA FAZ ESTE GATE SAIR VERDE SEM MEDIR NADA.**
#
# `git diff --name-only "$BASE..."` compara **commits**: trabalho por commitar é
# invisível para ele. Numa worktree com o módulo inteiro ainda no working tree,
# `CHANGED` sai vazio, o script cai no ramo "no crate changes" e roda **4 testes** —
# saindo VERDE.
#
# ⚠️ **Medido em 2026-08-23**, ao fechar uma jornada: `Summary [0.003s] 4 tests run`
# sobre um diff de 13 ficheiros e 959 linhas. Depois de commitar, o mesmo comando
# correu **3.842**. É a MESMA doença que a cura de 2026-08-19 atacou (o `sed` do
# prefixo de caminho) noutra roupa, e partilha com ela o modo de falha caro:
# *não avisa, não falha, fica verde.*
#
# ⚠️ E o `BASE` por omissão é `origin/main`, que numa jornada Modo L pode estar
# ATRÁS do `main` local. Se o gate parecer medir de menos, é o primeiro sítio a olhar.
if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
    echo "⚠️  [nextest-impacted] A ÁRVORE ESTÁ SUJA, e este gate compara COMMITS."
    echo "    O que não está commitado NÃO entra na conta — commite antes de fechar"
    echo "    a linha, ou passe BASE=<ref> conscientemente."
fi

# ⚠️ **O PACOTE SE DERIVA DO `cargo metadata`, NUNCA DO PREFIXO DO CAMINHO.**
#
# Isto era `sed -n 's#^crates/\([^/]*\)/.*#\1#p'`, e a consequência foi medida em
# 2026-08-19: um diff inteiramente em `shells/desktop/src/` produzia `CHANGED`
# **vazio**, o script caía no ramo "no crate changes" e rodava **4 testes** —
# saindo VERDE. `shells/desktop` é o shell inteiro (sculpt3d, undo, persistência,
# `input_dispatch`), e todo fechamento de linha cujo diff fosse só de shell correu
# com essa cobertura. O mesmo valia para `tools/` e `tests/`.
#
# ⚠️ E o dir NÃO é o nome do pacote fora de `crates/` (`shells/desktop` é o
# `ph2d-host-desktop`), então a convenção antiga não podia simplesmente ser
# estendida a mais um prefixo: a fonte tem de ser o manifesto.
CHANGED=$(git diff --name-only "${BASE}"... 2>/dev/null | python3 -c '
import json, os, subprocess, sys

changed = [l.strip() for l in sys.stdin if l.strip()]
if not changed:
    sys.exit(0)

meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"],
    capture_output=True, text=True, check=True).stdout)
root = meta["workspace_root"]

# (dir relativo à raiz, nome do pacote), do mais LONGO para o mais curto: uma
# crate aninhada dentro de outra pasta de crate tem de vencer a de fora.
pkgs = []
for p in meta["packages"]:
    d = os.path.relpath(os.path.dirname(p["manifest_path"]), root)
    pkgs.append(("" if d == "." else d + "/", p["name"]))
pkgs.sort(key=lambda x: -len(x[0]))

hit, rust_missed = set(), []
for f in changed:
    for prefix, name in pkgs:
        if f.startswith(prefix):
            hit.add(name)
            break
    else:
        if f.endswith(".rs"):
            rust_missed.append(f)

# ⚠️ Silêncio é o defeito que esta função existe para não repetir: um `.rs` que
# não caia em pacote nenhum é exatamente o caso que ficava verde por omissão.
for f in rust_missed:
    print(f"[nextest-impacted] AVISO: {f} nao pertence a pacote nenhum do workspace",
          file=sys.stderr)

for name in sorted(hit):
    print(name)
' | sort -u)

if [ -z "$CHANGED" ]; then
    echo "[nextest-impacted] no crate changes vs ${BASE}; running determinism golden only."
    # `binary(...)` matches the TEST BINARY name (tests/transform_determinism.rs);
    # `test(...)` matches individual fn NAMES (cross_os_golden_hash_pinned, …) which
    # do NOT contain "transform_determinism" → it silently matched 0 (false-green).
    exec cargo nextest run -E 'binary(transform_determinism)' --cargo-profile ci-test
fi

# rdeps(set) = the crate AND everything that depends on it (the real "impacted"
# set). `-p X` does NOT include dependents — filterset rdeps() does.
EXPR=""
for c in $CHANGED; do
    EXPR="${EXPR:+$EXPR + }rdeps(${c})"
done

# MANDATORY determinism net: force the HR-5 golden so a transitive math/dep edit
# can never silently skip it locally. `binary(...)` (NOT `test(...)`) — the golden
# lives in the `transform_determinism` test binary; its fn names don't contain that
# string, so `test(/transform_determinism/)` matched 0 and gave a false-green gate.
EXPR="$EXPR + binary(transform_determinism)"

echo "[nextest-impacted] changed: $(echo "$CHANGED" | tr '\n' ' ')"
echo "[nextest-impacted] -E '$EXPR'"
# If a changed dir name is not a real package, the filterset errors out (non-zero)
# — that surfaces the dir→package mismatch rather than silently under-testing.
#
# ⛔⛔ **`"$@"` NÃO É COSMÉTICO — sem ele os argumentos eram ENGOLIDOS EM SILÊNCIO.**
#
# ⚠️ **Medido em 2026-08-23:** o `CLAUDE.md` §5.0 manda usar `--no-fail-fast` («senão
# suítes inteiras nunca chegam a correr»), e este script era invocado com ele **dezenas
# de vezes por jornada** sem nunca o repassar. As corridas verdes escondiam-no — só
# quando um teste falha é que a diferença aparece —, e a corrida que a expôs parou em
# `1 689 de 3 853` com **2 164 testes por correr**, exactamente o cenário que a regra do
# roteador existe para evitar.
#
# *Um script que aceita argumentos e os deita fora falha do mesmo modo que o
# `str.replace` que não casa: sem erro, sem aviso, com ar de sucesso.*
exec cargo nextest run -E "$EXPR" --cargo-profile ci-test "$@"

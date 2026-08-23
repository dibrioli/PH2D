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

# ⚠️ **O `nextest` CANCELA na primeira falha, e o CLAUDE.md §5.0 manda usar `--no-fail-fast`** —
# mas quem executa a regra é ESTE script, e ele não a oferecia: uma corrida que tropeça numa das
# flakes de relógio conhecidas deixa **7 mil** testes por correr, e quem lê o resultado não fica a
# saber se o resto está verde. *Uma regra fora do caminho de quem a executa é uma regra que não
# existe* — a mesma lição que pôs o `CARGO_INCREMENTAL=0` aqui dentro.
#
#   NO_FAIL_FAST=1 ./scripts/nextest-impacted.sh   # corre tudo e lista TODAS as falhas
FAIL_MODE=()
if [ -n "${NO_FAIL_FAST:-}" ]; then
    FAIL_MODE=(--no-fail-fast)
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
# ⚠️ **O TRABALHO NÃO COMMITADO TAMBÉM CONTA, e ele era INVISÍVEL aqui.**
#
# `git diff A...` compara COMMITS: um arquivo editado e ainda por commitar não aparece. Medido em
# 2026-08-23: um fecho com quatro arquivos da `ph2d-ui-state` por commitar correu 10 418 testes
# **sem correr um único** dos daquela crate — ela não estava no conjunto, e nada disse. É a mesma
# família do defeito que o bloco abaixo narra (o `sed` do prefixo), noutra roupa: o gate saía VERDE
# por não medir.
#
# ⇒ o conjunto é a UNIÃO do diff commitado com o que o `git status` mostra (modificado,
# adicionado, por rastrear).
CHANGED=$( { git diff --name-only "${BASE}"... 2>/dev/null; git status --porcelain 2>/dev/null | cut -c4-; } | sed 's/^"//;s/"$//' | sort -u | python3 -c '
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
    exec cargo nextest run "${FAIL_MODE[@]}" -E 'binary(transform_determinism)' --cargo-profile ci-test "$@"
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

# ⭐⭐⭐ REDE OBRIGATÓRIA nº 2: os gates de ARQUITETURA que varrem a WORKSPACE INTEIRA.
#
# ⛔ MEDIDO 2026-08-23: o `workspace_src_files_under_loc_cap` vive em
# `ph2d-editor-core` e lê TODOS os `crates/*/src/**`. Um diff em `ph2d-quadfill` não
# toca aquele crate, logo o `rdeps()` não o inclui — e o ficheiro passou dos 700 LOC
# com o gate batched a devolver "3 861 verdes". A regressão só apareceu quando eu
# corri o `wc -l` à mão.
#
# ⚠️ É um ponto cego ESTRUTURAL, não um esquecimento: um gate cujo domínio é a
# workspace inteira nunca pertence ao fecho de dependências de quem o viola. *Um
# selector de impacto por dependências é cego a toda regra global.*
EXPR="$EXPR + binary(architecture_workspace_file_loc_cap) + binary(file_loc_caps)"

echo "[nextest-impacted] changed: $(echo "$CHANGED" | tr '\n' ' ')"
echo "[nextest-impacted] -E '$EXPR'"
# If a changed dir name is not a real package, the filterset errors out (non-zero)
# — that surfaces the dir→package mismatch rather than silently under-testing.
#
# ⛔⛔ **`"$@"` NÃO É COSMÉTICO — sem ele os argumentos eram ENGOLIDOS EM SILÊNCIO.**
# Medido em 2026-08-23: o script era invocado com `--no-fail-fast` dezenas de vezes por
# jornada sem nunca o repassar; a corrida que o expôs parou em 1 689 de 3 853. O
# `NO_FAIL_FAST=1` acima e o repasse de argumentos são COMPLEMENTARES, não redundantes.
exec cargo nextest run "${FAIL_MODE[@]}" -E "$EXPR" --cargo-profile ci-test "$@"

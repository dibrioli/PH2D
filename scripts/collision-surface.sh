#!/usr/bin/env bash
# collision-surface.sh — a superfície de colisão de uma linha, MEDIDA de uma vez.
#
# WHY this exists: medido em 2026-08-18 nas 6 maiores sessões de integração, o integrador
# gasta ~1.000 `grep`/`sed` POR INTEGRAÇÃO redescobrindo sempre as MESMAS perguntas —
# PROJECT_SCHEMA (717x), tetos de LOC (2.388x), números de ADR (360x), contagens de contrato
# congelado (402x), Cargo.lock, marcadores de conflito, ids de scrollbar. São 36 mil chamadas
# de navegação por sessão, 62 MB puxados para o contexto, e uma sessão de integração acaba
# arrastando ~8,4 M tokens para uma janela de 1 M — ou seja, ~8 compactações POR CONSTRUÇÃO,
# e o integrador re-descobre depois de cada uma.
#
# Isto é conferência REPETITIVA, não investigação: a mesma lista, toda vez. Este script a
# responde numa chamada e numa tabela. O que sobra para a leitura humana é o que de facto
# exige julgamento (conflito de mesmo-símbolo, decisão de produto).
#
# ⚠️ Ele NÃO substitui `scripts/foundational-integrate.sh` — aquele é o gate MECÂNICO
# (rebase → registry → build → testes → land). Este é o mapa que se lê ANTES, para saber
# onde os números vão colidir. Os dois são complementares.
#
# ⚠️ Um número que soma entre linhas se CONTA, nunca se escolhe — e a colisão passa MUDA
# quando duas linhas escrevem o MESMO literal (o git não sabe o que o número significa).
# É por isso que a tabela mostra o valor da SUA árvore E o da base, lado a lado.
#
# USAGE
#   bash scripts/collision-surface.sh            # contra 'main'
#   bash scripts/collision-surface.sh <ref>      # contra outra base
#
# Só lê (git + arquivos). Não escreve nada, não muda branch, seguro a qualquer momento.

set -uo pipefail
BASE="${1:-main}"
cd "$(git rev-parse --show-toplevel)" || exit 1

git rev-parse --verify -q "$BASE" >/dev/null || { echo "✗ base '$BASE' não existe"; exit 1; }
MB=$(git merge-base HEAD "$BASE" 2>/dev/null || echo "$BASE")

echo
echo "SUPERFÍCIE DE COLISÃO — $(git branch --show-current 2>/dev/null || echo HEAD) contra $BASE"
echo "  merge-base $(git rev-parse --short "$MB")   ·   $(git rev-list --count "$MB"..HEAD) commit(s)   ·   $(git diff --name-only "$MB"..HEAD | wc -l) arquivo(s)"
echo "───────────────────────────────────────────────────────────────────────────────"

# --- helper: valor de uma const aqui e na base ---------------------------------
# ⚠️ o valor vem DEPOIS do último '=' ou ',' — um `head -1` ingênuo devolve o "32" de `u32`
# (bug real, apanhado pelo controle em 2026-08-18: os cinco schemas liam 32).
val() { sed -E 's/.*[=,] *([0-9]+).*/\1/; t; s/.*/—/'; }

# ⚠️ <arquivos> é uma LISTA de candidatos: a `line/physics` PARTIU o `project.rs` em 15/08 e
# levou a const para o irmão `project_schema.rs`. Uma worktree forkada antes disso ainda a tem
# no arquivo antigo — e um scanner de um caminho só reporta "—", que se lê como "não existe".
num() {  # num <rótulo> "<arquivo> [arquivo2 ...]" <regex>
  local rot="$1" arqs="$2" re="$3" aqui base marca arq
  for arq in $arqs; do
    [ -z "${aqui:-}" ] && aqui=$(grep -hoE "$re" "$arq" 2>/dev/null | head -1 | val)
    [ "${aqui:-—}" = "—" ] && aqui=""
    [ -z "${base:-}" ] && base=$(git show "$MB:$arq" 2>/dev/null | grep -hoE "$re" | head -1 | val)
    [ "${base:-—}" = "—" ] && base=""
  done
  [ -z "$aqui" ] && aqui="—"; [ -z "$base" ] && base="—"
  if [ "$aqui" = "$base" ]; then marca="  "; else marca="⚠ "; fi
  printf "  %s%-34s %6s   (base: %s)\n" "$marca" "$rot" "$aqui" "$base"
}

echo "▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios"
num "PROJECT_SCHEMA"        "shells/desktop/src/project_schema.rs shells/desktop/src/project.rs" 'PROJECT_SCHEMA: u32 = [0-9]+'
# a tripla mostra os TRÊS números, não um: ela é (PROJECT_SCHEMA, FLIP, VEC_SCENE) e o
# defeito que ela existe para pegar é justamente um deles andar sozinho.
TRI_A=$(grep -hoE '\([0-9]+, *[0-9]+, *[0-9]+\)' shells/desktop/src/project_schema_tests.rs 2>/dev/null | head -1)
TRI_B=$(git show "$MB:shells/desktop/src/project_schema_tests.rs" 2>/dev/null | grep -hoE '\([0-9]+, *[0-9]+, *[0-9]+\)' | head -1)
[ "$TRI_A" = "$TRI_B" ] && M="  " || M="⚠ "
printf "  %s%-34s %6s   (base: %s)\n" "$M" "  └ tripla do gate" "${TRI_A:-—}" "${TRI_B:-—}"
num "VEC_SCENE_SCHEMA"      "crates/ph2d-vec-scene/src/lib.rs"           'VEC_SCENE_SCHEMA_VERSION: u32 = [0-9]+'
num "FLIP_SCHEMA"           "crates/ph2d-flip/src/lib.rs"                'FLIP_SCHEMA_VERSION: u32 = [0-9]+'
num "DOC_VERSION (timeline)" "crates/ph2d-timeline/src/doc.rs"           'DOC_VERSION: u32 = [0-9]+'
if git diff --name-only "$MB"..HEAD | grep -qE 'shells/desktop/src/project'; then
  echo "  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;"
  echo "      um degrau escrito no arquivo errado funde LIMPO e evapora."
fi

echo
echo "▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate"
num "ph2d-ecs"    "crates/ph2d-ecs/src/scene/registry.rs" 'reg\.len\(\), *[0-9]+'
num "ph2d-render (espelho)" "crates/ph2d-render/src/registry.rs" 'reg\.len\(\), *[0-9]+'
num "ph2d-script (espelho)" "crates/ph2d-script/src/registry.rs" 'reg\.len\(\), *[0-9]+'

echo
echo "▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR"
for f in crates/ph2d-nodegraph/src/node.rs crates/ph2d-editor-core/src/tool.rs; do
  if [ -f "$f" ]; then
    n=$(git diff --numstat "$MB"..HEAD -- "$f" 2>/dev/null | awk '{print $1+$2}')
    [ -z "$n" ] && printf "    %-46s intocado\n" "$f" || printf "  ⚠ %-46s %s linha(s) MUDADAS\n" "$f" "$n"
  fi
done

echo
echo "▸ ADR — número escolhido numa linha paralela é PROVISÓRIO"
ult=$(ls docs/architecture/decisions/ 2>/dev/null | grep -oE '^[0-9]{4}' | sort -n | tail -1)
novos=$(git diff --name-only --diff-filter=A "$MB"..HEAD -- 'docs/architecture/decisions/*' 2>/dev/null | grep -oE '[0-9]{4}' | tr '\n' ' ')
printf "    último no disco: %s   próximo livre: %04d\n" "$ult" $((10#$ult + 1))
[ -n "$novos" ] && echo "  ⚠ esta linha cria ADR: $novos  — reconte contra o main do dia" \
                || echo "    esta linha não cria ADR ⇒ fora de toda disputa de número"

echo
echo "▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não"
add=$(git diff "$MB"..HEAD -- Cargo.lock 2>/dev/null | grep -cE '^\+name = ')
[ "$add" -gt 0 ] && { echo "  ⚠ $add pacote(s) '+name' novo(s):"
  git diff "$MB"..HEAD -- Cargo.lock | grep -E '^\+name = ' | sed 's/^+name = /      /'; } \
  || echo "    nenhum '+name' novo"

echo
echo "▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê"
hit=$(git diff --name-only "$MB"..HEAD | while read -r f; do
        [ -f "$f" ] && grep -lE '^(<<<<<<<|\|\|\|\|\|\|\||>>>>>>>)' "$f" 2>/dev/null; done | sort -u)
[ -n "$hit" ] && { echo "  ✗ ENCONTRADOS:"; echo "$hit" | sed 's/^/      /'; } || echo "    nenhum nos arquivos da linha"

echo
echo "▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)"
# ⚠️ ESPELHA `is_excluded()` do gate real (architecture_workspace_file_loc_cap.rs).
# Sem isto o script grita sobre arquivos que o gate NUNCA cobra — medido em
# 2026-08-18: acusou `ph2d-gpu-cook/tests/gpu_cpu_parity.rs` (6.247 linhas) como
# violação, e `tests/` é a PRIMEIRA exclusão do gate. Um mapa que inventa alarme
# gasta o mesmo tempo que o grep que ele veio poupar, e ensina a ignorá-lo.
over=0
while read -r f; do
  [ -f "$f" ] || continue
  case "$f" in *.rs) ;; *) continue ;; esac
  case "$f" in
    */tests/*|*/tests.rs|crates/ph2d-tool-runtime/src/*|*registry-init*) continue ;;
  esac
  n=$(wc -l < "$f")
  cap=700
  case "$f" in
    crates/ph2d-editor-core/src/widget/*) cap=500 ;;
    crates/ph2d-panel-*|shells/desktop/*)  cap=600 ;;
  esac
  # o escape numerado do gate: uma entrada na allowlist congela o valor daquele dia
  if grep -qE "^\s*\(\"${f#crates/}\"|// ph2d-loc-cap:" "$f" 2>/dev/null; then
    printf "    %5d / %d   %s  (tem marcador/allowlist — confira o valor congelado)\n" "$n" "$cap" "$f"
    continue
  fi
  [ "$n" -gt "$cap" ] && { printf "  ✗ %5d / %d   %s\n" "$n" "$cap" "$f"; over=$((over+1)); }
done < <(git diff --name-only "$MB"..HEAD)
[ "$over" -eq 0 ] && echo "    nenhum arquivo da linha passa do teto"

echo "───────────────────────────────────────────────────────────────────────────────"
echo "  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;"
echo "     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana."
echo

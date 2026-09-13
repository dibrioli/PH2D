#!/usr/bin/env bash
# W148 — as PROVAS DE MUTAÇÃO da cache de fitas contra o casco (docs/3DModeling/12).
#
# Cada mutação: cópia → substituir a agulha (a CONTAGEM tem de ser exactamente 1, senão ela não é
# aplicada e o script DIZ) → correr os gates que a devem matar → restaurar → `touch` (um `mv` devolve
# o mtime antigo, e o cargo guardaria o build DA MUTAÇÃO — project-memory, 2026-09).
#
# Quatro veredictos, e nenhum é lido como outro:
#   MORTA        — compilou, correu testes, e algum reprovou
#   SOBREVIVEU   — compilou, correu testes, e todos passaram
#   NÃO COMPILOU — não é prova de nada (a mutação partiu a compilação)
#   FILTRO VAZIO — o filtro casou zero testes (um «sobreviveu» de zero testes é um byte, não uma prova)
#
# Uso (da raiz da worktree): bash docs/3DModeling/ferramentas/w148_provas_de_mutacao.sh [saída]
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
out="${1:-target/w148_mutacoes.txt}"
mkdir -p target
: >"$out"

mutate() {
  local name="$1" file="$2" old="$3" new="$4" pkg="$5" filter="$6"
  local log="target/w148_${name}.log"
  cp "$file" "$file.w148bak" || return
  if ! python3 - "$file" "$old" "$new" <<'PY'
import sys
path, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
src = open(path, encoding="utf-8").read()
n = src.count(old)
if n != 1:
    sys.exit(f"{path}: a agulha casou {n} vezes (esperava 1)")
open(path, "w", encoding="utf-8").write(src.replace(old, new))
PY
  then
    echo "✗ AGULHA     $name — a mutação NÃO foi aplicada" | tee -a "$out"
    mv "$file.w148bak" "$file"
    touch "$file"
    return
  fi
  cargo nextest run -p "$pkg" --cargo-profile ci-test --no-fail-fast -E "$filter" >"$log" 2>&1
  local rc=$?
  mv "$file.w148bak" "$file"
  touch "$file"
  local summary
  summary=$(grep -E 'Summary' "$log" | tail -1 | sed -E 's/^ +//')
  # ⚠️ NÃO `^error:` — o nextest imprime `error: test run failed` quando um teste REPROVA, e a 1.ª
  # redacção deste script lia assim quatro mutações MORTAS como «não compilou».
  if grep -qE '^error\[E[0-9]+\]|could not compile' "$log"; then
    echo "NÃO COMPILOU $name ($log)" | tee -a "$out"
  elif grep -qE ' 0 tests run' "$log"; then
    echo "FILTRO VAZIO $name — $filter" | tee -a "$out"
  elif [ "$rc" -eq 0 ]; then
    echo "SOBREVIVEU   $name — $summary" | tee -a "$out"
  else
    local killers
    killers=$(grep -E '^ +FAIL ' "$log" | sed -E 's/.* ([a-z_0-9:]+)$/\1/' | sort -u | tr '\n' ' ')
    echo "MORTA        $name — por: $killers — $summary" | tee -a "$out"
  fi
}

# M1 — o `contains` só pergunta pela caixa local (o casco é ignorado).
mutate m1_contains_so_caixa \
  crates/ph2d-field-eval/src/region_hulls.rs \
  '        polygon.iter().all(|p| in_convex(*p, outer))' \
  '        polygon.iter().all(|_| true) || in_convex([0.0; 2], outer)' \
  ph2d-field-eval \
  'test(the_containment_of_hulls_is_sound_in_any_pose)'

# M2 — a cache serve uma fita de CASCO só pela contenção de caixa.
mutate m2_serve_so_por_caixa \
  crates/ph2d-field-render/src/tape_cache.rs \
  '                    .is_none_or(|h| query.is_some_and(|q| h.contains(q)))' \
  '                    .is_none_or(|_| query.is_some() || true)' \
  ph2d-field-render \
  'test(the_hull_cache_never_changes_the_image_of_a_posed_piece) | test(the_cache_never_changes_the_image) | test(a_cached_tape_is_never_served_to_another_document) | test(a_hull_tape_is_never_served_to_a_region_its_hulls_do_not_contain)'
# ⚠️ Os três primeiros gates deste filtro são de IMAGEM, e a M2 SOBREVIVEU a eles na 1.ª corrida (13/09):
# o `dmax` do corte é um majorante generoso, e uma fita servida um pouco fora do casco quase sempre
# ainda tem a aresta vencedora. O quarto gateia a PROPRIEDADE no `TapeCache::get`, e foi escrito por
# causa dela.

# M3 — a consulta chega sem os cascos dela (a cache nunca acerta).
mutate m3_consulta_sem_cascos \
  crates/ph2d-field-render/src/tiles.rs \
  '                .map(|_| rc.hulls(doc, r.lo, r.hi, &r.pts));' \
  '                .map(|_| ph2d_field_eval::RegionHulls::default());' \
  ph2d-field-render \
  'test(a_drag_stops_recompiling_the_tapes_it_already_has) | test(the_hull_cache_never_changes_the_image_of_a_posed_piece)'

# M4 — a folga da caixa perde o lado de baixo.
mutate m4_folga_sem_lado_de_baixo \
  crates/ph2d-field-render/src/tape_cache.rs \
  '        out.0[k] = lo[k] - pad + pad * u[k];' \
  '        out.0[k] = lo[k] + pad * u[k];' \
  ph2d-field-render \
  'test(the_padded_region_still_contains_its_query)'

# M5 — o mapa mundo→local esquece a pose do pai (a descida que subiu para o `affine.rs`).
mutate m5_mapa_sem_pose_do_pai \
  crates/ph2d-field-eval/src/affine.rs \
  '                to_local[ci] = Some(Affine::of(doc.nodes()[ci].xform).after(parent));' \
  '                to_local[ci] = Some(Affine::of(doc.nodes()[ci].xform)); let _ = parent;' \
  ph2d-field-eval \
  'all()'

echo "── fim · $(cat /proc/loadavg)" | tee -a "$out"

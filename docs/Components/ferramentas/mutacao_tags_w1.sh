#!/usr/bin/env bash
# Provas de mutação da W1 das Tags (line/components, 2026-09-13) — v2, depois da cura da chave guardada.
#
# Correr (de dentro da árvore): bash docs/Components/ferramentas/mutacao_tags_w1.sh
#   22 mutações, cada uma com o seu CONTROLO. Medido em 2026-09-13: 22 de 22 SANGRARAM
#   (`docs/Components/08_plano_tags.md` §5.2). ⚠️ Corre `cargo test` em série e MUTA ficheiros do
#   produto — não corra outra coisa na mesma árvore enquanto ele corre; o `trap` restaura no fim, e
#   uma âncora que deixe de casar (o código mudou) diz-se «âncora não casou», nunca «sobreviveu».
#   Os logs vão para `$PH2D_MUT_DIR` (ou um `mktemp -d`, impresso no fim).
#
# Para cada mutação: (1) CONTROLO — o filtro corre >= 1 teste e passa na árvore limpa (senão um
# filtro que casa zero imprimiria «sobreviveu»); (2) muta, com a âncora a ocorrer EXACTAMENTE uma
# vez (um str.replace que não casa é no-op silencioso); (3) corre; (4) restaura do backup e dá
# `touch` (restaurar por cópia devolve mtime antigo e o cargo guardaria o build da mutação).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
export LC_ALL=C
S=${PH2D_MUT_DIR:-$(mktemp -d)}
mkdir -p "$S/bak"
echo "logs em $S"
declare -a BACKED=()

restore_all() {
  for f in "${BACKED[@]:-}"; do
    [ -n "$f" ] || continue
    local b="$S/bak/$(echo "$f" | tr '/' '_')"
    [ -f "$b" ] && cp "$b" "$f" && touch "$f"
  done
}
trap restore_all EXIT INT TERM

backup() {
  local f=$1 b="$S/bak/$(echo "$1" | tr '/' '_')"
  if [ ! -f "$b" ]; then cp "$f" "$b"; BACKED+=("$f"); fi
}

replace() { # ficheiro velho novo
  python3 - "$1" "$2" "$3" <<'PY'
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p, encoding="utf-8").read()
n = s.count(old)
if n != 1:
    print(f"ANCORA {n}x em {p}: {old!r}")
    sys.exit(3)
open(p, "w", encoding="utf-8").write(s.replace(old, new))
PY
}

corre() { # crate alvo filtro log -> "passed failed" ou "ERRO"
  local crate=$1 target=$2 filter=$3 log=$4
  cargo test -q -p "$crate" $target -- "$filter" >"$log" 2>&1
  local p f
  p=$(grep -oE '[0-9]+ passed' "$log" | awk '{s+=$1} END {print s+0}')
  f=$(grep -oE '[0-9]+ failed' "$log" | awk '{s+=$1} END {print s+0}')
  if ! grep -q 'test result' "$log"; then echo "ERRO"; else echo "$p $f"; fi
}

N=0
mutacao() { # nome ficheiro velho novo crate alvo filtro
  local nome=$1 file=$2 old=$3 new=$4 crate=$5 target=$6 filter=$7
  N=$((N+1))
  local ctl mut
  ctl=$(corre "$crate" "$target" "$filter" "$S/$N-controlo.log")
  if [ "$ctl" = "ERRO" ] || [ "${ctl%% *}" -lt 1 ] || [ "${ctl##* }" -ne 0 ]; then
    echo "✗ $nome — CONTROLO inválido ($ctl) · filtro «$filter»"; return
  fi
  backup "$file"
  if ! replace "$file" "$old" "$new"; then
    echo "✗ $nome — âncora não casou"; return
  fi
  mut=$(corre "$crate" "$target" "$filter" "$S/$N-mutante.log")
  local b="$S/bak/$(echo "$file" | tr '/' '_')"
  cp "$b" "$file" && touch "$file"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}

E=crates/ph2d-ecs/src/tags.rs
T=crates/ph2d-tags/src/lib.rs
F=crates/ph2d-label-fold/src/lib.rs
L=crates/ph2d-label-path/src/lib.rs
C=crates/ph2d-asset-index/src/catalog.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

mutacao "M1 belongs lê só o conjunto directo" "$E" \
  '    tags.0.iter().any(|&m| tree.reaches(q, TagId(m)))' '    tags.0.contains(&q.0)' \
  ph2d-ecs --lib belonging_reaches_the_subtree_and_never_the_sibling_root
mutacao "M1b tagged lê só o conjunto directo, contra o oráculo do Blender" "$E" \
  '        .filter(|(_, t, _)| t.meets(&reach))' '        .filter(|(_, t, _)| t.0.contains(&q.0))' \
  ph2d-ecs "--test it" the_hierarchy_is_the_one_blender_measures::the_hierarchy_is_the_one_blender_measures
mutacao "M2 tagged sem ordenar pela identidade" "$E" \
  '    hits.sort_unstable_by_key(|&(s, e)| (s, e.index()));' '' \
  ph2d-ecs --lib the_query_order_is_the_identity_not_the_archetype
mutacao "M3 scrub tira só a primeira tag apagada" "$E" \
  '            t.0.retain(|&i| !ids.contains(&TagId(i)));' '            t.0.retain(|&i| Some(TagId(i)) != ids.first().copied());' \
  ph2d-ecs --lib deleting_a_tag_takes_its_subtree_and_the_membership_in_one_gesture
mutacao "M4 scrub carimba todos os objectos" "$E" \
  '            t.0.retain(|&i| !ids.contains(&TagId(i)));
        }
    }
    touched.len()' \
  '            t.0.retain(|&i| !ids.contains(&TagId(i)));
        }
    }
    let todos: Vec<Entity> = world.try_query::<(Entity, &Tags)>().map(|mut q| q.iter(world).map(|(e, _)| e).collect()).unwrap_or_default();
    for e in todos { if let Some(mut t) = world.get_mut::<Tags>(e) { use bevy_ecs::change_detection::DetectChangesMut; t.set_changed(); } }
    touched.len()' \
  ph2d-ecs --lib scrubbing_only_stamps_the_objects_it_changes
mutacao "M5 remap não aplicado" "$E" \
  '    if remap.is_empty() {' '    if true {' \
  ph2d-ecs --lib a_document_with_twins_merges_them_and_keeps_every_member
mutacao "M6 o id reservado 0 entra" "$E" \
  '        id.0 != 0 && self.0.insert(id.0)' '        self.0.insert(id.0)' \
  ph2d-ecs --lib tags_bytes_do_not_depend_on_insertion_order

# M7 — o censo: um leitor plantado FORA da porta, numa crate que o `--test it` do ph2d-ecs não compila.
P=shells/desktop/src/prefab_exit.rs
N=$((N+1))
ctl=$(corre ph2d-ecs "--test it" only_the_door_reads_tags::only_the_door_reads_tags "$S/$N-controlo.log")
backup "$P"
printf '\nfn _plantado(t: &ph2d_ecs::Tags) -> usize {\n    t.direct_ids().count()\n}\n' >> "$P"
mut=$(corre ph2d-ecs "--test it" only_the_door_reads_tags::only_the_door_reads_tags "$S/$N-mutante.log")
cp "$S/bak/$(echo "$P" | tr '/' '_')" "$P" && touch "$P"
if [ "$mut" != "ERRO" ] && [ "${mut##* }" -ge 1 ] 2>/dev/null; then
  echo "✓ M7 censo apanha um leitor plantado na shell — SANGROU (controlo $ctl · mutante $mut)"
else
  echo "✗ M7 censo — SOBREVIVEU ou erro (controlo $ctl · mutante $mut)"
fi

mutacao "T1 a subárvore pára na própria tag" "$T" \
  '            .take_while(|e| path::key_is_self_or_descendant(&e.chave, raiz))' '            .take_while(|e| e.chave == *raiz)' \
  ph2d-tags --lib a_subtree_is_itself_and_its_descendants_and_never_a_text_neighbour
mutacao "T1b reaches sem a hierarquia" "$T" \
  '        path::key_is_self_or_descendant(&self.entradas[j].chave, &self.entradas[i].chave)
    }' \
  '        i == j
    }' \
  ph2d-ecs --lib belonging_reaches_the_subtree_and_never_the_sibling_root
mutacao "T2 a chave sem a dobra" "$T" \
  '    path::tree_order_key(p, fold)' '    path::tree_order_key(p, |s| {
        let _ = fold;
        s.to_string()
    })' \
  ph2d-tags --lib creating_a_tag_that_exists_folded_returns_the_existing_one
mutacao "T3 restore sem dizer para onde foi cada gémeo" "$T" \
  '                    remap.insert(t.id, existente.id);' '                    let _ = existente;' \
  ph2d-tags --lib a_document_with_twins_merges_them_and_says_where_each_went
mutacao "T4 uma recusa move a revisão" "$T" \
  '        if label.is_empty() {' '        self.touch();
        if label.is_empty() {' \
  ph2d-tags --lib a_refused_gesture_does_not_move_the_revision
mutacao "T5 mover para dentro da própria subárvore" "$T" \
  '            return Err(TagError::IntoOwnSubtree);' '            let _ = TagError::IntoOwnSubtree;' \
  ph2d-tags --lib moving_a_tag_carries_its_subtree_and_refuses_a_cycle
mutacao "T6 o restore não passa a grafia do pai ao filho" "$T" \
  '                t.path = format!("{grafia_do_pai}{}{}", path::SEP, niveis[niveis.len() - 1]);' \
  '                let _ = (&t.path, &grafia_do_pai);' \
  ph2d-tags --lib a_document_missing_an_ancestor_gets_it_with_the_parents_spelling
mutacao "T7 o restore não cria o ancestral em falta" "$T" \
  '                grafia_do_pai = caminho;' '                grafia_do_pai = caminho;
                por_chave.remove(prefixo);' \
  ph2d-tags --lib a_document_missing_an_ancestor_gets_it_with_the_parents_spelling

mutacao "F1 to_lowercase no lugar do case folding" "$F" \
  'CaseMapperBorrowed::new().fold_string(&decomposto)' 'decomposto.to_lowercase()' \
  ph2d-label-fold --lib the_fold_collapses_case_and_accents_and_nothing_else
mutacao "F2 as marcas não-espaçadoras ficam" "$F" \
  'categoria.get(c) != GeneralCategory::NonspacingMark' 'true' \
  ph2d-label-fold --lib the_fold_collapses_case_and_accents_and_nothing_else

mutacao "P1 descendente por prefixo de TEXTO (sem a fronteira de nível)" "$L" \
  '    let mut niveis = path.split(SEP);
    ancestor
        .split(SEP)
        .all(|a| niveis.next().is_some_and(|p| key(p) == key(a)))' \
  '    key(path).starts_with(&key(ancestor))' \
  ph2d-label-path --lib a_text_prefix_is_not_a_path_prefix
mutacao "P2 ordem pela string crua" "$L" \
  '    path.split(SEP).map(key).collect()' '    vec![key(path)]' \
  ph2d-label-path --lib the_tree_order_puts_a_parent_immediately_before_its_children
mutacao "P3 a forma por chaves compara só o último nível" "$L" \
  '    key.starts_with(ancestor)' '    key.last() == ancestor.last()' \
  ph2d-label-path --lib the_keyed_and_the_textual_descendant_checks_agree

mutacao "C1 renomear um catálogo para cima de um irmão" "$C" \
  'c.id != id && c.path == new_path' 'false' \
  ph2d-asset-index --lib renaming_a_catalog_onto_an_existing_sibling_is_refused

echo "load $(cut -d' ' -f1 /proc/loadavg)"
git status --short -- crates shells | grep -v '^??' || true

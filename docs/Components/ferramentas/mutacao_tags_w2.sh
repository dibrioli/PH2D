#!/usr/bin/env bash
# Provas de mutação da W2 das Tags (line/components, 2026-09-13) — o alvo por tag, o documento no
# projecto e a migração v128 → v129.
#
# Para cada mutação: (1) CONTROLO — o filtro corre >= 1 teste e passa na árvore limpa; (2) muta, com
# a âncora a ocorrer EXACTAMENTE uma vez; (3) corre; (4) restaura do backup e dá `touch`.
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

replace() {
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
  cp "$S/bak/$(echo "$file" | tr '/' '_')" "$file" && touch "$file"
  if [ "$mut" = "ERRO" ]; then
    echo "? $nome — o mutante NÃO COMPILOU (ver $S/$N-mutante.log)"
  elif [ "${mut##* }" -ge 1 ]; then
    echo "✓ $nome — SANGROU (controlo $ctl · mutante $mut)"
  else
    echo "✗ $nome — SOBREVIVEU (controlo $ctl · mutante $mut)"
  fi
}

E=crates/ph2d-ecs/src/signal_actions.rs
V=crates/ph2d-ecs/src/signal_actions_v1.rs
T=crates/ph2d-app-components/src/tags_doc.rs
R=crates/ph2d-ecs/src/scene/registry.rs
U=shells/desktop/src/undo.rs
P=shells/desktop/src/project_tags.rs
M=shells/desktop/src/project_migrate.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

mutacao "N1 o alvo por TAG cai no alvo por nome" "$E" \
  '        SignalTarget::Tagged(id) => crate::tags::tagged(world, tree, TagId(id)),' \
  '        SignalTarget::Tagged(id) => {
            let _ = (tree, id);
            target_of(world, source, &action.target).into_iter().collect()
        }' \
  ph2d-ecs --lib a_signal_to_a_tag_reaches_the_whole_subtree_and_the_sibling_root_stays
mutacao "N2 a migração aceita bytes a MAIS" "$V" \
  '    let Ok((antigo, [])) = postcard::take_from_bytes::<SignalActionsV1>(bytes) else {' \
  '    let Ok((antigo, _)) = postcard::take_from_bytes::<SignalActionsV1>(bytes) else {' \
  ph2d-ecs --lib a_blob_that_is_not_v128_is_left_alone
mutacao "N3 a migração escreve o alvo por TAG" "$V" \
  '                target_by: SignalTarget::Named,' '                target_by: SignalTarget::Tagged(0),' \
  ph2d-ecs --lib a_v128_signal_action_loads_as_a_named_target
mutacao "N4 o next_id não viaja" "$T" \
  '    postcard::to_allocvec(&(TAGS_DOC_VERSION, tags, tree.next_id())).unwrap_or_default()' \
  '    postcard::to_allocvec(&(TAGS_DOC_VERSION, tags, 0u64)).unwrap_or_default()' \
  ph2d-app-components --lib the_next_id_travels_so_a_deleted_tag_is_never_recycled
mutacao "N5 a versão do blob é ignorada" "$T" \
  '    if ver != TAGS_DOC_VERSION {' '    if false {' \
  ph2d-app-components --lib an_unreadable_or_foreign_blob_opens_an_empty_tree
mutacao "N6 a cache codifica a cada quadro" "$T" \
  '        if self.rev != Some(tree.revision()) {' '        if true {' \
  ph2d-app-components --lib the_cache_encodes_once_per_revision_and_again_after_invalidate
mutacao "N7 o invalidate é inerte" "$T" \
  '        self.rev = None;' '        let _ = &self.rev;' \
  ph2d-app-components --lib the_cache_forgets_the_previous_tree_even_when_the_revision_collides
# ⚠️ **A 1.ª redacção desta mutação SOBREVIVEU** (2026-09-13): ela punha o descritor do `Tags` em
# `Propagation::InstanceLocal` — e a política do descritor **não tem um único leitor na workspace**
# (`grep -rn 'Propagation::'` fora do catálogo: zero). O que faz a tag da receita chegar às cópias é
# o REGISTO: o passe compara os BYTES dos componentes registados. ⇒ a alavanca é esta.
mutacao "N8 o Tags sai do registo de componentes" "$R" \
  '    reg.register_default::<crate::Tags>("ph2d::ecs::Tags");' \
  '    let _ = "ph2d::ecs::Tags";' \
  ph2d-app-components --lib a_recipe_tag_reaches_every_copy
mutacao "N9 a captura larga a árvore de tags" "$U" \
  '            tags: tags.to_vec(),' '            tags: Vec::new(),' \
  ph2d-host-desktop --bins the_tag_tree_travels_in_the_project_and_through_undo
mutacao "N10 o load não reaponta a pertença dos gémeos" "$P" \
  '    let reapontados = ph2d_ecs::tags::remap(world, &remap);' \
  '    let reapontados = {
        let _ = &remap;
        0
    };' \
  ph2d-host-desktop --bins loading_a_document_with_twins_keeps_every_member
mutacao "N11 a travessia não reescreve o blob das acções" "$M" \
  '                row.components[slot].data = bytes;' '                let _ = bytes;' \
  ph2d-host-desktop --bins a_frozen_v128_file_migrates_its_signal_actions

echo "load $(cut -d' ' -f1 /proc/loadavg)"
git status --short -- crates shells | grep -v '^??' || true

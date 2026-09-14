#!/usr/bin/env bash
# Provas de mutação da W4 das Tags (line/components, 2026-09-14) — as duas portas novas
# (`ancestry` · `counts`), os cinco gestos sobre a árvore, a costura do painel e as duas cenas.
#
# ⚠️ Como na W3, elas correm DEPOIS do código: a prova de que os gates não são inertes é ESTA, e
# por isso ela cobre as asserções que CARREGAM a lei — não uma amostra.
#
# ⛔ Um `✗ SOBREVIVEU` é um gate que não afirma o que o doc-comment dele diz. Um `✗ CONTROLO
# inválido` é o FILTRO errado, não o produto.
#
# Para cada mutação: (1) CONTROLO — o filtro corre >= 1 teste e passa na árvore limpa; (2) muta, com
# a âncora a ocorrer EXACTAMENTE uma vez; (3) corre; (4) restaura do backup e dá `touch`.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
export LC_ALL=C
# ⛔ **`S` é a pasta dos LOGS.** Um alias de ficheiro chamado `S` no corpo faz o arnês escrever
# `…/tags.rs/1-controlo.log` e TODOS os controlos saem «inválidos» de uma vez — falha alto, mas
# o sintoma não aponta para a causa. Os aliases do corpo usam nomes de DUAS letras.
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


E=crates/ph2d-ecs/src/tags.rs
G=crates/ph2d-tags/src/lib.rs
SH_=shells/desktop/src/render_loop/tags_panel.rs
PN=crates/ph2d-panel-tags/src/rows.rs
SM=crates/ph2d-app-components/src/tags_smoke.rs
SE=crates/ph2d-panel-tags/src/seam.rs
HI=crates/ph2d-editor-core/src/interaction/dispatch/hierarchy.rs

echo "load $(cut -d' ' -f1 /proc/loadavg)"

# ── As duas PORTAS novas ──────────────────────────────────────────────────────
mutacao "M1 a ancestralidade devolve so' o proprio id" "$G" \
  '        (1..=chave.len())' '        (chave.len()..=chave.len())' \
  ph2d-tags --lib the_ancestry_of_a_tag_is_the_root_down_to_itself

mutacao "M2 a ancestralidade sai da PROFUNDIDADE e nao do caminho" "$G" \
  '            .filter_map(|n| self.por_chave(&chave[..n]))' \
  '            .filter_map(|n| self.entradas.iter().position(|e| e.chave.len() == n))' \
  ph2d-tags --lib a_sibling_root_is_never_an_ancestor

mutacao "M3 duas recusas partilham a frase" "$G" \
  '            Self::IntoOwnSubtree => "Cannot move a tag inside itself.",' \
  '            Self::IntoOwnSubtree => "That tag no longer exists.",' \
  ph2d-tags --lib every_refusal_says_why_in_words_and_no_two_say_the_same

mutacao "M4 a contagem le' so' a pertenca DIRECTA" "$E" \
  '            alcance.extend(tree.ancestry(m));' '            alcance.insert(m);' \
  ph2d-ecs --lib the_count_of_a_tag_is_what_the_query_returns_for_it

mutacao "M5 a contagem soma PERTENCAS em vez de OBJECTOS" "$E" \
  '        for q in &alcance {
            if let Some(n) = out.get_mut(q) {
                *n += 1;
            }
        }' \
  '        for m in tags.direct_ids() {
            for q in tree.ancestry(m) {
                if let Some(n) = out.get_mut(&q) {
                    *n += 1;
                }
            }
        }' \
  ph2d-ecs --lib an_object_in_two_levels_of_one_branch_is_counted_once

# ⛔⛔⛔ **Esta precisou de TRÊS redacções, e as duas primeiras ensinaram a mesma coisa: a lei tem
# DOIS guardas e nenhum é observável sozinho.** (1) atacar o `get_mut` do `counts` sobrevive —
# o `ancestry` já recusa um id que a árvore não conhece. (2) atacar o `ancestry` (devolver
# `vec![id]`) TAMBÉM sobrevive — o `out` só tem chaves das tags da árvore, e o `get_mut` recusa-o
# ali. ⇒ *uma mutação que ataque um guarda defendido pelo outro mede a redundância, não a lei.* A
# alavanca que fica escreve o id DIRECTO no mapa, saltando os dois de uma vez.
mutacao "M6 um id orfao ganha linha (os DOIS guardas de uma vez)" "$E" \
  '        for q in &alcance {
            if let Some(n) = out.get_mut(q) {
                *n += 1;
            }
        }' \
  '        for m in tags.direct_ids() {
            *out.entry(m).or_default() += 1;
        }' \
  ph2d-ecs --lib an_orphan_id_counts_for_nobody

mutacao "M7 a consulta desiste num mundo sem o componente" "$E" \
  '    let Some(mut query) = world.try_query::<&Tags>() else {
        return out;
    };' '    let Some(mut query) = world.try_query::<&Tags>() else {
        return BTreeMap::new();
    };' \
  ph2d-ecs --lib a_world_without_the_component_still_lists_every_tag_at_zero

mutacao "M8 a ordem da consulta deixa de ser a identidade" "$E" \
  '    hits.sort_unstable_by_key(|&(s, e)| (s, e.index()));' \
  '    hits.sort_unstable_by_key(|&(_, e)| e.index());' \
  ph2d-ecs --lib the_query_order_is_still_the_identity_after_the_cure

# ── Os cinco gestos sobre a ÁRVORE ────────────────────────────────────────────
mutacao "M9 a coluna do painel conta a pertenca DIRECTA" "$SH_" \
  '                members: membros.get(&t.id).copied().unwrap_or(0),' \
  '                members: 0,' \
  ph2d-host-desktop --bins the_panel_column_is_what_the_query_answers_for_each_tag

mutacao "M10 a linha nao diz quantas TAGS o apagar leva" "$SH_" \
  '                subtree: tree.subtree(t.id).len(),' '                subtree: 1,' \
  ph2d-host-desktop --bins a_row_says_how_many_tags_deleting_it_would_take

mutacao "M11 o nome de uma tag nova e' fixo" "$SH_" \
  '        let nome = if i == 1 {
            NOME_BASE.to_string()
        } else {
            format!("{NOME_BASE} {i}")
        };' '        let _ = i;
        let nome = NOME_BASE.to_string();' \
  ph2d-host-desktop --bins creating_picks_a_name_that_is_free_where_it_lands

mutacao "M12 apagar salta o scrub e deixa ids orfaos" "$SH_" \
  '            scrub(world, &saem);' '            let _ = &saem;' \
  ph2d-host-desktop --bins deleting_takes_the_subtree_and_the_membership_in_one_gesture

mutacao "M13 a recusa do MOVE fica MUDA" "$SH_" \
  '            Err(why) => TagEditOutcome::Refused { tag: *id, why },
        },
        TagTreeEdit::Delete { id } => {' \
  '            Err(_) => TagEditOutcome::Nothing,
        },
        TagTreeEdit::Delete { id } => {' \
  ph2d-host-desktop --bins a_refused_gesture_arrives_with_its_sentence_and_its_row

mutacao "M14 renomear re-atribui a pertenca" "$SH_" \
  '        TagTreeEdit::Rename { id, label } => match tree.rename(TagId(*id), label) {' \
  '        TagTreeEdit::Rename { id, label } => match tree.rename(TagId(*id), label).map(|()| {
            scrub(world, &tree.subtree(TagId(*id)));
        }) {' \
  ph2d-host-desktop --bins renaming_or_moving_never_touches_a_member

mutacao "M15 o Select devolve so' os ids DIRECTOS" "$SH_" \
  '            let alvos = tagged(world, tree, TagId(*id));' \
  '            let alvos: Vec<Entity> = world
                .iter_entities()
                .filter(|e| {
                    e.get::<ph2d_ecs::tags::Tags>()
                        .is_some_and(|t| t.direct_ids().any(|d| d.0 == *id))
                })
                .map(|e| e.id())
                .collect();' \
  ph2d-host-desktop --bins select_tagged_returns_the_whole_subtree_and_not_the_sibling_root

mutacao "M16 um gesto sobre uma tag morta mexe na arvore" "$SH_" \
  '            let saem = tree.delete(TagId(*id));
            if saem.is_empty() {
                return TagEditOutcome::Nothing;
            }' '            let saem = tree.delete(TagId(*id));' \
  ph2d-host-desktop --bins a_gesture_on_a_tag_that_is_gone_changes_nothing

# ── A COSTURA do painel ───────────────────────────────────────────────────────
mutacao "M17 a barra oferece verbos sem sujeito" "$PN" \
  '    let Some(row) = focused else { return out };' \
  '    let padrao = TagsPanelRow {
        id: 0,
        label: String::new(),
        depth: 1,
        members: 0,
        subtree: 1,
    };
    let row = focused.unwrap_or(&padrao);' \
  ph2d-panel-tags --lib the_bar_only_offers_verbs_that_have_a_subject

mutacao "M18 o Delete nao diz o estrago" "$PN" \
  '            format!("Delete ({} tags, {} objects)", row.subtree, row.members)' \
  '            "Delete".to_string()' \
  ph2d-panel-tags --lib the_delete_verb_spells_out_what_it_takes

mutacao "M19 as linhas nascem MORTAS sob o dedo" "$SE" \
  '        store.register_if_absent(crate::ids::row_id(t), InteractiveState::Plain);' \
  '        let _ = t;' \
  ph2d-panel-tags --test=it every_verb_of_a_chosen_row_reaches_the_bus

mutacao "M20 as linhas nao sao arrastaveis" "$SE" \
  '    store.set_panel_row_ids(
        PanelRowFamily::TagTree,
        tags.iter().map(|&t| crate::ids::row_id(t)).collect(),
    );' '    let _ = tags;' \
  ph2d-panel-tags --test=it dragging_a_row_onto_another_moves_it_inside

mutacao "M21 largar no pai que ja' se tem escreve um passo" "$SE" \
  '                && parent != parent_of(tag)' '' \
  ph2d-panel-tags --test=it dropping_onto_the_parent_it_already_has_writes_nothing

mutacao "M22 uma linha de OUTRA familia e' alvo" "$HI" \
  '        if store.panel_row_family(id) != Some(family) || id == dragged {' \
  '        if store.panel_row_family(id).is_none() || id == dragged {' \
  ph2d-panel-tags --test=it a_row_of_another_family_is_never_a_drop_target

mutacao "M23 o campo de renomear abre VAZIO" "$SE" \
  '            let seed = label_of(tag);
            open_rename(host.store_mut(), &seed);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if tag_of(id).is_some() => {' \
  '            open_rename(host.store_mut(), "");
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if tag_of(id).is_some() => {' \
  ph2d-panel-tags --test=it a_double_click_on_a_row_opens_the_rename_and_the_commit_reaches_the_bus

mutacao "M24 o Esc comita em vez de abortar" "$SE" \
  '        WidgetEvent::Cancel(id) if id == crate::ids::TAGS_RENAME_INPUT => {
            state.renaming = None;' \
  '        WidgetEvent::Cancel(id) if id == crate::ids::TAGS_RENAME_INPUT => {
            if let Some(tag) = state.renaming.take() {
                push(
                    host,
                    TagTreeEdit::Rename {
                        id: tag,
                        label: "x".into(),
                    },
                );
            }' \
  ph2d-panel-tags --test=it escape_aborts_the_rename_without_writing

# ── As duas CENAS ─────────────────────────────────────────────────────────────
mutacao "M25 a =1 perde um objecto" "$SM" \
  '        ("Bat B", 0.0, VOADOR_RGBA, a.flying),' '' \
  ph2d-app-components --lib the_first_scene_shows_the_counts_the_doc_promises

mutacao "M26 o cerebro aponta por NOME e nao por tag" "$SM" \
  '            target_by: SignalTarget::Tagged(a.enemy.0),' \
  '            target_by: SignalTarget::Named,' \
  ph2d-app-components --lib the_brain_aims_at_the_tag_and_names_nobody

# ⛔⛔ **Esta SOBREVIVEU na 1.ª corrida, e o defeito era do GATE, não da mutação:** ele derivava o
# filtro da fixtura (`a.player`) em vez de o LER do mundo, então media a lei da árvore e não a
# FIAÇÃO da cena. *Um gate que re-deriva o valor que devia ler não mede o produto.* Curado; a
# mutação ficou como estava.
mutacao "M27 a =2 filtra pela tag ERRADA" "$SM" \
  '        SignalTagFilter(a.player.0),' '        SignalTagFilter(a.enemy.0),' \
  ph2d-app-components --lib the_trap_lets_the_hero_through_and_ignores_the_goblin

mutacao "M28 a armadilha deixa de ser um sensor" "$SM" \
  '            is_sensor: true,
            ..cuboide(3.2, 0.35)' '            is_sensor: false,
            ..cuboide(3.2, 0.35)' \
  ph2d-app-components --lib the_trap_is_a_sensor_and_carries_its_filter

mutacao "M29 os dois corpos caem juntos" "$SM" \
  '        ("Hero", 12.0, HEROI_RGBA, a.player),' '        ("Hero", 6.5, HEROI_RGBA, a.player),' \
  ph2d-app-components --lib the_two_bodies_fall_one_at_a_time

mutacao "M30 o nivel desconhecido nao monta nada" "$SM" \
  '        _ => {
            cena_um(world, &a);
            1
        }' '        _ => 1,' \
  ph2d-app-components --lib an_unknown_level_falls_back_to_the_first_scene

mutacao "M31 o CENAS deixa de descrever os bracos" \
  crates/ph2d-app-components/src/tags_smoke.rs \
  'pub const CENAS: u32 = 2;' 'pub const CENAS: u32 = 3;' \
  ph2d-app-components --lib the_scene_count_is_counted_from_the_match_and_published

echo
echo "$N mutacoes · load $(cut -d' ' -f1 /proc/loadavg)"

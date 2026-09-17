//! **O retrato da HIERARQUIA** — irmão de [`super::snapshots`] pelo tecto de 600 LOC (HR-18).
//!
//! ⚠️ **O corte é por ASSUNTO:** lá *o que a selecção e o gizmo mostram*; aqui *a árvore*.
//! O ficheiro passou o tecto por ACUMULAÇÃO na rodada de 16/09 — nenhuma das seis linhas o
//! estoura sozinha (`CLAUDE.md` §5.0).

use super::*;

/// A Hierarquia viva: o instantâneo da cena sem as receitas, a seleção e o realce de proveniência carimbados nas
/// linhas, o selo booleano, e o cabeçalho da seleção.
#[cfg(feature = "panel-hierarchy")]
pub(super) fn publish_hierarchy(
    hero: &mut HeroScreen,
    hero_live: &mut Option<HeroLive>,
    hovered: Option<u64>,
    sim: &SimWorld,
    bool_badges: &std::collections::BTreeMap<u64, &'static str>,
) {
    // M14.4a: if live-bridge enabled, rebuild HierarchySnapshot
    // from SimWorld + push into HeroScreen BEFORE paint. The
    // snapshot's DFS visit order = hierarchy panel display
    // order. ADR-0029 Phase C.2: the typed Hierarchy panel owns the
    // live-entries thread-local; we call into the panel crate
    // directly here (the shell already gates `panel-hierarchy` via
    // feature).
    #[cfg(feature = "panel-hierarchy")]
    if let Some(live) = hero_live.as_mut() {
        crate::build_hierarchy_snapshot(
            sim.world(),
            &mut live.walk_state,
            &mut live.walk_scratch,
            &mut live.snapshot,
        );
        let (ordered, mut entries) = live.bridge.sync_from_snapshot(&live.snapshot);
        // ⭐⭐⭐ **UMA RECEITA NÃO É UMA LINHA DA CENA** (report do Enio, 2026-08-30: *«se apagar o
        // objeto de origem na hierarquia, some no painel»* + *«o original deve ficar apenas no
        // painel»*).
        //
        // O *Make Component* marca como receita o objecto escolhido e põe uma cópia no lugar. A
        // receita já não se DESENHA — mas continuava a ser uma linha da Hierarquia, e apagá-la de
        // lá destruía o asset. ⇒ ela sai da lista: o sítio dela é a biblioteca.
        //
        // ⭐ **E a lei é a MESMA do canvas**, não uma segunda: o
        // [`ph2d_entity_visibility::off_canvas::is_unedited_recipe`] já responde *«esta entidade é peça de uma
        // receita que ninguém está a editar agora?»*, e é o que o extract usa para não a desenhar.
        // Uma cópia dessa regra aqui divergiria no dia em que a edição de receita mudasse.
        //
        // ⚠️ **A receita que está a ser EDITADA volta à lista** — senão a forma do mestre seria
        // impossível de mudar, que é a metade que o `is_unedited_recipe` protege.
        let hidden_rows: std::collections::BTreeSet<ph2d_editor_core::NodeId> = ordered
            .iter()
            .copied()
            .filter(|id| {
                live.bridge.entity_for(*id).is_some_and(|bits| {
                    ph2d_entity_visibility::off_canvas::is_unedited_recipe(
                        sim.world(),
                        ph2d_ecs::Entity::from_bits(bits),
                    )
                })
            })
            .collect();
        let ordered: Vec<ph2d_editor_core::NodeId> = ordered
            .into_iter()
            .filter(|id| !hidden_rows.contains(id))
            .collect();
        entries.retain(|id, _| !hidden_rows.contains(id));
        // Fase 0 hotfix: mark every multi-selection row's
        // `HierarchyEntity.selected` BEFORE the panel paints, so
        // the row painter highlights N rows instead of just the
        // primary (paint.rs falls back to label match only when
        // `selected` is still false — fixture/demo path).
        for bits in hero.gizmo.iter_selected() {
            if let Some(node_id) = live.bridge.node_for(bits)
                && let Some(entry) = entries.get_mut(&node_id)
            {
                entry.selected = true;
            }
        }
        // ⭐ **O REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2) — carimbado pela porta ÚNICA
        // (`App::hovered_object`), que responde ao ponteiro venha ele do canvas ou desta lista.
        //
        // ⚠️ **UM objecto, uma linha.** A porta devolve `Option`, então duas linhas acesas ao mesmo
        // tempo não é exprimível daqui — e seria a assinatura de um segundo produtor a nascer.
        if let Some(bits) = hovered
            && let Some(node_id) = live.bridge.node_for(bits)
            && let Some(entry) = entries.get_mut(&node_id)
        {
            entry.hovered = true;
        }
        // Onda 1 hotfix: centralise the header label sync to the
        // multi-selection primary. Input handlers (canvas pick,
        // Hierarchy panel click, modifier override) used to stamp
        // hero.selection themselves and could race — e.g. Hierarchy
        // Cmd+click on row A stamped label="A" BEFORE the bus drain
        // toggled A out of the selection, leaving paint's label-match
        // fallback to re-highlight A. Snapshotting it once here
        // post-drain, against the post-toggle primary, removes the
        // race entirely.
        let primary_label = hero
            .gizmo
            .selection
            .and_then(|bits| live.bridge.node_for(bits))
            .and_then(|node| {
                entries
                    .get(&node)
                    .map(|e| (e.name.clone(), e.badge.clone()))
            });
        // **O SELO DO PAPEL BOOLEANO**, stampado DEPOIS do `primary_label` de propósito: o
        // cabeçalho usa o badge como *tipo* da seleção, e sobrescrevê-lo antes faria a
        // barra de cima dizer `SUB` onde sempre disse `ENT`. São dois consumidores do
        // mesmo campo, e só um deles pediu esta informação.
        if !bool_badges.is_empty() {
            for (&bits, &badge) in bool_badges {
                if let Some(node_id) = live.bridge.node_for(bits)
                    && let Some(entry) = entries.get_mut(&node_id)
                {
                    entry.badge = Some(badge.to_string());
                }
            }
        }
        ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &ordered, entries);
        if let Some((label, badge)) = primary_label {
            hero.selection = Some(ph2d_editor_core::HeroSelection {
                label,
                kind: badge.unwrap_or_else(|| tr("shell.snapshots.ent").to_string()),
                world_pos: (0.0, 0.0),
            });
        } else if hero.gizmo.selection.is_none() {
            hero.selection = None;
        }
    }
}

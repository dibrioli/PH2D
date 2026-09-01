//! Hierarchy intent dispatch phase.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Dispatches camera-reset + view-focus + 9 hierarchy
//! intents (visibility_toggle / reparent / duplicate / add_child /
//! reset_transform / delete / row_click / rename_seed / rename_commit)
//! using the locals pre-populated by the consolidated bus drain in
//! mod.rs. Returns `true` iff any dispatch pushed a toast.
//!
//! Behavior-preserving lift.

use crate::HeroLive;
use crate::hero_intents;
use ph2d_ecs::PresentWorld;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::action_bus::SelectModifier;
use ph2d_editor::screens::hero::HierReparentIntent;
use ph2d_editor::{HeroScreen, NodeId, Toast, ToastQueue, ViewFocusKind};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// Fase 0e: hierarchy-side multi-select intent collected from the
/// editor bus drain in `render_loop::mod`. The shell resolves
/// `row → entity_bits` here (panel crate has no bridge access) and
/// applies the matching `GizmoStateGroup` mutation. `Row` covers
/// click + cmd-click + double-click; `Range` covers shift-click and
/// walks the hierarchy order between the current primary's row and
/// the target row, calling `add_to_selection` on every entity in
/// between (inclusive).
#[derive(Copy, Clone, Debug)]
pub(super) enum HierarchySelectIntent {
    Row {
        row: NodeId,
        modifier: SelectModifier,
    },
    Range {
        row: NodeId,
    },
}

/// Dispatches camera-reset, view-focus, and 9 hierarchy intents.
/// Returns `true` if any dispatch pushed a toast.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    view_focus_kind: Option<ViewFocusKind>,
    visibility_toggle_row: Option<NodeId>,
    lock_toggle_row: Option<NodeId>,
    group_toggle_row: Option<NodeId>,
    reparent_intent: Option<HierReparentIntent>,
    duplicate_row: Option<NodeId>,
    add_child_row: Option<NodeId>,
    // ⭐ O botão `Add` do cabeçalho da Hierarquia (ADR-0166 / F3). Ver o bloco de `add_root`.
    add_root: bool,
    reset_transform_row: Option<NodeId>,
    // ⭐ *Revert to Master* (ADR-0164 / F4.4) — a linha cuja instância volta à receita.
    revert_to_master_row: Option<NodeId>,
    // ⭐ Os outros verbos de instância (ADR-0164 / F4.5).
    instance_verb_row: Option<(NodeId, crate::instance_verbs::Verb)>,
    // ⭐ **O MESMO verbo, endereçado por `StableId`** — o canal do navegador de assets (plano
    // `docs/Components/07`, wave A7). ⚠️ Ele mora aqui, ao lado do irmão, e chama a MESMA
    // `instance_verbs::drain`: a lei de instanciar continua com um dono só. O que muda é o
    // SUJEITO — o navegador endereça a receita pela identidade, e não por uma linha que ela nem
    // tem (uma receita está escondida da Hierarquia por construção).
    instance_verb_stable_id: Option<(u64, crate::instance_verbs::Verb, Option<[f32; 2]>)>,
    // ⭐⭐ **O menu de um CARTÃO da biblioteca** (etapa C) — o par `(endereço, verbo)`. Ele mora
    // aqui, junto dos outros verbos, porque é aqui que o `sim`, a voz e o gizmo estão os três
    // emprestados ao mesmo tempo; a decisão e as recusas vivem no `asset_card_verbs`.
    asset_card_verb: Option<(
        ph2d_editor::interaction::drag_payload::DragPayload,
        ph2d_editor::action_bus::AssetCardAction,
    )>,
    // ⭐ `célula do átlas → AssetId`, para o *Select users* achar uma imagem importada.
    atlas_assets: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    delete_row: Option<NodeId>,
    hierarchy_row_click: Option<NodeId>,
    hierarchy_select_intent: Option<HierarchySelectIntent>,
    rename_seed_row: Option<NodeId>,
    rename_commit: Option<(NodeId, String)>,
    hero: &mut HeroScreen,
    hero_live: &Option<HeroLive>,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    camera: &mut Camera2d,
    toasts: &mut ToastQueue,
    window_size: WindowSize,
    // O documento vetorial. ⚠️ Ele chega aqui porque a row **Duplicate** da Hierarchy tem de
    // duplicar uma FORMA pela mesma porta do botão do painel — ver o bloco de `duplicate_row`.
    vec_scene: &mut ph2d_vec_scene::VecScene,
    // ⭐ O mapa `path ⟺ entidade` (F4.6) — uma cópia profunda que clona um `VecPath` tem de
    // registar o par, senão o `vec_entities::sync` cunha uma segunda entidade para o clone.
    vec_entities: &mut crate::vec_entities::VecEntityMap,
    vec_history: &mut ph2d_vec_edit::History,
    vec_pen: &mut ph2d_vec_edit::PenTool,
    // Out: `(source_bits, new_bits)` of a sprite duplicate so the caller (which holds the painter +
    // renderer) can bake the source's live paint and give the copy an independent texture.
    duplicate_made: &mut Option<(u64, u64)>,
    // ⭐ O registo de componentes (ADR-0164 / F4.2) — a row **Duplicate** copia bytes de tipos que
    // esta shell não conhece, e a vtable dele é a única porta que sabe fazê-lo.
    registry: &ph2d_ecs::scene::ComponentRegistry,
    // ⭐ O eco do mestre (ADR-0164 / F4.4) — o *Revert* tem de o esquecer naquela chave, senão o
    // override renasce no quadro seguinte. Ver `instance_sync::revert_override`.
    echo: &mut crate::instance_sync::MasterEcho,
) -> bool {
    let mut title_dirty = false;

    // M14.4b.bis: drain pending camera-reset request from the VIEW
    // button (legacy "Zero" mode — kept around for shells that still
    // raise it).
    if hero.camera_reset_pending {
        hero.camera_reset_pending = false;
        *camera = Camera2d::default();
        toasts.push(Toast::info("View · Zero (camera reset)"));
        title_dirty = true;
    }
    // M14.7 polish: drain pending view-focus intent (F/Home key OR
    // VIEW button click). Per `ViewFocusKind`:
    //   - `Selected`: pan to gizmo_selection or (0,0).
    //   - `Camera`: pan to (0,0) until camera-object exists.
    //   - `All`: pan + zoom to fit all sprites.
    if let Some(kind) = view_focus_kind
        && hero_intents::drain_view_focus(
            kind,
            hero.gizmo.selection,
            present,
            camera,
            window_size,
            toasts,
        )
    {
        title_dirty = true;
    }
    // M14.6A: drain pending hierarchy visibility toggle — resolve row
    // NodeId → ECS Entity via the bridge, flip the `Visibility`
    // component on SimWorld.
    if let Some(row_id) = visibility_toggle_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row_id)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            let was_hidden = entry
                .get::<ph2d_ecs::Visibility>()
                .is_some_and(|v| v.hidden);
            entry.insert(ph2d_ecs::Visibility {
                hidden: !was_hidden,
            });
        }
    }
    // 2026-05-26 — drain Lock + GroupedChildren toggles. Mesma
    // pattern do visibility_toggle_row.
    if let Some(row_id) = lock_toggle_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row_id)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            if entry.get::<ph2d_ecs::Locked>().is_some() {
                entry.remove::<ph2d_ecs::Locked>();
            } else {
                entry.insert(ph2d_ecs::Locked);
            }
        }
    }
    if let Some(row_id) = group_toggle_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row_id)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            if entry.get::<ph2d_ecs::GroupedChildren>().is_some() {
                entry.remove::<ph2d_ecs::GroupedChildren>();
            } else {
                entry.insert(ph2d_ecs::GroupedChildren);
            }
        }
    }
    // M14.6B: drain pending hierarchy reparent intent — translate
    // dragged + new_parent NodeIds via the bridge, then either
    // `insert(ChildOf(p))` or remove the `ChildOf` component for a
    // root-level drop. With M14.7 polish (14.3 continuation) we also
    // honor `intent.before` to position the dragged entity at a
    // specific slot in the new parent's `Children` list — bevy_ecs
    // 0.18 `Children` preserves insertion order, so we rebuild the
    // ordering by re-inserting every relevant child's ChildOf in the
    // desired sequence.
    if let Some(intent) = reparent_intent
        && let Some(live) = hero_live.as_ref()
    {
        hero_intents::drain_reparent(intent, live, sim, toasts);
    }
    // M14.6 F: drain per-row Hierarchy context-menu actions. Each is
    // a `HierDuplicate/AddChild/ResetTransform/Delete` bus variant
    // — bridge resolves row → Entity, then we apply the corresponding
    // ECS mutation. Order is intentional: Delete last, so a
    // (degenerate) frame that queues "duplicate then delete" leaves
    // the duplicate in place and removes the original. The next
    // snapshot rebuild picks up the result automatically.
    if let Some(row) = duplicate_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        // ⭐ **O que duplicar QUER DIZER mora no irmão** (`hierarchy_duplicate`), pelo tecto de
        // 600 LOC — o corte é por assunto: aqui o dreno das intenções, lá a lei da cópia.
        title_dirty |= super::hierarchy_duplicate::drain(
            ph2d_ecs::Entity::from_bits(entity_bits),
            entity_bits,
            hero,
            sim,
            camera,
            window_size,
            toasts,
            vec_scene,
            vec_entities,
            vec_history,
            vec_pen,
            duplicate_made,
            registry,
        );
    }
    if let Some(row) = add_child_row
        && let Some(live) = hero_live.as_ref()
        && let Some(parent_bits) = live.bridge.entity_for(row)
    {
        let parent = ph2d_ecs::Entity::from_bits(parent_bits);
        let child_name = crate::name_unique::unique_name(sim, "Child");
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new(child_name),
            ph2d_ecs::ChildOf(parent),
        ));
        toasts.push(Toast::success("Added child entity"));
        title_dirty = true;
    }
    // ⭐ **O objeto VAZIO na raiz** (ADR-0166 / F3) — o primeiro passo do smoke desta fase.
    //
    // ⚠️ **`Transform` + `Name`, e mais NADA.** É esta a base de que a F3 fala: o Inspector passa a
    // mostrar o que o objeto TEM, então um objeto acabado de nascer mostra duas seções, não doze. A
    // tentação de lhe dar um `Sprite` "para se ver alguma coisa" é exatamente o que a fase apaga.
    //
    // ⚠️ **`assign_missing_root_order` a seguir, não depois:** uma raiz sem `RootOrder` colate em
    // `u32::MAX` e o desempate cai no `Entity::to_bits()`, que muda a cada respawn do undo — foi
    // esse o defeito que fez a captura deixar de ser ponto fixo (BUGS #15). O `StableId` vem pela
    // mesma porta, e por isso os dois assigners andam em par (precedente: `inspector_joint_create`).
    //
    // ⚠️ **E o objeto novo fica SELECIONADO**, senão o `+` do Inspector não teria sobre quem abrir:
    // criar um objeto e não o mostrar obriga o artista a caçá-lo na lista para continuar o gesto.
    // ⭐ **DEVOLVER à receita** (ADR-0164 / F4.4) — o dreno mora em `instance_sync`, com o verbo:
    // ele é sobre INSTÂNCIAS, e não sobre a mecânica da Hierarquia. (E este ficheiro estava no
    // teto de 600 LOC — *o corte é por assunto*.)
    if let Some(row) = revert_to_master_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
        && crate::instance_revert::drain_revert_to_master(sim, echo, entity_bits, toasts)
    {
        title_dirty = true;
    }
    // ⭐ **Os outros verbos de instância** (ADR-0164 / F4.5) — o dreno mora com eles, pela razão
    // do *Revert*: é sobre INSTÂNCIAS, e não sobre a mecânica das linhas.
    // ⭐ Para onde a selecção vai depois de um verbo de instância.
    //
    // ⚠️ **DOIS slots, e não um.** Os dois drenos (o endereçado por LINHA e o endereçado por
    // `StableId`) escreviam no mesmo — e a escrita da pose da queda lia esse único enquanto o
    // guarda perguntava pelo slot do `StableId`. Se os dois estivessem armados no mesmo quadro e o
    // segundo saísse cedo, o ponto de queda aterrava na entidade que o PRIMEIRO criou.
    // *Um comentário a afirmar que dois não coexistem não é uma cerca.*
    let mut verb_select: Option<u64> = None;
    let mut drop_select: Option<u64> = None;
    if let Some((row, verb)) = instance_verb_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
        && crate::instance_verbs::drain(
            verb,
            sim,
            registry,
            echo,
            entity_bits,
            toasts,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene,
                vec_entities,
            },
            // O passo da cascata: o MESMO de tela que o *Duplicate* usa, convertido pela câmera.
            {
                let (dx, dy) = crate::input_dispatch::screen_offset_world(
                    camera,
                    window_size,
                    crate::input_dispatch::PASTE_OFFSET_PX,
                );
                [dx as f32, dy as f32]
            },
            &mut verb_select,
        )
    {
        title_dirty = true;
    }
    // ⭐ O gémeo por `StableId`. ⚠️ **Ele corre DEPOIS do irmão e os dois não podem estar armados
    // ao mesmo tempo** — um quadro tem um gesto.
    // ⛔ **Uma queda que não achou a receita DIZ que não achou.** O `if let` abaixo curto-circuita
    // quando o `StableId` já não existe (a receita foi apagada entre o `Down` e o `Up`), e sem esta
    // linha o gesto acabava em silêncio total — o artista conclui que colocou.
    if let Some((stable_id, _, _)) = instance_verb_stable_id
        && crate::instance_verbs::entity_for_stable_id(sim, stable_id).is_none()
    {
        toasts.push(Toast::warning("That prefab is no longer in the project"));
    }
    if let Some((stable_id, verb, _at)) = instance_verb_stable_id
        && let Some(entity_bits) = crate::instance_verbs::entity_for_stable_id(sim, stable_id)
        && crate::instance_verbs::drain(
            verb,
            sim,
            registry,
            echo,
            entity_bits,
            toasts,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene,
                vec_entities,
            },
            {
                let (dx, dy) = crate::input_dispatch::screen_offset_world(
                    camera,
                    window_size,
                    crate::input_dispatch::PASTE_OFFSET_PX,
                );
                [dx as f32, dy as f32]
            },
            &mut drop_select,
        )
    {
        title_dirty = true;
    }
    // ⭐⭐ **A QUEDA põe a cópia ONDE a mão largou** (etapa B). ⚠️ Depois do verbo, e não dentro
    // dele: o `instantiate_master` copia o `Transform` da receita verbatim de propósito — uma prova
    // de mutação já matou uma versão que o reescrevia lá dentro.
    //
    // ⛔ A pose é LOCAL: sob uma receita com pai escalado, o ponto chega escalado. É a mesma cerca
    // que o `duplicate_subtree` já declara, e curá-la pede a inversa do mundo do pai — wave própria.
    if let Some((_, _, Some(world))) = instance_verb_stable_id
        && let Some(bits) = drop_select
        && let Some(mut t) = sim
            .world_mut()
            .get_mut::<Transform>(ph2d_ecs::Entity::from_bits(bits))
    {
        t.translation.x = world[0];
        t.translation.y = world[1];
    }
    // ⭐⭐ **O menu de um cartão da biblioteca** (etapa C) — o corpo mudou-se para o irmão
    // `hierarchy_asset_verbs` quando este ficheiro bateu no tecto de 600 LOC do shell.
    let card_select = super::hierarchy_asset_verbs::drain_card_verb(
        asset_card_verb,
        sim,
        registry,
        echo,
        hero,
        toasts,
        vec_scene,
        vec_entities,
        camera,
        window_size,
        atlas_assets,
        &mut title_dirty,
    );
    // ⭐⭐ **A selecção segue a CÓPIA.** Sem isto, o gesto seguinte do artista acerta na receita
    // invisível — que é o mecanismo dos dois reports de 30/08 (apagar e esconder).
    if let Some(bits) = verb_select.or(drop_select).or(card_select) {
        hero.gizmo.replace_selection(Some(bits));
    }
    if add_root {
        let bits = super::hierarchy_add_root::spawn_empty_root(sim);
        hero.gizmo.replace_selection(Some(bits));
        toasts.push(Toast::success("Added empty object"));
        title_dirty = true;
    }
    if let Some(row) = reset_transform_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
            *t = Transform::IDENTITY;
            toasts.push(Toast::info("Transform reset"));
            title_dirty = true;
        }
    }
    if let Some(row) = delete_row
        && let Some(live) = hero_live.as_ref()
        && let Some(clicked_entity_bits) = live.bridge.entity_for(row)
    {
        // Onda 2 fix: if the clicked row is part of the multi-
        // selection, delete EVERY selected sprite (Photoshop / Figma
        // convention — multi-select right-click → Delete affects the
        // whole group). Otherwise just the clicked row. bevy_ecs 0.19
        // `ChildOf` cascade despawns descendants.
        let to_delete: Vec<u64> = if hero.gizmo.is_selected(clicked_entity_bits) {
            hero.gizmo.iter_selected().collect()
        } else {
            vec![clicked_entity_bits]
        };
        for bits in &to_delete {
            let entity = ph2d_ecs::Entity::from_bits(*bits);
            sim.world_mut().despawn(entity);
        }
        // Remove every deleted bits from the selection set. Without
        // this, `selected_len()` stays > 1 even after a multi-delete,
        // which keeps the global gizmo painted around vanished sprites
        // (user-reported: "se deletar algumas e sobrar 1, o gizmo
        // global fica aparecendo mesmo com uma sprite").
        for bits in &to_delete {
            if hero.gizmo.selection == Some(*bits) {
                hero.gizmo.selection = None;
            }
            hero.gizmo.extra_selection.retain(|b| b != bits);
        }
        // If primary was deleted but extras remain, promote the
        // oldest extra so the selection isn't headless.
        if hero.gizmo.selection.is_none() && !hero.gizmo.extra_selection.is_empty() {
            hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
        }
        let n = to_delete.len();
        toasts.push(Toast::warning(if n == 1 {
            "Deleted entity".to_string()
        } else {
            format!("Deleted {n} entities")
        }));
        title_dirty = true;
    }
    // M14.6 D: drain pending hierarchy-row click → sync
    // `gizmo_selection` to whichever entity the user just picked in
    // the hierarchy panel. Legacy single-select path (kept for
    // double-click + any consumer still emitting the variant). Fase 0c
    // shifted the Hierarchy panel to `HierSelectRow` / `HierRangeSelect`
    // which carry modifier semantics — see the dispatch below.
    if let Some(row) = hierarchy_row_click
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        hero.gizmo.replace_selection(Some(entity_bits));
    }
    // Fase 0e: drain pending multi-select-aware hierarchy intent.
    // Bridge resolves row → entity_bits; modifier picks the
    // `GizmoStateGroup` mutation (Replace / Add / Toggle). Range walks
    // the canonical hierarchy order between primary's row and target,
    // adding every entity in between.
    if let Some(intent) = hierarchy_select_intent
        && let Some(live) = hero_live.as_ref()
    {
        match intent {
            HierarchySelectIntent::Row { row, modifier } => {
                if let Some(entity_bits) = live.bridge.entity_for(row) {
                    match modifier {
                        SelectModifier::Replace => {
                            // Smart-click parity with canvas pick (Fase
                            // 0 hotfix): bare click on a row already
                            // part of a multi-selection preserves the
                            // group instead of collapsing to single.
                            let preserves_multi = hero.gizmo.selected_len() > 1
                                && hero.gizmo.is_selected(entity_bits);
                            if !preserves_multi {
                                hero.gizmo.replace_selection(Some(entity_bits));
                            }
                        }
                        SelectModifier::Add => {
                            hero.gizmo.add_to_selection(entity_bits);
                        }
                        SelectModifier::Toggle => {
                            hero.gizmo.toggle_in_selection(entity_bits);
                        }
                    }
                }
            }
            HierarchySelectIntent::Range { row: target_row } => {
                // Onda 2 hotfix v2 — preserves the anchor (Enio: "o
                // shift em múltiplas sprites desselecionou a primeira").
                // Decision based on the TARGET row's current state:
                //   - target NOT selected → ADD every row in
                //     [anchor..target] (anchor stays as primary; new
                //     rows enter via add_to_selection, which is a
                //     no-op for ones already there).
                //   - target selected → REMOVE every row in
                //     [anchor..target] EXCEPT the anchor itself
                //     (anchor never demoted by this gesture; primary
                //     remains stable for the next range click).
                // Without an anchor (selection empty), degenerates to
                // a single add of the target — same as Cmd-click /
                // bare-click on an empty selection.
                let target_bits = live.bridge.entity_for(target_row);
                let anchor_row = hero
                    .gizmo
                    .selection
                    .and_then(|bits| live.bridge.node_for(bits));
                if let (Some(target_bits), Some(anchor_row)) = (target_bits, anchor_row) {
                    let order = hero.store.hierarchy_order();
                    let i_anchor = order.iter().position(|n| *n == anchor_row);
                    let i_target = order.iter().position(|n| *n == target_row);
                    if let (Some(a), Some(t)) = (i_anchor, i_target) {
                        let (lo, hi) = if a <= t { (a, t) } else { (t, a) };
                        let row_range: Vec<_> = order[lo..=hi].to_vec();
                        let target_was_selected = hero.gizmo.is_selected(target_bits);
                        for n in row_range {
                            if n == anchor_row {
                                continue;
                            }
                            if let Some(bits) = live.bridge.entity_for(n) {
                                if target_was_selected {
                                    // Remove from extras only — anchor
                                    // (= primary) skipped above so we
                                    // never demote it.
                                    hero.gizmo.extra_selection.retain(|b| *b != bits);
                                } else {
                                    hero.gizmo.add_to_selection(bits);
                                }
                            }
                        }
                    } else {
                        hero.gizmo.add_to_selection(target_bits);
                    }
                } else if let Some(target_bits) = target_bits {
                    hero.gizmo.add_to_selection(target_bits);
                }
            }
        }
    }
    // M14.7 polish: one-shot seed of the rename TextInput when rename
    // mode opens. `HierRenameSeed` is pushed by hero on the open path
    // (right-click Rename / long-press) and drained here exactly once
    // — so subsequent Backspace edits that empty the buffer don't get
    // clobbered back to the original name on the next frame.
    // ⭐⭐ **O RENOMEAR mudou-se para o irmão** ([`super::hierarchy_rename`]) quando este ficheiro
    // bateu no teto de 600 LOC do shell (HR-18) — pago por CORTE, nunca por excepção.
    //
    // ⚠️ **É um corte por RESPONSABILIDADE, não pelo fim do ficheiro:** semear o campo, gravar o
    // nome, limpar o buffer e honrar as chaves que o nome declara são **uma** coisa — *renomear uma
    // linha* —, e ela não tem nada a ver com os verbos de instância que este ficheiro dreno.
    title_dirty |= super::hierarchy_rename::drain(
        rename_seed_row,
        rename_commit,
        hero,
        hero_live,
        sim,
        toasts,
    );

    title_dirty
}

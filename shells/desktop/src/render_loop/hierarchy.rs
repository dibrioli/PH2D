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

/// ⭐ **Quem sabe duplicar esta entidade.**
///
/// ⚠️ A decisão mora aqui, com nome, porque **a escolha errada é silenciosa**: o braço genérico
/// copia `Transform` + `Sprite` + `Name`, e para uma entidade que guarda a geometria noutro sítio
/// isso produz um **sósia que não desenha nada** — uma linha na Hierarchy sobre coisa nenhuma.
///
/// Já aconteceu duas vezes, em dois módulos:
///
/// | Entidade | O que o braço genérico produzia | Quem duplica de verdade |
/// |---|---|---|
/// | um **path vetorial** (`VecPathRef`) | um sósia sem geometria — ou, pior, dois donos do mesmo path | o documento vetorial, pela porta do painel |
/// | um **nó de modelagem 3D** (`FieldNode`) | uma linha sem `FieldNode` nem `FieldPose`, invisível ao traçado | `field3d_scene::duplicate_node`, a porta do painel |
///
/// *Uma entidade cuja geometria não está nela não se duplica clonando-a.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DuplicateKind {
    /// Um nó de modelagem 3D (ADR-0161).
    Field,
    /// Um path do editor vetorial (ADR-0110).
    VecPath,
    /// Tudo o resto: sprites e entidades comuns, que **são** o que guardam.
    Entity,
}

/// Ver [`DuplicateKind`].
pub(super) fn duplicate_kind(
    world: &bevy_ecs::world::World,
    src: ph2d_ecs::Entity,
) -> DuplicateKind {
    if world.get::<ph2d_field_ecs::FieldNode>(src).is_some() {
        DuplicateKind::Field
    } else if world.get::<ph2d_ecs::VecPathRef>(src).is_some() {
        DuplicateKind::VecPath
    } else {
        DuplicateKind::Entity
    }
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
        hero_intents::drain_reparent(intent, live, sim);
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
        let src = ph2d_ecs::Entity::from_bits(entity_bits);
        // ⚠️ **Uma forma VETORIAL não se duplica clonando a entidade.** O dono da geometria é o
        // documento, e `vec_entities::sync` mantém UMA entidade por path, nas duas direções:
        //
        // - clonar a entidade sem o `VecPathRef` dá um sósia que **não desenha nada** — uma linha
        //   na Hierarchy sobre geometria nenhuma, que era o que esta row fazia;
        // - copiar o `VecPathRef` seria pior: duas entidades a apontar para o MESMO path, e o
        //   `sync` tem de escolher uma.
        //
        // Então o clone é um **PATH**, feito pela porta que o botão Duplicate do painel usa, e o
        // `sync` cunha a entidade dele (com nome único e `RootOrder`) no mesmo frame.
        // ⚠️ **Quem duplica esta entidade não é óbvio, e a escolha errada é SILENCIOSA** — ver
        // [`duplicate_kind`], que é onde a decisão mora (e onde um gate lhe chega).
        if duplicate_kind(sim.world(), src) == DuplicateKind::Field {
            if let Some(copy) = crate::field3d_scene::duplicate_node(sim.world_mut(), src) {
                // ⭐ A cópia fica selecionada, como no botão do painel: é o que põe o gizmo em cima
                // dela sem ninguém a ter de procurar.
                hero.gizmo.replace_selection(Some(copy));
                toasts.push(Toast::success("Duplicated shape"));
                title_dirty = true;
            }
        } else if let Some(vp) = sim.world().get::<ph2d_ecs::VecPathRef>(src).copied() {
            let (dx, dy) = crate::input_dispatch::screen_offset_world(
                camera,
                window_size,
                crate::input_dispatch::PASTE_OFFSET_PX,
            );
            if crate::input_dispatch::duplicate_vec_paths(
                vec_scene,
                vec_history,
                vec_pen,
                &[vp.0],
                dx,
                dy,
            ) {
                toasts.push(Toast::success("Duplicated shape"));
                title_dirty = true;
            }
        } else {
            // ⭐ **A cópia é PROFUNDA** (ADR-0164 / F4.2) — a subárvore inteira, todo componente
            // registado, identidade nova, e as referências internas remapeadas.
            //
            // ⚠️ **O que estava aqui antes copiava QUATRO componentes** (`Transform`, `Sprite`,
            // `Name`, `ChildOf`) **e nenhum filho**: duplicar um ragdoll dava uma linha vazia na
            // Hierarquia, e duplicar um corpo com junta dava um corpo solto. O ADR-0164 nomeia
            // este defeito, e ele existia por falta de porta, não por decisão.
            //
            // ⚠️ **O nome único continua a ser lei** — a Hierarquia já teve seleção por rótulo, e
            // qualquer código que volte a chavear pelo nome amigável merece a mesma defesa.
            let sprite = sim.world().get::<ph2d_render::Sprite>(src).is_some();
            let mut docs = crate::instance_docs::OwnedDocs {
                vec_scene,
                vec_entities,
            };
            if let Some(copy) = crate::instantiate::duplicate_subtree(sim, registry, src, &mut docs)
            {
                // Report the pair so the caller can fork the copy's texture off the source
                // (independent object) + flush any live paint on the source first. Only matters
                // for sprite entities.
                if sprite {
                    *duplicate_made = Some((entity_bits, copy.to_bits()));
                }
                toasts.push(Toast::success("Duplicated entity"));
                title_dirty = true;
            }
        }
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
        )
    {
        title_dirty = true;
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
        // whole group). Otherwise just the clicked row. bevy_ecs 0.18
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
    if let Some(row) = rename_seed_row
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let value = sim
            .world()
            .get::<Name>(entity)
            .map(|n| n.as_str().to_owned())
            .unwrap_or_default();
        if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = hero
            .store
            .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
        {
            let len = value.len();
            *text = value;
            *caret = len;
            *selection_anchor = Some(0); // select all
        }
    }
    // Drain a finalized rename commit (Enter pressed in rename
    // input). Write the new Name component on the entity; toast
    // confirms.
    if let Some((row, new_name)) = rename_commit
        && let Some(live) = hero_live.as_ref()
        && let Some(entity_bits) = live.bridge.entity_for(row)
    {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        // Reject user-typed collisions with other entities. `_excluding`
        // ignores `entity`'s own current name so committing the same
        // name (no-op rename) doesn't auto-suffix into "(1)".
        let final_name = crate::name_unique::unique_name_excluding(sim, &new_name, entity);
        let was_adjusted = final_name != new_name;
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            entry.insert(Name::new(final_name.clone()));
            if was_adjusted {
                toasts.push(Toast::warning(format!(
                    "Name in use — renamed to {final_name}"
                )));
            } else {
                toasts.push(Toast::success(format!("Renamed to {final_name}")));
            }
            title_dirty = true;
        }
        // Clear the rename TextInput buffer for next session.
        if let Some(ph2d_editor::interaction::InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = hero
            .store
            .get_mut(ph2d_editor::screens::hero::ids::HIER_RENAME_INPUT)
        {
            text.clear();
            *caret = 0;
            *selection_anchor = None;
        }
    }

    title_dirty
}

/// ⚠️ Os gates do ROTEAMENTO vivem no irmão, pelo teto de 600 LOC — o corte é por assunto
/// (o dreno das intenções aqui, a prova de que cada tipo vai para a porta certa lá).
#[cfg(test)]
#[path = "hierarchy_duplicate_routing_tests.rs"]
mod duplicate_routing;

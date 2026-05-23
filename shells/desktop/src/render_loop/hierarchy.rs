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
    reparent_intent: Option<HierReparentIntent>,
    duplicate_row: Option<NodeId>,
    add_child_row: Option<NodeId>,
    reset_transform_row: Option<NodeId>,
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
        let sim_w = sim.world_mut();
        let transform = sim_w.get::<Transform>(src).copied();
        let sprite = sim_w.get::<ph2d_render::Sprite>(src).copied();
        let name = sim_w.get::<Name>(src).map(|n| n.as_str().to_owned());
        let parent = sim_w.get::<ph2d_ecs::ChildOf>(src).map(|c| c.parent());
        let copy_name = name
            .map(|n| format!("{n}_copy"))
            .unwrap_or_else(|| "copy".to_string());
        let mut builder = sim_w.spawn_empty();
        if let Some(t) = transform {
            builder.insert(t);
        }
        if let Some(s) = sprite {
            builder.insert(s);
        }
        builder.insert(Name::new(copy_name));
        if let Some(p) = parent {
            builder.insert(ph2d_ecs::ChildOf(p));
        }
        toasts.push(Toast::success("Duplicated entity"));
        title_dirty = true;
    }
    if let Some(row) = add_child_row
        && let Some(live) = hero_live.as_ref()
        && let Some(parent_bits) = live.bridge.entity_for(row)
    {
        let parent = ph2d_ecs::Entity::from_bits(parent_bits);
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Child"),
            ph2d_ecs::ChildOf(parent),
        ));
        toasts.push(Toast::success("Added child entity"));
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
        let sim_w = sim.world_mut();
        if let Ok(mut entry) = sim_w.get_entity_mut(entity) {
            entry.insert(Name::new(new_name.clone()));
            toasts.push(Toast::success(format!("Renamed to {new_name}")));
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

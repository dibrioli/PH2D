//! Hierarchy drag-reparent drain (`EditorAction::HierReparent`).
//!
//! Wave 3.1 stage A — extracted from `hero_intents.rs` as part of
//! the HR-18 closeout split. Behavior-preserving lift.

use ph2d_ecs::SimWorld;

/// Drain a `pending_reparent` hierarchy intent: move a dragged
/// hierarchy row to a new parent (or root), positioning it relative
/// to a target sibling (before / after). Guards against cycles
/// (refuses to make dragged a descendant of itself). Re-inserts
/// `ChildOf` on every sibling in the desired order so bevy_ecs's
/// `Children` list reflects the user-chosen sequence.
///
/// Returns `false` — never pushes a toast (silent reparent matches
/// existing UX). Caller does not set title_dirty.
pub(crate) fn drain_reparent(
    intent: ph2d_editor::screens::hero::HierReparentIntent,
    live: &crate::HeroLive,
    sim: &mut SimWorld,
) -> bool {
    use ph2d_ecs::Transform;
    let Some(dragged_bits) = live.bridge.entity_for(intent.dragged) else {
        return false;
    };
    let dragged = ph2d_ecs::Entity::from_bits(dragged_bits);

    let new_parent_entity = if let Some(parent_node) = intent.new_parent
        && let Some(parent_bits) = live.bridge.entity_for(parent_node)
    {
        Some(ph2d_ecs::Entity::from_bits(parent_bits))
    } else if let Some(before_node) = intent.before
        && let Some(target_bits) = live.bridge.entity_for(before_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else if let Some(after_node) = intent.after
        && let Some(target_bits) = live.bridge.entity_for(after_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else {
        None
    };
    let sim_w = sim.world_mut();
    let would_cycle = new_parent_entity.is_some_and(|np| {
        let mut current = Some(np);
        while let Some(c) = current {
            if c == dragged {
                return true;
            }
            current = sim_w.get::<ph2d_ecs::ChildOf>(c).map(|c| c.parent());
        }
        false
    });
    if would_cycle {
        return false;
    }
    // Step 1: pick the new ChildOf relation.
    if let Ok(mut entry) = sim_w.get_entity_mut(dragged) {
        match new_parent_entity {
            Some(p) => {
                entry.insert(ph2d_ecs::ChildOf(p));
            }
            None => {
                entry.remove::<ph2d_ecs::ChildOf>();
            }
        }
    }
    // M14.7 polish: root drops need an explicit `RootOrder`.
    if new_parent_entity.is_none() {
        let mut roots: Vec<ph2d_ecs::Entity> = {
            let mut q = sim_w.query_filtered::<ph2d_ecs::Entity, (
                ph2d_ecs::With<Transform>,
                ph2d_ecs::Without<ph2d_ecs::ChildOf>,
            )>();
            let mut acc: Vec<(ph2d_ecs::Entity, u32)> = Vec::new();
            for entity in q.iter(sim_w) {
                if entity == dragged {
                    continue;
                }
                let order = sim_w
                    .get::<ph2d_ecs::RootOrder>(entity)
                    .map(|r| r.0)
                    .unwrap_or(u32::MAX);
                acc.push((entity, order));
            }
            acc.sort_unstable_by(|a, b| {
                a.1.cmp(&b.1)
                    .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
            });
            acc.into_iter().map(|(e, _)| e).collect()
        };
        let before_target = intent
            .before
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let after_target = intent
            .after
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let insert_at = if let Some(b) = before_target {
            roots.iter().position(|e| *e == b).unwrap_or(roots.len())
        } else if let Some(a) = after_target {
            roots
                .iter()
                .position(|e| *e == a)
                .map(|i| i + 1)
                .unwrap_or(roots.len())
        } else {
            roots.len()
        };
        roots.insert(insert_at.min(roots.len()), dragged);
        for (idx, e) in roots.iter().enumerate() {
            if let Ok(mut entry) = sim_w.get_entity_mut(*e) {
                entry.insert(ph2d_ecs::RootOrder(idx as u32));
            }
        }
    }
    // Step 2: enforce sibling order.
    let target_kind: Option<(ph2d_ecs::Entity, bool)> = if let Some(before_node) = intent.before
        && let Some(b) = live.bridge.entity_for(before_node)
    {
        Some((ph2d_ecs::Entity::from_bits(b), true))
    } else if let Some(after_node) = intent.after
        && let Some(a) = live.bridge.entity_for(after_node)
    {
        Some((ph2d_ecs::Entity::from_bits(a), false))
    } else {
        None
    };
    if let (Some(parent), Some((target, place_before))) = (new_parent_entity, target_kind) {
        let current: Vec<ph2d_ecs::Entity> = sim_w
            .get::<bevy_ecs::hierarchy::Children>(parent)
            .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
            .unwrap_or_default();
        let mut desired: Vec<ph2d_ecs::Entity> = Vec::with_capacity(current.len() + 1);
        let mut inserted = false;
        for &c in &current {
            if !inserted && c == target && place_before {
                desired.push(dragged);
                inserted = true;
            }
            desired.push(c);
            if !inserted && c == target && !place_before {
                desired.push(dragged);
                inserted = true;
            }
        }
        if !inserted {
            desired.push(dragged);
        }
        for &child in &desired {
            if let Ok(mut entry) = sim_w.get_entity_mut(child) {
                entry.remove::<ph2d_ecs::ChildOf>();
                entry.insert(ph2d_ecs::ChildOf(parent));
            }
        }
    }
    false
}

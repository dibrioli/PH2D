//! Editor-consumable snapshots of the SimWorld (ADR-0025 M14.3c).
//!
//! These types are the **substrate** for a future editor integration.
//! M14.3 ships them with tests so the `ph2d-editor` crate can plug in
//! later (M15+) without re-designing the data flow. The editor itself
//! is intentionally untouched in this milestone (hero screen is in
//! dev/test per user constraint).
//!
//! # Two snapshot types
//!
//! - [`HierarchySnapshot`] — flat list of entities in DFS order, with
//!   `parent` ids + `depth`. The Hierarchy panel will iterate this.
//! - [`ComponentSnapshot`] — per-entity component blobs for the
//!   Inspector panel. Built on demand (per selected entity), not per
//!   frame for every entity.
//!
//! Both snapshots are populated from `SimWorld` data via a
//! `bevy_ecs::QueryState` pre-built at boot — the same pattern
//! `propagate_transforms` uses, so the editor's per-frame cost is a
//! bounded walk.

use crate::Name;
use crate::Transform;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::query::{QueryState, With, Without};
use bevy_ecs::world::World;

use super::registry::{ComponentRegistry, RegistryError};

// ───────────────────────── HierarchySnapshot ──────────────────────

/// One row in [`HierarchySnapshot`]. Hex-id form so the editor never
/// holds a live `bevy_ecs::Entity` (HR-8).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HierarchyEntry {
    /// `bevy_ecs::Entity::to_bits()` — opaque u64.
    pub entity: u64,
    /// `Name` component value if present.
    pub name: Option<String>,
    /// `ChildOf::0` parent, also as opaque u64. `None` for roots.
    pub parent: Option<u64>,
    /// Depth from root (0 = root). Caps at `u8::MAX` — deeper
    /// hierarchies saturate.
    pub depth: u8,
    /// M14.6A: `true` when the entity is rendered; `false` when the
    /// user has toggled the hierarchy eye-icon. Default `true` for
    /// entities without a [`Visibility`](crate::Visibility) component.
    pub visible: bool,
    /// 2026-05-26: `true` iff entity carries [`crate::Locked`]. Gizmo
    /// edits on this entity are rejected; descendants remain editable.
    pub locked: bool,
    /// 2026-05-26: `true` iff entity carries [`crate::GroupedChildren`].
    /// Descendants are locked while THIS entity remains editable.
    pub group_locked: bool,
}

/// DFS-ordered flat view of the sim hierarchy. Each `build_*` pass
/// clears + repopulates; capacity is preserved (HR-3 friendly).
#[derive(Default)]
pub struct HierarchySnapshot {
    pub entries: Vec<HierarchyEntry>,
}

impl HierarchySnapshot {
    pub const DEFAULT_CAPACITY: usize = 4096;

    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(Self::DEFAULT_CAPACITY),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Find the entry for a given opaque entity id, if present.
    /// `O(n)` — fine for editor-side per-frame search.
    pub fn find(&self, entity: u64) -> Option<&HierarchyEntry> {
        self.entries.iter().find(|e| e.entity == entity)
    }
}

/// Tuple of optional component refs the hierarchy walk fetches per
/// entity. Aliased pra clippy `type_complexity` (era 5-field tuple
/// inline).
type HierarchyChainFetch = (
    Option<&'static Name>,
    Option<&'static Children>,
    Option<&'static crate::Visibility>,
    Option<&'static crate::Locked>,
    Option<&'static crate::GroupedChildren>,
);

/// Pre-built query state for [`build_hierarchy_snapshot`]. One per
/// app; constructed once at boot from `&mut SimWorld`.
pub struct HierarchyWalkState {
    roots: QueryState<Entity, (With<Transform>, Without<ChildOf>)>,
    chain: QueryState<HierarchyChainFetch>,
}

impl HierarchyWalkState {
    pub fn new(world: &mut World) -> Self {
        Self {
            roots: world.query_filtered::<Entity, (With<Transform>, Without<ChildOf>)>(),
            chain: world.query::<(
                Option<&Name>,
                Option<&Children>,
                Option<&crate::Visibility>,
                Option<&crate::Locked>,
                Option<&crate::GroupedChildren>,
            )>(),
        }
    }
}

/// Build a deterministic [`HierarchySnapshot`] from `sim_w`.
///
/// `scratch` is a caller-provided `Vec` that the function uses for
/// its DFS stack. Holding it externally lets the editor reuse it
/// frame-to-frame without re-allocating (HR-3 substrate).
///
/// Determinism (HR-5): roots are sorted by `(RootOrder.0,
/// entity.to_bits())`. Entities without `RootOrder` collate AFTER
/// every explicitly-ordered one (via `u32::MAX`) so freshly-spawned
/// roots show up at the bottom of the panel by default — matching
/// user intuition that "the new one lands last." The editor's
/// `pending_reparent` drain assigns sequential indices when the
/// user drops a root before/after another root. Children are pushed
/// in reverse so DFS visits them in `Children`-insertion order.
/// Names are cloned into the entry — there's no shared-borrow
/// lifetime hazard for editor consumers.
pub fn build_hierarchy_snapshot(
    sim_w: &World,
    state: &mut HierarchyWalkState,
    scratch: &mut Vec<(Entity, u8, Option<Entity>)>,
    out: &mut HierarchySnapshot,
) {
    out.clear();
    scratch.clear();

    // Collect roots with their explicit ordering key (if any). The
    // sort runs ascending, but the DFS pops from the END of `scratch`
    // — so push in REVERSE of the desired display order.
    let mut roots: Vec<(Entity, u32)> = Vec::new();
    for entity in state.roots.iter(sim_w) {
        let order = sim_w
            .get::<crate::RootOrder>(entity)
            .map(|r| r.0)
            .unwrap_or(u32::MAX);
        roots.push((entity, order));
    }
    roots.sort_unstable_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
    });
    // Reverse before pushing: DFS pops LIFO, so the LAST scratch
    // entry becomes the FIRST entry in the snapshot.
    for (entity, _) in roots.into_iter().rev() {
        scratch.push((entity, 0, None));
    }

    while let Some((entity, depth, parent)) = scratch.pop() {
        let Ok((name, children, vis, lk, grp)) = state.chain.get(sim_w, entity) else {
            continue;
        };
        out.entries.push(HierarchyEntry {
            entity: entity.to_bits(),
            name: name.map(|n| n.as_str().to_owned()),
            parent: parent.map(|p| p.to_bits()),
            depth,
            visible: !vis.is_some_and(|v| v.hidden),
            locked: lk.is_some(),
            group_locked: grp.is_some(),
        });
        if let Some(children) = children {
            // Push children in reverse so DFS visits the first child
            // first (LIFO stack semantics).
            let kids: Vec<Entity> = children.iter().copied().collect();
            for c in kids.iter().rev().copied() {
                let next_depth = depth.saturating_add(1);
                scratch.push((c, next_depth, Some(entity)));
            }
        }
    }
}

// ───────────────────────── ComponentSnapshot ──────────────────────

/// One serialized component blob for the Inspector panel. Same
/// shape as `ph2d_asset::ComponentBlob` but lives in PresentWorld
/// (no save-format dependency).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentEntry {
    /// Canonical name (e.g. `"ph2d::ecs::Transform"`). The editor
    /// uses this as the panel header.
    pub canonical_name: &'static str,
    /// Stable type id (same as the cooked-prefab one).
    pub type_id: u64,
    /// Postcard-encoded component value.
    pub data: Vec<u8>,
}

/// Per-entity component snapshot — populated by
/// [`extract_component_snapshot`] on demand (typically when the
/// editor selection changes).
#[derive(Default)]
pub struct ComponentSnapshot {
    /// The entity this snapshot describes, as `Entity::to_bits()`.
    pub entity: u64,
    pub entries: Vec<ComponentEntry>,
}

impl ComponentSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.entity = 0;
        self.entries.clear();
    }
}

/// Populate `out` with every registered component present on
/// `entity`. Iterates the registry in id-sorted order so the
/// inspector panel order is stable across runs (HR-5).
pub fn extract_component_snapshot(
    sim_w: &World,
    entity: Entity,
    registry: &ComponentRegistry,
    out: &mut ComponentSnapshot,
) -> Result<(), RegistryError> {
    out.clear();
    out.entity = entity.to_bits();
    for entry in registry.iter() {
        match (entry.serialize)(sim_w, entity) {
            Ok(Some(bytes)) => {
                out.entries.push(ComponentEntry {
                    canonical_name: entry.canonical_name,
                    type_id: entry.type_id,
                    data: bytes,
                });
            }
            Ok(None) => {} // entity exists but has no component of this type
            Err(RegistryError::EntityMissing(_)) => {
                // Entity may have been despawned mid-frame; not an
                // error from the editor's perspective — clear and
                // return success with empty snapshot.
                out.clear();
                return Ok(());
            }
            Err(other) => return Err(other),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimWorld;
    use crate::scene::register_ecs_components;
    use ph2d_core::Vec2;

    #[test]
    fn hierarchy_snapshot_3_levels() {
        let mut sim = SimWorld::new();
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Root")))
            .id();
        let mid = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Mid"), ChildOf(root)))
            .id();
        let _leaf = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Leaf"), ChildOf(mid)))
            .id();
        let mut state = HierarchyWalkState::new(sim.world_mut());
        let mut scratch = Vec::new();
        let mut snapshot = HierarchySnapshot::new();
        build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snapshot);

        assert_eq!(snapshot.len(), 3);
        let by_name: std::collections::BTreeMap<String, &HierarchyEntry> = snapshot
            .entries
            .iter()
            .map(|e| (e.name.clone().unwrap_or_default(), e))
            .collect();
        let root_entry = by_name.get("Root").unwrap();
        let mid_entry = by_name.get("Mid").unwrap();
        let leaf_entry = by_name.get("Leaf").unwrap();
        assert_eq!(root_entry.depth, 0);
        assert_eq!(root_entry.parent, None);
        assert_eq!(mid_entry.depth, 1);
        assert_eq!(mid_entry.parent, Some(root.to_bits()));
        assert_eq!(leaf_entry.depth, 2);
        assert_eq!(leaf_entry.parent, Some(mid.to_bits()));
    }

    #[test]
    fn hierarchy_snapshot_reuses_capacity() {
        let mut sim = SimWorld::new();
        for _ in 0..10 {
            sim.world_mut().spawn(Transform::IDENTITY);
        }
        let mut state = HierarchyWalkState::new(sim.world_mut());
        let mut scratch = Vec::with_capacity(64);
        let mut snapshot = HierarchySnapshot::new();
        let cap_before = snapshot.entries.capacity();
        build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snapshot);
        let cap_after = snapshot.entries.capacity();
        // After build, capacity must be ≥ before (never shrink) and
        // ≥ entries.len().
        assert!(cap_after >= cap_before);
        assert!(cap_after >= snapshot.len());
        // Second build clears, capacity preserved.
        build_hierarchy_snapshot(sim.world(), &mut state, &mut scratch, &mut snapshot);
        assert_eq!(snapshot.entries.capacity(), cap_after);
    }

    #[test]
    fn component_snapshot_round_trip() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);

        let entity = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(7.0, 8.0)),
                Name::new("Subject"),
            ))
            .id();

        let mut snap = ComponentSnapshot::new();
        extract_component_snapshot(sim.world(), entity, &reg, &mut snap).unwrap();
        assert_eq!(snap.entity, entity.to_bits());
        assert_eq!(snap.entries.len(), 2);

        // Decode the Transform blob back and verify the value.
        let t_entry = snap
            .entries
            .iter()
            .find(|e| e.canonical_name == "ph2d::ecs::Transform")
            .unwrap();
        let t: Transform = postcard::from_bytes(&t_entry.data).unwrap();
        assert_eq!(t.translation, Vec2::new(7.0, 8.0));

        let n_entry = snap
            .entries
            .iter()
            .find(|e| e.canonical_name == "ph2d::ecs::Name")
            .unwrap();
        let n: Name = postcard::from_bytes(&n_entry.data).unwrap();
        assert_eq!(n.as_str(), "Subject");
    }

    #[test]
    fn component_snapshot_iterates_in_stable_order() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let entity = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("X")))
            .id();
        let mut a = ComponentSnapshot::new();
        let mut b = ComponentSnapshot::new();
        extract_component_snapshot(sim.world(), entity, &reg, &mut a).unwrap();
        extract_component_snapshot(sim.world(), entity, &reg, &mut b).unwrap();
        let names_a: Vec<&str> = a.entries.iter().map(|e| e.canonical_name).collect();
        let names_b: Vec<&str> = b.entries.iter().map(|e| e.canonical_name).collect();
        assert_eq!(names_a, names_b);
    }

    #[test]
    fn component_snapshot_missing_entity_returns_empty() {
        let sim = SimWorld::new();
        let reg = ComponentRegistry::new();
        let fake = Entity::from_raw_u32(999).unwrap();
        let mut snap = ComponentSnapshot::new();
        // Missing entity → registry returns EntityMissing for every
        // registered type, but the iter is empty so no error.
        extract_component_snapshot(sim.world(), fake, &reg, &mut snap).unwrap();
        assert!(snap.entries.is_empty());
    }
}

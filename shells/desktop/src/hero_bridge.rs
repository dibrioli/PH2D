// Several bridge methods (`entity_for`, `node_for`, `len`,
// `is_empty`, `clear`) are public API used by tests + future
// M14.4b/c wiring; the M14.4a-only `main.rs` only calls
// `sync_from_snapshot`. Allow dead_code crate-wide on this module
// until M14.4b lights up the rest.
#![allow(dead_code)]
//! `EntityNodeMap` — bridges `ph2d_ecs::HierarchySnapshot` to the
//! editor's hierarchy panel data shape (ADR-0025 M14.4a).
//!
//! # Why a separate bridge module
//!
//! - ADR-0021 puts the editor on the PresentWorld consumer side and
//!   forbids it from holding live `bevy_ecs::Entity` values
//!   (HR-8 keeps the editor a pure-data consumer). The bridge owns
//!   the `Entity ↔ NodeId` translation so neither the editor nor
//!   the sim crates have to.
//! - ADR-0025 §"Editor → SimWorld bridge" places `HierarchySnapshot`
//!   in PresentWorld; the bridge is the natural single-responsibility
//!   home for translating that snapshot into the editor's
//!   `HierarchyEntity` rows.
//!
//! # NodeId allocation strategy
//!
//! `NodeId` is a `u64`. The fixed-name `HIER_*` ids in
//! `crates/ph2d-editor/src/screens/hero/ids.rs` live in the range
//! `400..=411`. We start the bridge's allocation at
//! [`Self::BASE_NODE_ID`] (100_000) so collisions are impossible.
//!
//! Allocation is **monotonic** within a session: each new entity
//! claims the next free `NodeId` and never recycles. Stale entries
//! (entities that despawned) are left in the maps; the
//! `WidgetStore::states` map grows by the unique entity count over
//! the session. For typical editor work (< 10k creates) this is
//! negligible. M15+ may add `WidgetStore::retain` if it ever
//! matters.
//!
//! # HR-3 friendliness
//!
//! Per-frame `sync_from_snapshot` reuses the internal `BTreeMap`s
//! via `entry().or_insert_with` — no realloc unless a brand-new
//! entity is seen. The output `Vec<NodeId>` and
//! `BTreeMap<NodeId, HierarchyEntity>` are allocated fresh each
//! call (the editor's `sync_from_hierarchy` takes ownership), but
//! capacity stays bounded by entity count.

use ph2d_ecs::scene::HierarchySnapshot;
use ph2d_editor::NodeId;
use ph2d_editor::icons::IconId;
use ph2d_editor::screens::hero::fixture::HierarchyEntity;
use std::collections::BTreeMap;

/// Bi-directional `Entity::to_bits() ↔ NodeId` map.
///
/// Survives across frames; cleared only on explicit reset (e.g.
/// scene unload).
pub struct EntityNodeMap {
    entity_to_node: BTreeMap<u64, NodeId>,
    node_to_entity: BTreeMap<NodeId, u64>,
    next_id: u64,
}

impl EntityNodeMap {
    /// First `NodeId` value handed out by [`Self::allocate`]. Chosen
    /// to be far above the reserved `HIER_*` range (400-411) and
    /// any other static editor ids so collisions are impossible.
    pub const BASE_NODE_ID: u64 = 100_000;

    pub fn new() -> Self {
        Self {
            entity_to_node: BTreeMap::new(),
            node_to_entity: BTreeMap::new(),
            next_id: Self::BASE_NODE_ID,
        }
    }

    /// Return the bridge's `NodeId` for `entity_bits`, allocating a
    /// fresh one on first sight. Subsequent calls return the same
    /// id — stability is what makes hierarchy drag-state survive
    /// across frames.
    pub fn allocate(&mut self, entity_bits: u64) -> NodeId {
        if let Some(&id) = self.entity_to_node.get(&entity_bits) {
            return id;
        }
        let id = NodeId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("EntityNodeMap: NodeId space exhausted (2^64 entities in one session)");
        self.entity_to_node.insert(entity_bits, id);
        self.node_to_entity.insert(id, entity_bits);
        id
    }

    /// Reverse lookup: given a `NodeId` from a hierarchy click, find
    /// the originating entity bits. Returns `None` for any id the
    /// bridge didn't allocate (e.g. fixture `HIER_*` constants).
    pub fn entity_for(&self, id: NodeId) -> Option<u64> {
        self.node_to_entity.get(&id).copied()
    }

    /// Forward lookup. Returns `None` if the entity hasn't been
    /// seen this session.
    pub fn node_for(&self, entity_bits: u64) -> Option<NodeId> {
        self.entity_to_node.get(&entity_bits).copied()
    }

    pub fn len(&self) -> usize {
        self.entity_to_node.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entity_to_node.is_empty()
    }

    /// Drop every entity ↔ node mapping. Called on full scene unload
    /// (M14.5+). Resets the allocator to `BASE_NODE_ID`.
    pub fn clear(&mut self) {
        self.entity_to_node.clear();
        self.node_to_entity.clear();
        self.next_id = Self::BASE_NODE_ID;
    }

    /// Translate a [`HierarchySnapshot`] into the pair the editor's
    /// [`ph2d_editor::HeroScreen::sync_from_hierarchy`] expects:
    ///
    /// - `Vec<NodeId>` in the snapshot's DFS visit order (matches
    ///   the hierarchy panel's natural top-to-bottom display).
    /// - `BTreeMap<NodeId, HierarchyEntity>` so the painter can
    ///   look up each row's display data.
    ///
    /// Re-uses existing `NodeId`s for entities already seen this
    /// session; allocates new ids only for unseen ones.
    pub fn sync_from_snapshot(
        &mut self,
        snapshot: &HierarchySnapshot,
    ) -> (Vec<NodeId>, BTreeMap<NodeId, HierarchyEntity>) {
        let mut ordered = Vec::with_capacity(snapshot.entries.len());
        let mut entries = BTreeMap::new();
        for entry in &snapshot.entries {
            let node_id = self.allocate(entry.entity);
            ordered.push(node_id);
            entries.insert(
                node_id,
                HierarchyEntity {
                    name: entry
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Entity_{:x}", entry.entity)),
                    icon: IconId::Sprite,
                    indent: entry.depth,
                    badge: None,
                    swatch: None,
                    visible: entry.visible,
                    selected: false,
                    muted: false,
                },
            );
        }
        (ordered, entries)
    }
}

impl Default for EntityNodeMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::scene::HierarchyEntry;

    fn entry(entity: u64, name: &str, parent: Option<u64>, depth: u8) -> HierarchyEntry {
        HierarchyEntry {
            entity,
            name: Some(name.to_owned()),
            parent,
            depth,
            visible: true,
        }
    }

    #[test]
    fn allocate_is_idempotent() {
        let mut m = EntityNodeMap::new();
        let a1 = m.allocate(42);
        let a2 = m.allocate(42);
        assert_eq!(a1, a2);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn allocate_assigns_distinct_ids() {
        let mut m = EntityNodeMap::new();
        let a = m.allocate(1);
        let b = m.allocate(2);
        assert_ne!(a, b);
        assert_eq!(a.0, EntityNodeMap::BASE_NODE_ID);
        assert_eq!(b.0, EntityNodeMap::BASE_NODE_ID + 1);
    }

    #[test]
    fn reverse_lookup_round_trips() {
        let mut m = EntityNodeMap::new();
        let id = m.allocate(0xCAFE);
        assert_eq!(m.entity_for(id), Some(0xCAFE));
        assert_eq!(m.node_for(0xCAFE), Some(id));
        // Never-allocated id returns None.
        assert_eq!(m.entity_for(NodeId(1)), None);
    }

    #[test]
    fn sync_preserves_dfs_order() {
        let mut m = EntityNodeMap::new();
        let snap = HierarchySnapshot {
            entries: vec![
                entry(10, "Root", None, 0),
                entry(20, "Mid", Some(10), 1),
                entry(30, "Leaf", Some(20), 2),
            ],
        };
        let (ordered, entries) = m.sync_from_snapshot(&snap);
        assert_eq!(ordered.len(), 3);
        // Each NodeId in `ordered` resolves back to the right entity.
        assert_eq!(m.entity_for(ordered[0]), Some(10));
        assert_eq!(m.entity_for(ordered[1]), Some(20));
        assert_eq!(m.entity_for(ordered[2]), Some(30));
        // Entries carry the right names + depths.
        assert_eq!(entries.get(&ordered[0]).unwrap().name, "Root");
        assert_eq!(entries.get(&ordered[1]).unwrap().indent, 1);
        assert_eq!(entries.get(&ordered[2]).unwrap().indent, 2);
    }

    #[test]
    fn sync_reuses_ids_across_calls() {
        let mut m = EntityNodeMap::new();
        let snap_a = HierarchySnapshot {
            entries: vec![entry(7, "A", None, 0)],
        };
        let (ordered_a, _) = m.sync_from_snapshot(&snap_a);
        let snap_b = HierarchySnapshot {
            entries: vec![entry(7, "A", None, 0), entry(8, "B", None, 0)],
        };
        let (ordered_b, _) = m.sync_from_snapshot(&snap_b);
        // First entity keeps its NodeId across syncs.
        assert_eq!(ordered_a[0], ordered_b[0]);
        // New entity gets a fresh NodeId.
        assert_ne!(ordered_b[1], ordered_b[0]);
    }

    #[test]
    fn nameless_entity_gets_hex_placeholder() {
        let mut m = EntityNodeMap::new();
        let snap = HierarchySnapshot {
            entries: vec![HierarchyEntry {
                entity: 0xDEAD,
                name: None,
                parent: None,
                depth: 0,
                visible: true,
            }],
        };
        let (ordered, entries) = m.sync_from_snapshot(&snap);
        assert_eq!(entries.get(&ordered[0]).unwrap().name, "Entity_dead");
    }

    #[test]
    fn clear_resets_allocator() {
        let mut m = EntityNodeMap::new();
        m.allocate(1);
        m.allocate(2);
        m.allocate(3);
        m.clear();
        assert_eq!(m.len(), 0);
        let fresh = m.allocate(99);
        assert_eq!(fresh.0, EntityNodeMap::BASE_NODE_ID);
    }

    #[test]
    fn base_id_above_static_hier_range() {
        // The reserved HIER_* range in ids.rs lives at 400-411. The
        // bridge must allocate well above that so a click on a
        // bridge-allocated row never collides with a static id.
        const _: () = assert!(EntityNodeMap::BASE_NODE_ID > 1000);
    }
}

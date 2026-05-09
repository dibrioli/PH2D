//! [`Tree`] — accumulator for AccessKit `TreeUpdate`s.
//!
//! Per-frame protocol:
//! 1. Editor widgets call [`Tree::insert`] / [`Tree::update`] to
//!    register or refresh their nodes (typically once per frame from
//!    the `extract!` phase).
//! 2. Editor calls [`Tree::set_focus`] when focus moves.
//! 3. Shell calls [`Tree::take_update`] and forwards to the
//!    `accesskit_<os>` adapter (e.g. `accesskit_winit::Adapter`).
//!
//! BTreeMap (not HashMap) per ADR-0022: ph2d-a11y can be reached
//! from sim-tier code (script-driven editor automation in M9+),
//! so we honor the workspace ban defensively.

use crate::node::NodeId;
use accesskit::{Node, Tree as AkTree, TreeId, TreeUpdate};
use std::collections::BTreeMap;

pub struct Tree {
    nodes: BTreeMap<NodeId, Node>,
    root: NodeId,
    focus: Option<NodeId>,
    /// Node ids whose Node was added/updated since the last
    /// `take_update`. Empty between frames; emit a TreeUpdate that
    /// includes only changed nodes (AccessKit supports incremental).
    dirty: Vec<NodeId>,
}

impl Tree {
    pub fn new(root: NodeId) -> Self {
        Self {
            nodes: BTreeMap::new(),
            root,
            focus: None,
            dirty: Vec::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Insert a node by id, or replace an existing one. Marks dirty.
    pub fn insert(&mut self, id: NodeId, node: Node) {
        self.nodes.insert(id, node);
        self.dirty.push(id);
    }

    /// Replace a node only if it already exists. Marks dirty if
    /// replaced. Returns whether a replacement happened.
    pub fn update(&mut self, id: NodeId, node: Node) -> bool {
        match self.nodes.entry(id) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                e.insert(node);
                self.dirty.push(id);
                true
            }
            std::collections::btree_map::Entry::Vacant(_) => false,
        }
    }

    /// Remove a node. Caller is responsible for first removing it
    /// from any parent's children list (AccessKit semantics).
    pub fn remove(&mut self, id: NodeId) -> Option<Node> {
        let removed = self.nodes.remove(&id);
        if removed.is_some() {
            self.dirty.push(id);
        }
        removed
    }

    pub fn set_focus(&mut self, id: Option<NodeId>) {
        self.focus = id;
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Drain the dirty set into a `TreeUpdate` ready for the OS
    /// adapter. After this call, `dirty` is empty.
    ///
    /// Includes the full AccessKit `Tree` metadata (root + app name)
    /// only on the first call (or when root changed); subsequent
    /// calls send incremental updates.
    pub fn take_update(&mut self) -> TreeUpdate {
        // Dedup to avoid sending the same node twice in one update.
        self.dirty.sort();
        self.dirty.dedup();

        let nodes: Vec<(accesskit::NodeId, Node)> = self
            .dirty
            .drain(..)
            .filter_map(|id| self.nodes.get(&id).cloned().map(|n| (id.into(), n)))
            .collect();

        TreeUpdate {
            nodes,
            tree: Some(AkTree::new(self.root.into())),
            tree_id: TreeId::ROOT,
            focus: self
                .focus
                .map(Into::into)
                .unwrap_or_else(|| self.root.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBuilder;
    use accesskit::Role;

    #[test]
    fn empty_tree_starts_with_root_only() {
        let t = Tree::new(NodeId::ROOT);
        assert_eq!(t.root(), NodeId::ROOT);
        assert!(t.is_empty());
        assert_eq!(t.dirty_count(), 0);
    }

    #[test]
    fn insert_marks_dirty_and_take_update_clears() {
        let mut t = Tree::new(NodeId::ROOT);
        let root_node = NodeBuilder::new(Role::Window).label("Editor").build();
        t.insert(NodeId::ROOT, root_node);

        let btn_id = NodeId(1);
        let btn = NodeBuilder::new(Role::Button)
            .label("Save")
            .focusable(true)
            .build();
        t.insert(btn_id, btn);

        assert_eq!(t.len(), 2);
        assert_eq!(t.dirty_count(), 2);

        let update = t.take_update();
        assert_eq!(update.nodes.len(), 2, "both insertions in update");
        assert_eq!(t.dirty_count(), 0, "dirty cleared after take");
    }

    #[test]
    fn update_only_replaces_existing() {
        let mut t = Tree::new(NodeId::ROOT);
        let new_node = NodeBuilder::new(Role::Window).label("X").build();
        // No existing node → no-op.
        assert!(!t.update(NodeId(99), new_node));
        assert_eq!(t.len(), 0);

        let initial = NodeBuilder::new(Role::Window).label("Initial").build();
        t.insert(NodeId(1), initial);
        let updated = NodeBuilder::new(Role::Window).label("Updated").build();
        assert!(t.update(NodeId(1), updated));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn remove_drops_node_and_dirties_id() {
        let mut t = Tree::new(NodeId::ROOT);
        let n = NodeBuilder::new(Role::Button)
            .label("X")
            .focusable(true)
            .build();
        t.insert(NodeId(1), n);
        let _ = t.take_update(); // clear dirty
        let removed = t.remove(NodeId(1));
        assert!(removed.is_some());
        assert_eq!(t.len(), 0);
        assert_eq!(t.dirty_count(), 1);
    }

    #[test]
    fn focus_round_trips_into_update() {
        let mut t = Tree::new(NodeId::ROOT);
        let n = NodeBuilder::new(Role::Window).label("R").build();
        t.insert(NodeId::ROOT, n);
        t.set_focus(Some(NodeId(42)));
        let update = t.take_update();
        let expected: accesskit::NodeId = NodeId(42).into();
        assert_eq!(update.focus, expected);
    }

    #[test]
    fn focus_defaults_to_root_when_none() {
        let mut t = Tree::new(NodeId::ROOT);
        let n = NodeBuilder::new(Role::Window).label("R").build();
        t.insert(NodeId::ROOT, n);
        let update = t.take_update();
        let expected: accesskit::NodeId = NodeId::ROOT.into();
        assert_eq!(update.focus, expected);
    }

    #[test]
    fn multiple_updates_to_same_id_dedup() {
        let mut t = Tree::new(NodeId::ROOT);
        let n = NodeBuilder::new(Role::Window).label("X").build();
        t.insert(NodeId(1), n.clone());
        t.insert(NodeId(1), n.clone());
        t.insert(NodeId(1), n);
        let update = t.take_update();
        assert_eq!(update.nodes.len(), 1, "duplicate ids deduped per update");
    }
}

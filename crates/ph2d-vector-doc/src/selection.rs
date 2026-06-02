//! [`VectorSelection`] — editor selection state over the committed vector
//! scene.
//!
//! Per plan **T2.3** (`docs/Vector Module/17_plano_de_implementacao.md` §5)
//! and the Coord decision (`787b6fc`): the Select and Direct-Select tools are two
//! **separate** `Tool` instances that must **share** one document
//! selection (switching V↔A preserves it, Illustrator-style). So the
//! selection can live in neither tool — the shell (`App`) owns one
//! [`VectorSelection`] instance and passes it by-ref to both tools'
//! interaction handlers + the selection overlay.
//!
//! The type lives here (not the shell, not a new satellite) because the
//! tool crates must *see* it to take `&mut VectorSelection` — the
//! shell→tools dependency direction rules out a shell-defined type. It is
//! pure transient editor state: **not** serialized into [`crate::Ph2dVectorAsset`].
//!
//! ## Indexing model
//!
//! `networks` indexes the shell's committed-scene `Vec<Ph2dVectorAsset>`.
//! `vertices` pairs that same asset index with a [`VertexId`]. The
//! committed list is append-only + clear within a W1/W2 session, so plain
//! indices are stable; [`VectorSelection::retain_below`] drops stale ones
//! after a scene clear.

use crate::cubic::VertexId;

/// Editor selection over the committed vector scene. Network-level
/// (Select tool) + vertex-level (Direct-Select tool).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VectorSelection {
    /// Selected whole networks — indices into the committed-scene list.
    pub networks: Vec<usize>,
    /// Selected vertices — `(committed asset index, vertex id)`.
    pub vertices: Vec<(usize, VertexId)>,
}

impl VectorSelection {
    /// Empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` if nothing is selected at either level.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.networks.is_empty() && self.vertices.is_empty()
    }

    /// Drop everything.
    pub fn clear(&mut self) {
        self.networks.clear();
        self.vertices.clear();
    }

    // ---- network-level (Select tool) ----

    /// `true` if network `idx` is selected.
    #[must_use]
    pub fn contains_network(&self, idx: usize) -> bool {
        self.networks.contains(&idx)
    }

    /// Replace the whole selection with just network `idx` (a plain click).
    /// Also clears vertex selection (network select supersedes it).
    pub fn select_only_network(&mut self, idx: usize) {
        self.networks.clear();
        self.vertices.clear();
        self.networks.push(idx);
    }

    /// Toggle network `idx` in/out of the selection (shift-click), keeping
    /// the rest. Sorted + deduped for deterministic order.
    pub fn toggle_network(&mut self, idx: usize) {
        if let Some(pos) = self.networks.iter().position(|&n| n == idx) {
            self.networks.remove(pos);
        } else {
            self.networks.push(idx);
            self.networks.sort_unstable();
        }
    }

    /// Replace the network selection with `indices` (a marquee result).
    /// Deduped + sorted; vertex selection is cleared.
    pub fn set_networks(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable();
        indices.dedup();
        self.networks = indices;
        self.vertices.clear();
    }

    // ---- vertex-level (Direct-Select tool) ----

    /// `true` if `(asset, vid)` is selected.
    #[must_use]
    pub fn contains_vertex(&self, asset: usize, vid: VertexId) -> bool {
        self.vertices.contains(&(asset, vid))
    }

    /// Replace the whole selection with just one vertex (a plain
    /// Direct-Select click). Clears network selection.
    pub fn select_only_vertex(&mut self, asset: usize, vid: VertexId) {
        self.networks.clear();
        self.vertices.clear();
        self.vertices.push((asset, vid));
    }

    /// Toggle one vertex (shift-click in Direct-Select).
    pub fn toggle_vertex(&mut self, asset: usize, vid: VertexId) {
        if let Some(pos) = self.vertices.iter().position(|&v| v == (asset, vid)) {
            self.vertices.remove(pos);
        } else {
            self.vertices.push((asset, vid));
            self.vertices.sort_unstable();
        }
    }

    // ---- maintenance ----

    /// Drop any selection referencing an asset index `>= len` — call after
    /// the committed scene shrinks (e.g. an Esc-clear) so stale indices
    /// can't dangle into a reused slot.
    pub fn retain_below(&mut self, len: usize) {
        self.networks.retain(|&n| n < len);
        self.vertices.retain(|&(a, _)| a < len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        let s = VectorSelection::new();
        assert!(s.is_empty());
    }

    #[test]
    fn select_only_network_replaces_and_clears_vertices() {
        let mut s = VectorSelection::new();
        s.vertices.push((9, 1));
        s.select_only_network(3);
        assert_eq!(s.networks, vec![3]);
        assert!(s.vertices.is_empty());
        assert!(s.contains_network(3));
    }

    #[test]
    fn toggle_network_adds_then_removes() {
        let mut s = VectorSelection::new();
        s.toggle_network(2);
        s.toggle_network(5);
        assert_eq!(s.networks, vec![2, 5]);
        s.toggle_network(2);
        assert_eq!(s.networks, vec![5]);
    }

    #[test]
    fn set_networks_dedups_and_sorts() {
        let mut s = VectorSelection::new();
        s.set_networks(vec![3, 1, 3, 2, 1]);
        assert_eq!(s.networks, vec![1, 2, 3]);
    }

    #[test]
    fn select_only_vertex_replaces_and_clears_networks() {
        let mut s = VectorSelection::new();
        s.networks.push(4);
        s.select_only_vertex(2, 7);
        assert!(s.networks.is_empty());
        assert!(s.contains_vertex(2, 7));
    }

    #[test]
    fn toggle_vertex_adds_then_removes() {
        let mut s = VectorSelection::new();
        s.toggle_vertex(0, 1);
        s.toggle_vertex(0, 2);
        assert!(s.contains_vertex(0, 1) && s.contains_vertex(0, 2));
        s.toggle_vertex(0, 1);
        assert!(!s.contains_vertex(0, 1) && s.contains_vertex(0, 2));
    }

    #[test]
    fn retain_below_drops_stale_indices() {
        let mut s = VectorSelection::new();
        s.networks = vec![0, 2, 5];
        s.vertices = vec![(1, 0), (5, 3)];
        s.retain_below(3); // committed scene now has 3 assets (0..2)
        assert_eq!(s.networks, vec![0, 2]);
        assert_eq!(s.vertices, vec![(1, 0)]);
    }
}

//! **What a copy needs to EQUAL its source** beyond the textual format: the next id to mint and the
//! live edge order (the format derives one and sorts the other). The macro stage (ADR-0186) loads a
//! document from text and must mint the same ids and hold the same edges as the one it copies.
//! A child of `graph` (sees its private fields), split off for the file LOC cap.

use super::{Edge, Graph};

impl Graph {
    /// The id the next [`Self::add_node`] will mint. Not part of the textual format (a load
    /// derives it as `max + 1`), so a copy that must mint the SAME ids as its source carries it
    /// beside the text — the macro stage does (ADR-0186).
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// Restores [`Self::next_id`] on a copy loaded from text. Never below `max + 1`: an id
    /// already present can only be minted again by a corrupt input, and then it is not.
    pub fn restore_next_id(&mut self, next: u32) {
        let floor = self
            .nodes
            .iter()
            .map(|n| n.id.0.saturating_add(1))
            .max()
            .unwrap_or(0);
        self.next_id = next.max(floor);
    }

    /// Puts the edges in the order of `order` — the live order, which the textual format does not
    /// keep (it sorts them), carried beside the text by a copy that must equal its source (the macro
    /// stage, ADR-0186). `false`, untouched, unless `order` holds exactly these edges.
    pub fn order_edges_as(&mut self, order: &[Edge]) -> bool {
        let mut a = self.edges.clone();
        let mut b = order.to_vec();
        let key = |e: &Edge| (e.from.0.0, e.from.1, e.to.0.0, e.to.1, e.delayed);
        a.sort_by_key(key);
        b.sort_by_key(key);
        if a != b {
            return false;
        }
        self.edges = order.to_vec();
        true
    }
}

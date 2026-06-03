//! Vector scene undo/redo (W2.T2.5) — snapshot-based.
//!
//! A user action that mutates the committed vector scene (a Pen/Pencil/Shape
//! commit, a Direct vertex/tangent edit, a recolor, an Esc-clear) first calls
//! [`App::vector_checkpoint`], which pushes a clone of
//! `committed_vector_pen_paths` onto the undo stack. Ctrl+Z restores the top of
//! that stack; Ctrl+Shift+Z / Ctrl+Y replays the redo stack.
//!
//! **Why snapshot, not the event-sourced `revert_last_op`:** one Ctrl+Z must
//! revert a whole user action across MANY assets (a recolor touches every
//! selected network; a commit appends one) and stay coherent with the
//! scene-object mirror — `vector_scene::reconcile` re-aligns the ECS entities
//! to the restored asset list each frame, so a snapshot restore "just works".
//! Gizmo MOVES (entity `Transform`) are a separate scene-undo domain and are
//! NOT captured here (the checkpoints fire on asset mutations only).

use ph2d_editor::toast::Toast;
use ph2d_vector::Ph2dVectorAsset;

use crate::App;

/// Bounded undo depth — oldest snapshot dropped beyond this. Vector scenes are
/// small (a handful of assets), so a deep history is cheap.
const VECTOR_UNDO_DEPTH: usize = 64;

/// Push `committed` onto `undo` (bounded) and clear `redo`. Free function so
/// both [`App::vector_checkpoint`] and the inspector recolor bridge (which only
/// borrows the stacks + committed slice, not all of `App`) can checkpoint.
pub(crate) fn checkpoint(
    undo: &mut Vec<Vec<Ph2dVectorAsset>>,
    redo: &mut Vec<Vec<Ph2dVectorAsset>>,
    committed: &[Ph2dVectorAsset],
) {
    undo.push(committed.to_vec());
    if undo.len() > VECTOR_UNDO_DEPTH {
        undo.remove(0);
    }
    redo.clear();
}

impl App {
    /// Snapshot the committed scene before an imminent vector mutation.
    pub(crate) fn vector_checkpoint(&mut self) {
        checkpoint(
            &mut self.vector_undo_stack,
            &mut self.vector_redo_stack,
            &self.committed_vector_pen_paths,
        );
    }

    fn push_vector_undo_toast(&mut self, msg: &'static str) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(Toast::info(msg));
        }
    }

    /// Ctrl+Z — restore the previous committed-scene snapshot. The current
    /// state moves to the redo stack. Selection is cleared (its indices may be
    /// stale against the restored scene); `reconcile` re-aligns the ECS mirror
    /// next frame. Returns `true` (consumes the key) iff something was undone.
    pub(crate) fn vector_undo(&mut self) -> bool {
        let Some(prev) = self.vector_undo_stack.pop() else {
            self.push_vector_undo_toast("Nothing to undo");
            return false;
        };
        self.vector_redo_stack
            .push(self.committed_vector_pen_paths.clone());
        self.committed_vector_pen_paths = prev;
        self.vector_selection.clear();
        self.push_vector_undo_toast("Undo");
        true
    }

    /// Ctrl+Shift+Z / Ctrl+Y — replay the next redo snapshot.
    pub(crate) fn vector_redo(&mut self) -> bool {
        let Some(next) = self.vector_redo_stack.pop() else {
            self.push_vector_undo_toast("Nothing to redo");
            return false;
        };
        self.vector_undo_stack
            .push(self.committed_vector_pen_paths.clone());
        self.committed_vector_pen_paths = next;
        self.vector_selection.clear();
        self.push_vector_undo_toast("Redo");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{StyleTable, VectorNetwork};

    fn asset(n: u32) -> Ph2dVectorAsset {
        // Distinct assets by vertex count so snapshots compare unequal.
        let mut net = VectorNetwork::empty();
        for i in 0..n {
            net.vertices
                .push(ph2d_vector_doc::Vertex::auto(i, ph2d_core::Vec2::new(i as f32, 0.0)));
        }
        Ph2dVectorAsset::from_network(net, StyleTable::default())
    }

    #[test]
    fn checkpoint_pushes_and_clears_redo() {
        let mut undo = Vec::new();
        let mut redo = vec![vec![asset(9)]]; // stale redo entry
        checkpoint(&mut undo, &mut redo, &[asset(1)]);
        assert_eq!(undo.len(), 1);
        assert!(redo.is_empty(), "a fresh checkpoint invalidates redo");
    }

    #[test]
    fn checkpoint_is_depth_bounded() {
        let mut undo = Vec::new();
        let mut redo = Vec::new();
        for _ in 0..(VECTOR_UNDO_DEPTH + 10) {
            checkpoint(&mut undo, &mut redo, &[asset(1)]);
        }
        assert_eq!(undo.len(), VECTOR_UNDO_DEPTH, "oldest snapshots dropped");
    }
}

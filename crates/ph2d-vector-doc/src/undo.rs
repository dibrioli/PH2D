//! [`VectorUndoAction`] — document-level undo/redo over the committed
//! vector scene.
//!
//! Per plan **T2.5** + Coord decision (`04459c3` handoff): undo is a
//! *document* operation (Ctrl+Z works regardless of the active tool), so
//! the shell owns the undo + redo **stacks** in `App`; this module owns the
//! **pure transition logic** that operates on `(committed scene, selection)`
//! by-ref — same split (and home) as [`crate::VectorSelection`].
//!
//! ## Action model (Coord-blessed)
//!
//! Two user-action granularities cover the four W2 tools:
//! - **Create** — a create-tool (Pen / Pencil / Shape) appended a network.
//!   Undo pops it; redo re-appends it.
//! - **Edit** — Direct-Select applied a `Move*` op to a network. Undo
//!   reverts the last op ([`crate::EditLog::revert_last_op`]); redo
//!   re-applies it.
//!
//! Each direction returns the *opposite* action carrying the payload the
//! reverse needs (the popped asset / the reverted op), so the shell just
//! shuffles entries between its undo and redo stacks.
//!
//! ## LIFO contract
//!
//! Undo is strict LIFO. Creates only ever *append*, so the most-recently
//! created network is the last element — [`apply_undo`] of a `Create` pops
//! the tail (no index shift can dangle the other stack entries). `Edit`
//! indices stay valid because edits never change the scene length and the
//! edited asset is never removed before its own edits are undone.

use crate::edit_log::VectorOp;
use crate::postcard_schema::Ph2dVectorAsset;
use crate::selection::VectorSelection;

/// A user action recorded on the shell's **undo** stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorUndoAction {
    /// A create-tool appended a network (the tail of the committed scene
    /// at record time). The index is informational; undo pops the tail.
    Create {
        /// Committed index at create time (`== len − 1`).
        asset: usize,
    },
    /// Direct-Select applied a `Move*` op to a network.
    Edit {
        /// Committed index of the edited network.
        asset: usize,
    },
}

/// The matching entry pushed onto the shell's **redo** stack — carries the
/// payload needed to replay the undone action forward.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorRedoAction {
    /// Re-append a network that an undo popped.
    Recreate {
        /// The popped asset, to push back onto the scene.
        asset: Box<Ph2dVectorAsset>,
    },
    /// Re-apply an op that an undo reverted on network `asset`.
    Reedit {
        /// Committed index to re-apply onto.
        asset: usize,
        /// The op to re-apply.
        op: VectorOp,
    },
}

/// Perform `action` in the **undo** direction. Returns the [`VectorRedoAction`]
/// to push onto the redo stack, or `None` if the action can't apply (empty
/// scene / stale index — a no-op).
pub fn apply_undo(
    action: VectorUndoAction,
    committed: &mut Vec<Ph2dVectorAsset>,
    selection: &mut VectorSelection,
) -> Option<VectorRedoAction> {
    match action {
        VectorUndoAction::Create { .. } => {
            // LIFO: the created network is the tail.
            let asset = committed.pop()?;
            selection.retain_below(committed.len());
            Some(VectorRedoAction::Recreate {
                asset: Box::new(asset),
            })
        }
        VectorUndoAction::Edit { asset } => {
            let a = committed.get_mut(asset)?;
            let op = a.edit_log.revert_last_op(&mut a.network)?;
            Some(VectorRedoAction::Reedit { asset, op })
        }
    }
}

/// Perform `action` in the **redo** direction. Returns the
/// [`VectorUndoAction`] to push back onto the undo stack, or `None` on a
/// stale index.
pub fn apply_redo(
    action: VectorRedoAction,
    committed: &mut Vec<Ph2dVectorAsset>,
    selection: &mut VectorSelection,
) -> Option<VectorUndoAction> {
    match action {
        VectorRedoAction::Recreate { asset } => {
            committed.push(*asset);
            Some(VectorUndoAction::Create {
                asset: committed.len() - 1,
            })
        }
        VectorRedoAction::Reedit { asset, op } => {
            let a = committed.get_mut(asset)?;
            a.edit_log.push_and_apply(op, &mut a.network).ok()?;
            // Redoing an edit doesn't touch the selection, but keep it sane.
            let _ = selection;
            Some(VectorUndoAction::Edit { asset })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubic::VertexKind;
    use crate::network::VectorNetwork;
    use crate::style::StyleTable;
    use glam::Vec2;

    /// An asset with two vertices + one segment (a minimal editable path).
    fn segment_asset() -> Ph2dVectorAsset {
        let mut net = VectorNetwork::empty();
        let mut log = crate::EditLog::new();
        let _ = log.push_and_apply(
            VectorOp::AddVertex {
                id: 0,
                pos: Vec2::ZERO,
                kind: VertexKind::Auto,
            },
            &mut net,
        );
        let _ = log.push_and_apply(
            VectorOp::AddVertex {
                id: 1,
                pos: Vec2::new(10.0, 0.0),
                kind: VertexKind::Auto,
            },
            &mut net,
        );
        let mut asset = Ph2dVectorAsset::from_network(net, StyleTable::default());
        asset.edit_log = log;
        asset
    }

    #[test]
    fn create_undo_pops_tail_and_redo_restores() {
        let mut committed = vec![segment_asset(), segment_asset()];
        let mut sel = VectorSelection::new();
        sel.select_only_network(1);
        // Undo the creation of the last (idx 1).
        let redo = apply_undo(VectorUndoAction::Create { asset: 1 }, &mut committed, &mut sel)
            .expect("redo action");
        assert_eq!(committed.len(), 1);
        assert!(sel.is_empty(), "stale selection pruned by retain_below");
        assert!(matches!(redo, VectorRedoAction::Recreate { .. }));
        // Redo re-appends it.
        let undo = apply_redo(redo, &mut committed, &mut sel).expect("undo action");
        assert_eq!(committed.len(), 2);
        assert_eq!(undo, VectorUndoAction::Create { asset: 1 });
    }

    #[test]
    fn edit_undo_reverts_last_op_and_redo_reapplies() {
        let mut committed = vec![segment_asset()];
        let mut sel = VectorSelection::new();
        // Apply a move (as Direct-Select would).
        {
            let a = &mut committed[0];
            let _ = a.edit_log.push_and_apply(
                VectorOp::MoveVertex {
                    id: 1,
                    new_pos: Vec2::new(50.0, 20.0),
                },
                &mut a.network,
            );
        }
        let moved = committed[0].network.vertices.iter().find(|v| v.id == 1).unwrap().pos;
        assert_eq!(moved, Vec2::new(50.0, 20.0));
        // Undo the edit.
        let redo = apply_undo(VectorUndoAction::Edit { asset: 0 }, &mut committed, &mut sel)
            .expect("redo");
        let back = committed[0].network.vertices.iter().find(|v| v.id == 1).unwrap().pos;
        assert_eq!(back, Vec2::new(10.0, 0.0), "vertex restored");
        assert!(matches!(redo, VectorRedoAction::Reedit { asset: 0, .. }));
        // Redo re-applies the move.
        apply_redo(redo, &mut committed, &mut sel).expect("undo");
        let redone = committed[0].network.vertices.iter().find(|v| v.id == 1).unwrap().pos;
        assert_eq!(redone, Vec2::new(50.0, 20.0));
    }

    #[test]
    fn create_undo_on_empty_scene_is_none() {
        let mut committed: Vec<Ph2dVectorAsset> = Vec::new();
        let mut sel = VectorSelection::new();
        assert!(apply_undo(VectorUndoAction::Create { asset: 0 }, &mut committed, &mut sel).is_none());
    }

    #[test]
    fn edit_undo_on_stale_index_is_none() {
        let mut committed = vec![segment_asset()];
        let mut sel = VectorSelection::new();
        assert!(apply_undo(VectorUndoAction::Edit { asset: 9 }, &mut committed, &mut sel).is_none());
    }

    #[test]
    fn round_trip_create_then_edit_undo_order() {
        // Create A, Create B, Edit B → LIFO undo: Edit B, Create B, Create A.
        let mut committed = vec![segment_asset(), segment_asset()];
        let mut sel = VectorSelection::new();
        {
            let b = &mut committed[1];
            let _ = b.edit_log.push_and_apply(
                VectorOp::MoveVertex { id: 0, new_pos: Vec2::new(5.0, 5.0) },
                &mut b.network,
            );
        }
        // Undo Edit B.
        apply_undo(VectorUndoAction::Edit { asset: 1 }, &mut committed, &mut sel).unwrap();
        assert_eq!(committed[1].network.vertices.iter().find(|v| v.id == 0).unwrap().pos, Vec2::ZERO);
        // Undo Create B (pop tail).
        apply_undo(VectorUndoAction::Create { asset: 1 }, &mut committed, &mut sel).unwrap();
        assert_eq!(committed.len(), 1);
        // Undo Create A.
        apply_undo(VectorUndoAction::Create { asset: 0 }, &mut committed, &mut sel).unwrap();
        assert!(committed.is_empty());
    }
}

//! Painter undo/redo controller — **transactional (structural) layer undo**.
//!
//! # What it covers
//!
//! Every structural layer edit — add / delete / duplicate / group / reorder a
//! layer, create a mask or adjustment, move a layer, switch the active layer —
//! is recorded as a *before/after* pair of full model snapshots. Undo rolls the
//! model back to `before`; redo rolls forward to `after`. This is the
//! layers + effects editor's undo history.
//!
//! # Design
//!
//! [`UndoController`] keeps two chronological stacks of [`UndoEntry`] (the
//! `undo` stack the user can step back through, the `redo` stack populated by
//! `undo` and cleared by any new edit — the standard linear-history contract),
//! bounded to `max_depth`. Each entry carries BOTH endpoints (`before` + `after`
//! [`ModelSnapshot`]s) so the swap needs no live state and is allocation-stable;
//! a snapshot's `canvas_rgba` is `Arc`-shared so the clone on a (rare, user-paced)
//! structural edit is cheap CoW.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::{LayerId as RtLayerId, LayerStack};

/// A full snapshot of the editable layer model, for **transactional (structural)
/// undo** — adding / deleting / duplicating a layer, creating a mask or
/// adjustment, or switching the active layer. A structural edit reshapes the
/// whole model, so the undo entry stores the entire state to roll back to.
///
/// `canvas_rgba` is `Arc`-shared (cheap CoW clone — the tool already wraps the
/// active layer's working buffer in an `Arc`); the active layer id lives INSIDE
/// `layers` (`LayerStack` owns it), so restoring `layers` restores the active
/// target too. `images` is the per-layer pixel store for every NON-active layer.
#[derive(Clone, Debug)]
pub struct ModelSnapshot {
    pub layers: LayerStack,
    pub images: BTreeMap<RtLayerId, LayerImage>,
    pub canvas_rgba: Arc<Vec<u8>>,
    pub selection: BTreeSet<RtLayerId>,
}

/// Default cap on retained undo entries (ring depth). The caller can raise or
/// lower it from its memory budget. The oldest entry beyond the cap is dropped
/// (that depth becomes non-undoable, like a ring history).
pub const DEFAULT_MAX_DEPTH: usize = 300;

/// One retained history entry: a structural edit stored as BOTH endpoints (the
/// model `before` and `after` the edit). Carrying both means the entry needs no
/// live state to swap directions; structural edits are user-paced (rare) so two
/// model snapshots is a fine trade for the simpler, allocation-stable swap.
#[derive(Clone, Debug)]
struct UndoEntry {
    before: Box<ModelSnapshot>,
    after: Box<ModelSnapshot>,
}

/// Snapshot-based undo/redo for the editable layer model.
///
/// The caller (the [`PainterTool`](crate::tool::PainterTool)) drives it at two
/// points: just after a structural edit commits ([`Self::record_structural`]),
/// and when the gesture/shell requests [`Self::undo`] / [`Self::redo`] (each
/// returns the [`ModelSnapshot`] the caller must reinstall).
#[derive(Debug)]
pub struct UndoController {
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    max_depth: usize,
}

impl Default for UndoController {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_DEPTH)
    }
}

impl UndoController {
    /// New controller with an explicit retained-depth ceiling.
    #[must_use]
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            // A depth of 0 would make undo permanently unavailable; clamp to 1
            // so a degenerate budget still allows a single level.
            max_depth: max_depth.max(1),
        }
    }

    /// Record a STRUCTURAL transition (add/delete/duplicate layer, mask/adjustment
    /// create, active-layer switch). `before` is the model to roll back to on undo;
    /// `after` is the model to roll forward to on redo. Pushing it clears the redo
    /// branch (standard linear-history semantics).
    pub fn record_structural(&mut self, before: ModelSnapshot, after: ModelSnapshot) {
        self.undo.push(UndoEntry {
            before: Box::new(before),
            after: Box::new(after),
        });
        self.redo.clear();
        self.cap();
    }

    /// Undo the most recent structural edit: roll back to its `before` model and
    /// park the entry on the redo stack so a later [`Self::redo`] can roll forward
    /// to `after`. Returns the model to reinstall, or `None` if nothing to undo.
    pub fn undo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.undo.pop()?;
        let restore = entry.before.clone();
        self.redo.push(entry);
        Some(restore)
    }

    /// Redo the most recently undone structural edit: roll forward to its `after`
    /// model and park the entry back on the undo stack. Returns the model to
    /// reinstall, or `None` if the redo stack is empty.
    pub fn redo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.redo.pop()?;
        let restore = entry.after.clone();
        self.undo.push(entry);
        Some(restore)
    }

    /// `true` if there is at least one edit to undo (drives the `undo_enabled`
    /// affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if there is at least one undone edit to redo (drives the
    /// `redo_enabled` affordance).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Retained undo depth (for tests / memory introspection).
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Retained redo depth.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Drop the controller's history (e.g. on `set_source` of a fresh canvas).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Enforce the ring depth ceiling: drop the oldest entries (front of the
    /// Vec) so the retained undo stack never exceeds `max_depth`.
    fn cap(&mut self) {
        if self.undo.len() > self.max_depth {
            let overflow = self.undo.len() - self.max_depth;
            self.undo.drain(0..overflow);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(active_px: u8) -> ModelSnapshot {
        ModelSnapshot {
            layers: LayerStack::new(),
            images: BTreeMap::new(),
            canvas_rgba: Arc::new(vec![active_px; 16]),
            selection: BTreeSet::new(),
        }
    }

    #[test]
    fn undo_rolls_back_to_before() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0x11), model(0x22));
        assert!(c.can_undo());
        let restored = c.undo().expect("one entry to undo");
        assert_eq!(restored.canvas_rgba.as_slice(), &[0x11; 16]);
        assert!(!c.can_undo());
        assert!(c.can_redo());
    }

    #[test]
    fn redo_rolls_forward_to_after() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0x11), model(0x22));
        c.undo();
        let restored = c.redo().expect("one entry to redo");
        assert_eq!(restored.canvas_rgba.as_slice(), &[0x22; 16]);
        assert!(c.can_undo());
        assert!(!c.can_redo());
    }

    #[test]
    fn new_edit_clears_redo_branch() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0), model(1));
        c.record_structural(model(1), model(2));
        c.undo();
        assert!(c.can_redo());
        c.record_structural(model(1), model(3));
        assert!(!c.can_redo(), "a new edit must invalidate the redo branch");
    }

    #[test]
    fn depth_cap_drops_oldest() {
        let mut c = UndoController::new(4);
        for v in 0..10u8 {
            c.record_structural(model(v), model(v + 1));
        }
        assert!(c.undo_depth() <= 4, "cap enforced; got {}", c.undo_depth());
    }

    #[test]
    fn clear_drops_both_stacks() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0), model(1));
        c.undo();
        c.clear();
        assert!(!c.can_undo() && !c.can_redo());
    }
}

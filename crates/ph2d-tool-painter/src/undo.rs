//! W2.T2.2 — Painter undo/redo controller (snapshot-based layer recompose).
//!
//! # Why a dedicated controller (gap analysis)
//!
//! [`crate::tool::PainterTool`] already owns a
//! [`ph2d_painter_stroke::StrokeHistory`] — the *semantic* canon of strokes
//! (one [`StrokeRecord`](ph2d_painter_stroke::StrokeRecord) per committed
//! stroke). But pre-T2.2:
//!
//! - `PainterUiEdit::Undo` popped the semantic record (`stroke_history.undo()`)
//!   and **dropped** it, so [`ph2d_painter_stroke::StrokeHistory::redo`] (which
//!   needs the caller to hold the popped record) could never run — `Redo` was a
//!   literal no-op and `redo_enabled` was hardcoded `false`.
//! - Neither path re-composed the **layer pixels** (`canvas_rgba`). The visible
//!   canvas never reverted, so the spec's criterion #1 ("undo restores the layer
//!   to the pre-stroke state") was unmet.
//!
//! There is no stroke *replay* engine yet (that is W11 `T-replay` — turning a
//! `StrokeRecord` back into stamps on a blank layer). So a "replay forward from
//! the nearest checkpoint" undo is not yet possible. The W2 spec itself
//! prescribes the practical path (15_plano §5 T2.2 + the in-tool comment
//! "snapshot-based undo, texture clone preserved"): **snapshot the layer texture
//! at each stroke boundary and restore it on undo/redo.** That is always correct
//! for criteria #1/#2/#3 and is independent of the not-yet-existing replay path.
//!
//! # Design
//!
//! [`UndoController`] keeps two stacks of *layer textures* (RGBA8 straight,
//! the `canvas_rgba` wire format):
//!
//! - `undo`: states the user can step BACK to. The top is the layer **before**
//!   the most recent committed stroke.
//! - `redo`: states the user can step FORWARD to (populated by `undo`, cleared
//!   by any new stroke — the standard linear-history contract).
//!
//! Each entry is a [`LayerSnapshot`] keyed by the `seq` of the stroke whose
//! commit produced it. The controller keeps **every** stroke's texture up to a
//! ring depth of `max_depth` (the oldest falls off, exactly like a ring
//! [`ph2d_painter_stroke::StrokeHistory`]), so every retained stroke is
//! single-step undoable — meeting the spec's "undo 250×" criterion directly.
//!
//! The controller deliberately stores **full textures** (not deltas) in W2:
//! deltas / "snapshot every 50 + replay forward" (15_plano §5 T2.2 /
//! 01 §1.14.3 / ADR-0046 §2.6) need the stroke *replay* engine to reconstruct
//! the intermediate states between checkpoints, and that engine is W11
//! (`T-replay` — no `StrokeRecord` → stamps rasterizer exists yet). Once it
//! lands, this full-texture ring is replaced by `SNAPSHOT_STROKE_INTERVAL`-
//! spaced [`LayerSnapshot`] checkpoints + forward replay, shrinking memory by
//! ~50×. Until then, full textures are the only way to make every single undo
//! exact, so we pay the memory and bound it with `max_depth`.

use ph2d_painter_stroke::{PersistLayerId, LayerSnapshot};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::{LayerId as RtLayerId, LayerStack};

/// A full snapshot of the editable layer model, for **transactional (structural)
/// undo** — adding / deleting / duplicating a layer, creating a mask or
/// adjustment, or switching the active layer. Unlike a stroke (which mutates only
/// the active layer's pixels), a structural edit reshapes the whole model, so the
/// undo entry stores the entire state to roll back to.
///
/// `canvas_rgba` is `Arc`-shared (cheap CoW clone — the tool already wraps the
/// active layer's working buffer in an `Arc`); the active layer id lives INSIDE
/// `layers` (`LayerStack` owns it), so restoring `layers` restores the paint
/// target too. `images` is the per-layer pixel store for every NON-active layer.
#[derive(Clone, Debug)]
pub struct ModelSnapshot {
    pub layers: LayerStack,
    pub images: BTreeMap<RtLayerId, LayerImage>,
    pub canvas_rgba: Arc<Vec<u8>>,
    pub selection: BTreeSet<RtLayerId>,
}

/// What an [`UndoController::undo`] / [`UndoController::redo`] step restored — the
/// caller applies whichever arm it gets. A stroke step hands back pixels to blit
/// into `canvas_rgba`; a structural step hands back the whole model to install.
pub enum Restore {
    /// Blit these straight-sRGB8 pixels into the active layer (`canvas_rgba`).
    Pixels(Vec<u8>),
    /// Reinstall the whole editable model (a structural edit was reversed).
    Model(Box<ModelSnapshot>),
}

/// Default cap on retained undo entries (ring depth). The caller can raise or
/// lower it from its memory budget. Sized so the spec's "undo 250×" criterion
/// is met with headroom; at 4K RGBA8 (~64 MiB/entry) a real deployment would
/// lower this from its `MemoryBudget`, and the W11 replay engine removes the
/// per-stroke full-texture cost entirely (see module docs). The oldest entry
/// beyond the cap is dropped (that depth becomes non-undoable, like a ring
/// `StrokeHistory`).
pub const DEFAULT_MAX_DEPTH: usize = 300;

/// One retained history entry. Strokes and structural edits share a single
/// chronological stack so undo/redo reverse the user's actions in exact order
/// (a stroke, then an "add layer", then a stroke → undo peels them back the
/// other way).
#[derive(Clone, Debug)]
enum UndoEntry {
    /// A committed stroke's pre-image (the active layer pixels BEFORE stroke
    /// `seq`). Lazy single-snapshot swap: on undo the entry stores the live
    /// post-stroke pixels for the matching redo (so a stroke costs ONE texture,
    /// not two). `snapshot` reuses the canonical schema (ADR-0046 §2.6).
    Stroke { seq: u64, snapshot: LayerSnapshot },
    /// A structural edit, stored as BOTH endpoints (the model `before` and
    /// `after` the edit). Carrying both means the entry needs no live state to
    /// swap directions; structural edits are user-paced (rare) so two model
    /// snapshots is a fine trade for the simpler, allocation-stable swap.
    Structural {
        before: Box<ModelSnapshot>,
        after: Box<ModelSnapshot>,
    },
}

/// Snapshot-based undo/redo for a single layer's raster texture.
///
/// The caller (the [`PainterTool`](crate::tool::PainterTool)) drives it at
/// three points: just before a stroke commits ([`Self::record_pre_stroke`]),
/// and when the gesture/shell requests [`Self::undo`] / [`Self::redo`]. Each of
/// the latter returns the texture the caller must blit into `canvas_rgba`.
#[derive(Debug)]
pub struct UndoController {
    layer_id: PersistLayerId,
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    max_depth: usize,
}

impl Default for UndoController {
    fn default() -> Self {
        Self::new(PersistLayerId(0), DEFAULT_MAX_DEPTH)
    }
}

impl UndoController {
    /// New controller for `layer_id` with an explicit retained-depth ceiling.
    #[must_use]
    pub fn new(layer_id: PersistLayerId, max_depth: usize) -> Self {
        Self {
            layer_id,
            undo: Vec::new(),
            redo: Vec::new(),
            // A depth of 0 would make undo permanently unavailable; clamp to 1
            // so a degenerate budget still allows a single level (and never
            // panics on the `len - max_depth` arithmetic below).
            max_depth: max_depth.max(1),
        }
    }

    /// Records the layer texture as it exists *before* stroke `seq` is applied.
    /// Call this at stroke-commit time, passing the layer's current pixels
    /// (i.e. the state to roll back to). Pushing a new stroke clears the redo
    /// stack (standard linear-history semantics).
    ///
    /// `pre_stroke_pixels` is cloned into a [`LayerSnapshot`]; the caller keeps
    /// ownership of its live buffer.
    pub fn record_pre_stroke(
        &mut self,
        seq: u64,
        // COLOR-RAW-OK: opaque layer RGBA8 canvas snapshot blob, cloned + re-blit wholesale.
        pre_stroke_pixels: &[u8],
    ) {
        let snapshot = LayerSnapshot::new_in_memory(self.layer_id, seq, pre_stroke_pixels.to_vec());
        self.undo.push(UndoEntry::Stroke { seq, snapshot });
        // Any new edit invalidates the redo branch.
        self.redo.clear();
        self.cap();
    }

    /// Record a STRUCTURAL transition (add/delete/duplicate layer, mask/adjustment
    /// create, active-layer switch). `before` is the model to roll back to on undo;
    /// `after` is the model to roll forward to on redo. Pushing it clears the redo
    /// branch (standard linear-history semantics), exactly like a stroke.
    pub fn record_structural(&mut self, before: ModelSnapshot, after: ModelSnapshot) {
        self.undo.push(UndoEntry::Structural {
            before: Box::new(before),
            after: Box::new(after),
        });
        self.redo.clear();
        self.cap();
    }

    /// Undo the most recent stroke. `current_pixels` is the live layer (the
    /// *post*-stroke state); the controller returns the texture to restore (the
    /// stored *pre*-image) and parks `current_pixels` on the redo stack so a
    /// subsequent [`Self::redo`] can re-apply it. Returns `None` (and ignores
    /// `current_pixels`) when there is nothing to undo.
    ///
    /// The caller must pair this with `stroke_history.undo()` to keep the
    /// semantic canon in sync (the controller owns *pixels*, `StrokeHistory`
    /// owns the *record*).
    pub fn undo(
        &mut self,
        // COLOR-RAW-OK: opaque layer RGBA8 canvas blob (in + returned), re-blit wholesale.
        current_pixels: &[u8],
    ) -> Option<Restore> {
        match self.undo.pop()? {
            UndoEntry::Stroke { seq, snapshot } => {
                // Swap: hand back the stored pre-image, and stash the current
                // (post-stroke) pixels so redo can restore them. A single snapshot
                // per entry suffices because the entry alternates direction as it
                // moves between stacks.
                let restore = snapshot_pixels(&snapshot)?;
                self.redo.push(UndoEntry::Stroke {
                    seq,
                    snapshot: LayerSnapshot::new_in_memory(
                        self.layer_id,
                        seq,
                        current_pixels.to_vec(),
                    ),
                });
                Some(Restore::Pixels(restore))
            }
            UndoEntry::Structural { before, after } => {
                // Roll back to `before`; keep the entry (both endpoints) on the redo
                // stack so redo can roll forward to `after`. The clone is one model
                // copy on a rare, user-paced path (its `canvas_rgba` is Arc-shared).
                let restore = before.clone();
                self.redo.push(UndoEntry::Structural { before, after });
                Some(Restore::Model(restore))
            }
        }
    }

    /// Redo the most recently undone stroke. `current_pixels` is the live layer
    /// (the *pre*-stroke state after the matching undo); the controller returns
    /// the stored *post*-image and parks `current_pixels` back on the undo
    /// stack. Returns `None` when the redo stack is empty.
    pub fn redo(
        &mut self,
        // COLOR-RAW-OK: opaque layer RGBA8 canvas blob (in + returned), re-blit wholesale.
        current_pixels: &[u8],
    ) -> Option<Restore> {
        match self.redo.pop()? {
            UndoEntry::Stroke { seq, snapshot } => {
                let restore = snapshot_pixels(&snapshot)?;
                self.undo.push(UndoEntry::Stroke {
                    seq,
                    snapshot: LayerSnapshot::new_in_memory(
                        self.layer_id,
                        seq,
                        current_pixels.to_vec(),
                    ),
                });
                Some(Restore::Pixels(restore))
            }
            UndoEntry::Structural { before, after } => {
                // Roll forward to `after`; keep the entry on the undo stack so a
                // later undo can roll back to `before`.
                let restore = after.clone();
                self.undo.push(UndoEntry::Structural { before, after });
                Some(Restore::Model(restore))
            }
        }
    }

    /// `true` if there is at least one stroke to undo (drives the sidebar
    /// `undo_enabled` affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if there is at least one undone stroke to redo (drives the
    /// sidebar `redo_enabled` affordance — previously hardcoded `false`).
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
    /// Vec) so the retained undo stack never exceeds `max_depth`. Every survivor
    /// is single-step undoable (no thinning) — see module docs for why W2 keeps
    /// full per-stroke textures rather than `SNAPSHOT_STROKE_INTERVAL` thinning.
    fn cap(&mut self) {
        if self.undo.len() > self.max_depth {
            let overflow = self.undo.len() - self.max_depth;
            self.undo.drain(0..overflow);
        }
    }
}

/// Extract the RGBA8 pixels from an in-memory stroke snapshot. Returns `None`
/// for an `OnDisk` snapshot (W11+ offload — the in-tool W2 path never produces
/// those, but the controller fails closed rather than panicking if a future
/// caller wires disk-offloaded entries without a loader).
fn snapshot_pixels(snapshot: &LayerSnapshot) -> Option<Vec<u8>> {
    match &snapshot.texture_data {
        ph2d_painter_stroke::SnapshotStorage::InMemory(bytes) => Some(bytes.clone()),
        ph2d_painter_stroke::SnapshotStorage::OnDisk(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(n: usize, v: u8) -> Vec<u8> {
        vec![v; n]
    }

    /// Drive the controller the way `PainterTool` does: a `live` canvas that the
    /// caller mutates, `record_pre_stroke` called at commit (live = post-stroke
    /// state, recorded value = pre-stroke pre-image). Undo/redo swap `live`.
    struct Sim {
        c: UndoController,
        live: Vec<u8>,
    }
    impl Sim {
        fn new(cap: usize, initial: Vec<u8>) -> Self {
            Self {
                c: UndoController::new(PersistLayerId(0), cap),
                live: initial,
            }
        }
        /// Commit a stroke that transitions `live` to `next`.
        fn paint(&mut self, seq: u64, next: Vec<u8>) {
            self.c.record_pre_stroke(seq, &self.live);
            self.live = next;
        }
        fn undo(&mut self) -> bool {
            match self.c.undo(&self.live) {
                Some(Restore::Pixels(px)) => {
                    self.live = px;
                    true
                }
                Some(Restore::Model(_)) => panic!("stroke-only Sim never records structural"),
                None => false,
            }
        }
        fn redo(&mut self) -> bool {
            match self.c.redo(&self.live) {
                Some(Restore::Pixels(px)) => {
                    self.live = px;
                    true
                }
                Some(Restore::Model(_)) => panic!("stroke-only Sim never records structural"),
                None => false,
            }
        }
    }

    #[test]
    fn undo_restores_pre_stroke_texture() {
        let mut s = Sim::new(DEFAULT_MAX_DEPTH, solid(16, 0x11));
        s.paint(0, solid(16, 0x22)); // 0x11 → 0x22
        assert!(s.undo());
        assert_eq!(s.live, solid(16, 0x11), "undo restores the pre-image");
        assert!(!s.undo(), "nothing left to undo");
    }

    #[test]
    fn redo_reapplies_last_undo() {
        let mut s = Sim::new(DEFAULT_MAX_DEPTH, solid(8, 0));
        s.paint(0, solid(8, 0xAA)); // 0 → AA
        s.paint(1, solid(8, 0xBB)); // AA → BB
        assert!(s.undo());
        assert_eq!(s.live, solid(8, 0xAA));
        assert!(s.undo());
        assert_eq!(s.live, solid(8, 0));
        assert!(!s.c.can_undo());
        assert!(s.c.can_redo());
        // Redo replays forward to the exact post-stroke states.
        assert!(s.redo());
        assert_eq!(s.live, solid(8, 0xAA));
        assert!(s.redo());
        assert_eq!(s.live, solid(8, 0xBB));
        assert!(!s.c.can_redo());
    }

    #[test]
    fn new_stroke_clears_redo_branch() {
        let mut s = Sim::new(DEFAULT_MAX_DEPTH, solid(4, 0));
        s.paint(0, solid(4, 1));
        s.paint(1, solid(4, 2));
        s.undo();
        assert!(s.c.can_redo());
        // Painting a NEW stroke after an undo discards the redo branch.
        s.paint(2, solid(4, 3));
        assert!(!s.c.can_redo(), "new stroke must invalidate redo");
    }

    #[test]
    fn undo_250_times_does_not_corrupt() {
        // Spec criterion #3: 250 undos without corruption. Each stroke paints a
        // distinct value; undoing all the way back must reach the initial state
        // byte-for-byte with no index drift / aliasing.
        let mut s = Sim::new(512, solid(64, 0));
        for seq in 0..250u64 {
            s.paint(seq, solid(64, ((seq + 1) % 251) as u8));
        }
        let mut undone = 0usize;
        while s.undo() {
            undone += 1;
            assert_eq!(s.live.len(), 64, "size never corrupts");
        }
        assert_eq!(undone, 250, "all 250 strokes undoable");
        assert_eq!(s.live, solid(64, 0), "fully undone to the initial state");
        // And redo all the way forward reaches the final state exactly.
        while s.redo() {}
        assert_eq!(s.live, solid(64, 250u8));
    }

    #[test]
    fn depth_cap_drops_oldest() {
        // With a tiny cap, the oldest entries fall off (that depth becomes
        // non-undoable, like a ring history) but recent ones stay exact.
        let mut s = Sim::new(4, solid(4, 0));
        for seq in 0..10u64 {
            s.paint(seq, solid(4, (seq + 1) as u8));
        }
        assert!(
            s.c.undo_depth() <= 4,
            "cap enforced; got {}",
            s.c.undo_depth()
        );
        // The most recent stroke (9→10) must still undo exactly.
        assert!(s.undo());
        assert_eq!(s.live, solid(4, 9));
    }

    #[test]
    fn clear_drops_both_stacks() {
        let mut s = Sim::new(DEFAULT_MAX_DEPTH, solid(4, 0));
        s.paint(0, solid(4, 1));
        s.undo();
        s.c.clear();
        assert!(!s.c.can_undo() && !s.c.can_redo());
    }
}

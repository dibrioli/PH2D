//! **Unified shape+paint undo** — the glue that folds on-canvas shape authoring (Curve / Ellipse /
//! Polygon: create, point-edit, reshape, Offset drag) into the SAME undo timeline as the pixel bakes
//! (Apply / Apply & Keep), so Ctrl+Z walks every step in reverse chronological order regardless of kind
//! (Enio 2026-06-28). Each shape gesture brackets its mutation with [`PainterTool::begin_shape_txn`] /
//! [`PainterTool::commit_shape_txn`], recording ONE [`crate::undo::ModelSnapshot`] entry that carries the
//! editor state ([`crate::undo::ShapeEditState`]) alongside the pixels — so a structural undo/redo
//! reinstates the live overlay together with the canvas and the two can never desync. Replaces the former
//! per-session `curve_undo` stack (which could not interleave with the layer history).

use super::*;

impl PainterTool {
    /// `true` while any on-canvas shape (Curve / Free Hand / Ellipse / Polygon) is being authored — used to
    /// route pointer events + gate Enter/Esc (NOT to choose an undo stack; undo is one unified timeline).
    pub(crate) fn is_editing_shape(&self) -> bool {
        self.paint.curve.as_ref().is_some_and(|ed| ed.editing)
            || self.paint.ellipse.is_some()
            || self.paint.polygon.is_some()
            || self.paint.line.is_some()
    }

    /// Capture the open shape editor as plain undo data, or `None` when none is open.
    pub(crate) fn capture_shape(&self) -> Option<Box<crate::undo::ShapeEditState>> {
        use crate::undo::ShapeEditState as S;
        if let Some(ed) = self.paint.curve.as_ref() {
            Some(Box::new(S::Curve(ed.to_state())))
        } else if let Some(ed) = self.paint.ellipse.as_ref() {
            Some(Box::new(S::Ellipse(ed.to_state())))
        } else if let Some(ed) = self.paint.polygon.as_ref() {
            Some(Box::new(S::Polygon(ed.to_state())))
        } else {
            self.paint
                .line
                .as_ref()
                .map(|ed| Box::new(S::Line(ed.to_state())))
        }
    }

    /// Like [`Self::snapshot_model`] but also captures the open shape editor + its live drag-preview, so a
    /// restore reinstates the on-canvas overlay together with the pixels.
    pub(crate) fn capture_shape_model(&self) -> crate::undo::ModelSnapshot {
        let mut m = self.snapshot_model();
        m.shape = self.capture_shape();
        m.preview_patch = self
            .paint
            .drag_preview
            .as_ref()
            .map(|p| crate::undo::PreviewPatch {
                x: p.rect.x,
                y: p.rect.y,
                w: p.rect.w,
                h: p.rect.h,
                pixels: p.pixels.clone(),
            });
        m
    }

    /// Reinstate (or clear) the open shape overlay from a restored snapshot: peel the snapshot canvas back
    /// to its pristine baseline (strip the live-preview patch), rebuild the matching editor, route the tool
    /// to it, and re-stamp its geometry so dots + pixels stay in sync. A snapshot taken with NO live preview
    /// (`patch == None`, e.g. an Apply & Keep result) skips the re-stamp — the pixels already show the final
    /// shape — so the overlay sits on the baked canvas without doubling it.
    pub(crate) fn restore_shape_overlay(
        &mut self,
        shape: Option<Box<crate::undo::ShapeEditState>>,
        patch: Option<crate::undo::PreviewPatch>,
    ) {
        let had_preview = patch.is_some();
        if let Some(p) = patch {
            let rect = Region {
                x: p.x,
                y: p.y,
                w: p.w,
                h: p.h,
            };
            self.restore_region(&rect, &p.pixels);
        }
        self.paint.drag_preview = None;
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        match shape.map(|b| *b) {
            Some(crate::undo::ShapeEditState::Curve(s)) => {
                self.paint.curve = Some(curve::CurveEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Curve;
                if had_preview {
                    self.curve_refill();
                }
            }
            Some(crate::undo::ShapeEditState::Ellipse(s)) => {
                self.paint.ellipse = Some(ellipse::EllipseEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Ellipse;
                if had_preview {
                    self.ellipse_refill();
                }
            }
            Some(crate::undo::ShapeEditState::Polygon(s)) => {
                self.paint.polygon = Some(polygon::PolygonEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Polygon;
                if had_preview {
                    self.polygon_refill();
                }
            }
            Some(crate::undo::ShapeEditState::Line(s)) => {
                self.paint.line = Some(line::LineEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Line;
                if had_preview {
                    self.line_refill();
                }
            }
            None => {}
        }
    }

    /// Open a per-gesture undo transaction: flush any still-open one, then snapshot the current shape +
    /// pixels as the `before`. Each shape gesture (create / point-edit / reshape) brackets its mutation with
    /// this and [`Self::commit_shape_txn`], so it becomes ONE entry on the shared undo timeline.
    pub(crate) fn begin_shape_txn(&mut self) {
        self.flush_shape_txn();
        self.paint.stroke_undo = Some(self.capture_shape_model());
    }

    /// Close a per-gesture transaction: record `before → now` iff the gesture changed the shape geometry or
    /// the Offset (a bare select / no-op click is dropped). Clears the redo branch via `record_structural`.
    pub(crate) fn commit_shape_txn(&mut self) {
        let Some(before) = self.paint.stroke_undo.take() else {
            return;
        };
        let after = self.capture_shape_model();
        if shape_model_changed(&before, &after) {
            self.undo.record_structural(before, after);
        }
    }

    /// Open a COALESCED transaction for an Offset-slider drag: a contiguous run of slider values from the
    /// same baseline collapses into one undo entry (snapshot only when none is open); the next gesture /
    /// bake / undo flushes it.
    pub(crate) fn begin_offset_txn(&mut self) {
        if self.paint.stroke_undo.is_none() {
            self.paint.stroke_undo = Some(self.capture_shape_model());
        }
    }

    /// Flush any open per-gesture / Offset transaction into the undo stack — called before a new gesture, a
    /// bake/commit, or an undo/redo so a coalesced Offset drag is closed as its own entry first.
    pub(crate) fn flush_shape_txn(&mut self) {
        self.commit_shape_txn();
    }

    /// `true` while a shape transaction is open (a creation/edit gesture in progress or a coalesced Offset
    /// drag) — an undoable action the affordance must reflect even before it's flushed to the stack.
    pub(crate) fn has_pending_shape_txn(&self) -> bool {
        self.paint.stroke_undo.is_some()
    }

    /// The **Offset** slider track — the cross-module accessor for the undo glue (the field is private to
    /// the `paint` module).
    pub(crate) fn shape_offset_norm(&self) -> f32 {
        self.paint.shape_offset_norm
    }

    /// Set the **Offset** slider track (used by `restore_model` to reinstate a snapshot's Offset).
    pub(crate) fn set_shape_offset_norm(&mut self, norm: f32) {
        self.paint.shape_offset_norm = norm;
    }

    /// The **accumulated** Offset (px) from prior Apply & Keep presses — the cross-module accessor for the
    /// undo glue (the field is private to the `paint` module).
    pub(crate) fn shape_offset_base_px(&self) -> f32 {
        self.paint.shape_offset_base_px
    }

    /// Set the accumulated Offset (used by `restore_model` to reinstate a snapshot's accumulated Offset).
    pub(crate) fn set_shape_offset_base_px(&mut self, px: f32) {
        self.paint.shape_offset_base_px = px;
    }
}

/// Whether a shape transaction's `before → after` is a real undo step: an Offset change, a geometry change
/// (selection-agnostic), or a create/apply (the shape appearing or vanishing). A pure point-selection is
/// NOT a step.
fn shape_model_changed(a: &crate::undo::ModelSnapshot, b: &crate::undo::ModelSnapshot) -> bool {
    if a.offset_norm != b.offset_norm {
        return true;
    }
    match (&a.shape, &b.shape) {
        (None, None) => false,
        (Some(x), Some(y)) => !x.geom_eq(y),
        _ => true, // None ↔ Some = the shape was created or applied/cancelled
    }
}

//! Interactive **drag-preview** stamping — the restore-then-re-stamp path that lets a moving/resizing/
//! growing preview (Drag Dot / Anchored / Line, and every shape editor) leave no trail, plus the dirty-rect
//! bookkeeping. Split from `paint.rs` for the workspace file-LOC cap; a child module of `paint`, so it keeps
//! access to `PaintState`'s module-private fields and the private `DragPreview` / `union_region` helpers.

use super::*;

impl PainterTool {
    /// Flag `rect` dirty for the next GPU preview upload + bump the active layer's pixel epoch.
    pub(super) fn mark_dirty(&mut self, rect: Region) {
        self.dirty_rect = Some(self.dirty_rect.map_or(rect, |acc| union_region(acc, rect)));
        self.preview_dirty = true;
        self.edited_since_bind = true; // unbaked work — the shell auto-persists on leave/deactivate
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// Stamp an interactive preview batch (Drag Dot / Anchored = 1 dab, Line = N): restore the
    /// previous footprint's saved pixels, then save the pristine pixels under the new dabs' UNION
    /// bbox and stamp there — so the moving preview leaves no trail. Pen-up: `commit_drag_preview`.
    ///
    /// The stamp goes through the full [`Self::stamp_dabs`] dispatcher (NOT the bare brush route), so a
    /// **Composite Brush** runs all three layers here too. `stamp_dabs` tiles internally (so it takes
    /// the UNtiled dabs); the save-region bbox is measured over the tiled set so it still covers the
    /// wrapped copies (else the wrapped paint falls outside the restore region — a trail).
    pub(super) fn stamp_drag_preview(&mut self, dabs: &[Dab]) {
        // In Selection **Edit** mode the native gizmo drives the SELECTION mask, not pixels — peel any
        // leftover preview and paint nothing (ADR-0103 Am.2). The mask refill runs off the pointer path.
        if self.paint.selection_edit_mode {
            if let Some(prev) = self.paint.drag_preview.take() {
                self.restore_region(&prev.rect, &prev.pixels);
            }
            return;
        }
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        // Coverage bbox over the wrapped Tiling copies (the stamp re-tiles them itself).
        let coverage_storage;
        let coverage: &[Dab] = if self.paint.tiling[0] || self.paint.tiling[1] {
            coverage_storage = tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            &coverage_storage
        } else {
            dabs
        };
        let bbox = coverage.iter().fold(None, |acc, d| {
            match (acc, self.dab_bbox(d.center, d.radius_px)) {
                (Some(a), Some(r)) => Some(union_region(a, r)),
                (a, r) => a.or(r),
            }
        });
        // Each preview frame re-stamps the WHOLE current batch onto the restored (pristine) canvas, so
        // a Composite Brush's Smear layer must chain fresh within THIS batch — clear the cross-batch
        // source (a Line's dabs then smear from the anchor; a single Drag-Dot dab simply has no source).
        self.paint.last_smear_pos = None;
        match bbox {
            Some(rect) => {
                let pixels = self.save_region(&rect);
                self.stamp_dabs(dabs);
                self.paint.drag_preview = Some(DragPreview { rect, pixels });
            }
            None => self.stamp_dabs(dabs),
        }
    }

    /// Commit the interactive preview: drop the restore record so the last batch stays painted.
    /// Safe to call for any method (a no-op unless a preview is live).
    pub(super) fn commit_drag_preview(&mut self) {
        self.paint.drag_preview = None;
    }

    /// Stamp the dabs a `begin`/`extend` produced. Drag Dot, Anchored AND Line are interactive
    /// preview methods: route their batch through the restore+re-stamp path so the moving/resizing/
    /// growing preview leaves no trail and `commit_drag_preview` keeps the last on pen-up. Every other
    /// method uses the cumulative stamp.
    pub(super) fn stamp_stroke_dabs(&mut self, dabs: &[Dab]) {
        if matches!(
            self.paint.brush.stroke_method,
            StrokeMethod::DragDot | StrokeMethod::Anchored | StrokeMethod::Line
        ) {
            self.stamp_drag_preview(dabs);
        } else {
            self.stamp_dabs(dabs);
        }
    }
}

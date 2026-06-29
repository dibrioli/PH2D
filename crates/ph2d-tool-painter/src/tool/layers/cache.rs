//! Composite-CACHE invalidation — the single structural chokepoint that drops
//! the CPU composite + dirty-rect + adjustment cut-cache and bumps the publish
//! revision. `impl PainterTool` (one of several blocks in this crate). Split out
//! of the former `tool/layers.rs` god-file (pure move).

use super::super::*;

impl PainterTool {
    /// Mark the composite stale so the next `current_preview` recomputes.
    pub(crate) fn invalidate_composite(&mut self) {
        self.composited = None;
        // Drop any accumulated dirty-rect: a structural edit (opacity / blend /
        // visibility / reorder / add / select) can change the composite OUTSIDE
        // the stamped region, so the next drain must do a FULL recompose.
        self.dirty_rect = None;
        self.preview_dirty = true;
        self.edited_since_bind = true; // structural/metadata edit → unbaked composite change
        // W5: a structural edit (add/remove/reorder/visibility/opacity/blend/
        // select) changes the composite below some adjustment → every cut is
        // potentially stale. Conservative-correct: drop them all (they cold-
        // refill on the next slider-drag). The id is ignored by `invalidate_from`.
        self.compositor_cache
            .invalidate_from(RtLayerId(0), &self.layers);
        // B.5: every structural/metadata edit funnels through here, so this is
        // the single chokepoint that bumps the publish revision (set_source is
        // the only structural reset that bypasses it — bumped there too).
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Force the NEXT preview drain to FULLY recompose + FULLY upload (drop the dirty-rect fast lane), so
    /// a new SHAPE session (Curve / Free Hand / Circle / Polygon "no session yet" creation block) starts
    /// from a byte-correct preview base instead of patching a possibly-stale `composited` cache — the rare
    /// early-session artifact where a sliver of the shape appears then vanishes, only on the first few uses
    /// (`HANDOFF_per_layer_color_perf_artifacts` §1.R FOLLOW-UP). Unlike [`Self::invalidate_composite`] it
    /// does NOT flag an edit (`edited_since_bind`) nor drop the adjustment cut-caches (no pixels changed,
    /// it only re-seeds the preview). Cheap: one full recompose + upload, once per shape-session start.
    pub(crate) fn reseed_preview_base(&mut self) {
        self.composited = None; // → `take_preview_arc` takes the full-recompose branch next drain
        self.dirty_rect = None; // → `preview_upload_bbox = None` → the bridge does a FULL texture upload
        self.preview_dirty = true; // ensure that drain runs even though no pixels changed yet
    }
}

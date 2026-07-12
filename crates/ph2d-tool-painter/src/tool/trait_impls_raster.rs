//! `impl RasterEditTool for PainterTool` — the shell push / composite-preview / Apply-bake interface.
//! Split from `trait_impls` for the workspace file-LOC cap.

use super::*;

impl RasterEditTool for PainterTool {
    /// Initialize the working canvas from the sprite source. RGBA8 straight; it
    /// is the base of the single-raster layer stack the layers panel grows from.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        assert_eq!(
            rgba.len(),
            (width as usize) * (height as usize) * 4,
            "set_source rgba length must equal width*height*4"
        );
        // A new working canvas invalidates every in-progress edit whose state points at the OLD buffer
        // (open shape, pending Fill ColorDrop, Mask scratch, Drag-Dot restore, armed Eyedropper); abandon
        // them so nothing floods / corrupts the freshly-bound sprite. See `paint::lifecycle`.
        self.reset_transient_edit_state();
        self.canvas_rgba = Arc::new(rgba);
        self.source_size = (width, height);
        self.preview_dirty = true;
        self.edited_since_bind = false; // fresh bind = no unbaked edits (canvas mirrors the sprite)
        // A fresh source = a fresh single-raster stack whose lone layer's pixels ARE `canvas_rgba` (trivial).
        self.layers = LayerStack::new();
        self.layers.add_raster("Layer 1", width, height);
        self.images.clear();
        self.layer_pixel_versions.clear();
        let active = self.layers.active();
        self.bump_layer_pixels(active);
        self.composited = None;
        // The compositor's cut-point cache is DOCUMENT-scoped: each entry is the accumulator BELOW an
        // Adjustment layer, a buffer sized for THAT canvas — and `LayerStack::new()` above restarts
        // `next_id` at 1, so the fresh stack's ids COLLIDE with the old document's by construction. The
        // cache's guard only asks "is there a cut for this id?", never "does it have the shape of THIS
        // canvas?", so a stale cut got reused: a bigger sprite indexes past the old accumulator (panic),
        // and a SAME-SIZE sprite composites its adjustment over the OLD sprite's layers-below — wrong, and
        // silent. `restore_doc` (the sibling rebind) has always cleared these four; `set_source` never did.
        // Sweep 2026-07-12, `BUGS_painter.md` #15 — same root as Bug #12: a reuse guard that asks
        // "initialised?" instead of "does the SHAPE match?".
        self.compositor_cache = CompositorCache::new();
        self.adjustment_cache_pending = false;
        self.dirty_rect = None;
        self.preview_upload_bbox = None;
        self.layers_revision = self.layers_revision.wrapping_add(1);
        // A different working canvas — undo/redo over the OLD model is meaningless on the NEW one.
        self.undo.clear();
    }

    /// Borrow the current composite iff it changed (drains `preview_dirty`); trivial → `canvas_rgba`.
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        // A mask row's grayscale-VIEW eye open shows that mask's grayscale; else the trivial fast path.
        let gray = self.mask_grayscale_view_pixels();
        if gray.is_none() && self.is_trivial_stack() && !self.mask_scratch_active() {
            return Some((&self.canvas_rgba, w, h));
        }
        let composited = if let Some(g) = gray {
            g
        } else {
            // Mask brush: the active layer composites NORMALLY (fully visible) — the protection scratch
            // never hides it; the overlay below only tints the frozen region.
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            let mut c = composite(&self.layers, &src, w, h);
            self.apply_mask_overlay(&mut c); // tint the protected region (no-op without a scratch)
            c
        };
        self.composited = Some(Arc::new(composited));
        let px = self
            .composited
            .as_ref()
            .map(|a| a.as_slice())
            .unwrap_or(&[]);
        Some((px, w, h))
    }

    fn take_pending_commit(&mut self) -> bool {
        std::mem::take(&mut self.pending_commit)
    }

    /// Bake the final canvas for commit (Apply): the full layer COMPOSITE (base + every layer's
    /// opacity / blend / visibility / adjustments) — exactly what the live preview shows.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let (w, h) = self.source_size;
        if !self.is_trivial_stack() {
            let active = self.layers.active().unwrap_or(RtLayerId(0));
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            return (composite(&self.layers, &src, w, h), w, h);
        }
        // Trivial single-layer fast path: take the Arc out so refcount==1 →
        // `unwrap_or_clone` returns the inner Vec without allocation.
        let canvas = std::mem::replace(&mut self.canvas_rgba, Arc::new(Vec::new()));
        (Arc::unwrap_or_clone(canvas), w, h)
    }

    fn deactivate(&mut self) {
        self.canvas_rgba = Arc::new(Vec::new());
        self.source_size = (0, 0);
        self.preview_dirty = false;
        self.pending_commit = false;
        self.edited_since_bind = false;
        self.deferred_bake = false;
        self.dock_shows_layers = false; // re-activating starts on the Brush view (Enio 2026-07-04)
    }
}

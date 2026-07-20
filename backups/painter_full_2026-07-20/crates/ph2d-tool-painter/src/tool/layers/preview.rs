//! GPU PREVIEW plumbing — per-layer pixel content-version bumps (the compositor
//! cache key), the live pixel borrow the shell bridge adapts to
//! `LayerPixelProvider`, the preview-dirty drain, and the trivial-stack fast
//! path. `impl PainterTool` (one of several blocks in this crate). Split out of
//! the former `tool/layers.rs` god-file (pure move).

use super::super::*;

impl PainterTool {
    /// Bump `id`'s pixel CONTENT version (the GPU preview compositor's cache
    /// key — see [`Self::layer_pixel_versions`]). Call from every canvas
    /// pixel-write chokepoint (stroke stamp, undo/redo, mask flatten, fresh
    /// source). No-op when `id` is `None` (degenerate / empty stack).
    pub(crate) fn bump_layer_pixels(&mut self, id: Option<RtLayerId>) {
        if let Some(id) = id {
            self.pixel_clock = self.pixel_clock.wrapping_add(1);
            self.layer_pixel_versions.insert(id, self.pixel_clock);
        }
    }

    /// Bump the pixel-content version of EVERY current layer — used after a
    /// structural undo/redo reinstalls a whole model snapshot, so the GPU
    /// compositor's per-slice cache (keyed by content version) re-uploads each
    /// layer rather than serving a stale slice from a prior identity.
    pub(crate) fn bump_all_layer_pixels(&mut self) {
        let ids: Vec<RtLayerId> = self.layers.all_ids().collect();
        for id in ids {
            self.bump_layer_pixels(Some(id));
        }
    }

    /// Borrow one layer's straight-sRGB8 pixels + its content version for the
    /// GPU preview compositor (the shell bridge adapts this to
    /// `ph2d_render::LayerPixelProvider`; the tool stays decoupled from the
    /// render crate). The ACTIVE layer reads the live `canvas_rgba` working
    /// buffer (always current — strokes mutate it in place); every other layer
    /// reads its `images` entry. `version` changes iff the layer's PIXELS
    /// changed (see [`Self::layer_pixel_versions`]), so the compositor
    /// re-uploads a slice only on a real pixel edit. `None` for an unknown key
    /// or an empty active buffer (the bridge then falls back to the CPU
    /// compositor). W3 GPU preview (ADR-0045 Phase 3, step 2).
    #[must_use]
    pub fn preview_layer_pixels(&self, key: u64) -> Option<(u64, &[u8])> {
        let id = RtLayerId(key);
        let version = self.layer_pixel_versions.get(&id).copied().unwrap_or(0);
        if self.layers.active() == Some(id) {
            let buf = self.canvas_rgba.as_ref().as_slice();
            if buf.is_empty() {
                return None;
            }
            Some((version, buf))
        } else {
            self.images
                .get(&id)
                .map(|img| (version, img.rgba8.as_slice()))
        }
    }

    /// Drain the `preview_dirty` flag WITHOUT compositing — the GPU preview
    /// path's equivalent of the dirty gate inside [`Self::take_preview_arc`],
    /// for when the shell composites on the GPU instead of the CPU. Returns
    /// `true` iff the preview changed since the last drain (so the bridge
    /// recomposites this frame). Mirrors `take_preview_arc`'s empty-canvas
    /// guard so an un-sourced tool reports clean.
    ///
    /// **A drain that bypasses the CPU composite leaves the CPU composite stale by definition** —
    /// the change that raised the flag was never folded into `composited`, and the dirty-rect that
    /// described it is about to be thrown away by the GPU lane. So a `true` drain drops both: the
    /// next `take_preview_arc` (a GPU→CPU producer handoff — e.g. the first impasto/sculpt dab on
    /// a GPU-composited stack) then does a FULL recompose + full upload instead of blitting the
    /// new rect into a cache whose *other* pixels predate every GPU-owned edit. Before this, the
    /// handoff frame could show a mix of eras: fresh paint inside the last dirty rect, pre-GPU
    /// pixels everywhere else (display gate `the_screen_survives_the_gpu_to_cpu_producer_handoff`).
    #[must_use]
    pub fn take_preview_dirty(&mut self) -> bool {
        if self.canvas_rgba.is_empty() {
            return false;
        }
        let dirty = std::mem::take(&mut self.preview_dirty);
        if dirty {
            self.composited = None;
            self.dirty_rect = None;
        }
        dirty
    }

    /// Public projection of [`Self::is_trivial_stack`] for the shell's GPU
    /// preview producer (it bows out of GPU compositing on a trivial stack so
    /// the zero-copy CPU fast path owns the slot).
    #[must_use]
    pub fn preview_is_trivial_stack(&self) -> bool {
        self.is_trivial_stack()
    }

    /// `true` when the stack is a single visible, opaque, Normal raster with
    /// no mask/clip — i.e. the composite is byte-identical to `canvas_rgba`,
    /// so `current_preview` skips compositing entirely (the fast path).
    pub(crate) fn is_trivial_stack(&self) -> bool {
        let root = self.layers.root();
        if root.len() != 1 {
            return false;
        }
        match self.layers.get(root[0]) {
            Some(l) => {
                matches!(l.kind, LayerKind::Raster(_))
                    && l.visible
                    && l.opacity >= 1.0
                    && l.blend_mode == BlendMode::Normal
                    && l.mask.is_none()
                    && !l.clipping
            }
            None => true, // empty/degenerate → nothing to composite
        }
    }
}

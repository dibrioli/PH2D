//! Reusable scratch buffers shared by the bgremoval pipeline.
//!
//! Buffers grow lazily as inputs scale; they never shrink. The
//! [`BgRemovalTool`](super::tool::BgRemovalTool) owns one instance
//! so per-frame thumbnail re-runs are zero-allocation after the
//! first call (HR-3). Same pattern as `GridSnapState::scratch`.

/// One span on a scanline flood-fill stack.
///
/// `parent_dir` from the classical Heckbert variant is intentionally
/// dropped: we use BFS-on-spans (push neighbours unconditionally,
/// rely on `mask != 0` to skip already-visited pixels). The stack
/// invariant is "any unprocessed run-of-bg-pixels" rather than
/// "a span plus the direction it grew from".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FloodSpan {
    pub y: u32,
    pub x_left: u32,
    pub x_right: u32,
}

/// Reusable scratch buffers for one full pipeline run.
///
/// Sizing convention: `ensure(w, h, color_guide)` grows every buffer
/// to fit a `w × h` image. Subsequent calls with smaller dimensions
/// reuse the existing capacity (no shrink). All fields are public so
/// the algorithm modules can write into them without cell-borrow
/// gymnastics; callers must respect the layout invariants in the
/// per-field docs.
#[derive(Clone, Debug, Default)]
pub struct BgRemovalScratch {
    /// Per-pixel Oklab color distance (ΔE) to the detected background
    /// reference. Size: `w*h`. Owned by `algorithm::chroma`. Read by
    /// `algorithm::compose` for soft-band synthesis.
    pub delta_e: Vec<f32>,

    /// Segmentation mask: 0 = background, 255 = foreground. Size:
    /// `w*h`. Owned by `algorithm::chroma` / `algorithm::grabcut`.
    /// Read by `algorithm::guided_filter::refine`.
    pub mask: Vec<u8>,

    /// Scanline flood-fill stack. Pre-grown to capacity `4*W` on
    /// first use (loose upper bound; practical span counts are much
    /// lower). Owned by `algorithm::chroma::flood_from_borders`.
    pub spans: Vec<FloodSpan>,

    /// Oklab samples extracted from corner patches in
    /// `algorithm::chroma::corner_samples`. Capacity grown to
    /// `CORNER_PATCH_TOTAL` once and reused — the Vec is `.clear()`-ed
    /// (length 0) between calls and re-filled, so the same backing
    /// allocation persists across preview ticks (HR-3).
    pub corner_samples: Vec<[f32; 3]>,

    /// K-means cluster assignment per `corner_samples` entry: 0 or 1.
    /// Same lifecycle as `corner_samples`.
    pub corner_assignments: Vec<u8>,

    /// Soft-alpha output in `[0,1]`. Size: `w*h`. Written by
    /// `algorithm::guided_filter::refine`. Read by `algorithm::compose`.
    pub alpha_f32: Vec<f32>,

    /// Float guide buffer for the guided filter: linearized sRGB
    /// luma (size `w*h`) when `color_guide = false`, else RGB packed
    /// `[R₀,G₀,B₀,R₁,G₁,B₁,…]` (size `w*h*3`).
    pub guide_f32: Vec<f32>,

    /// Box-filter intermediate buffer A. Size: `w*h`. Reused across
    /// the 6-9 box-filter passes inside guided_filter.
    pub box_buf_a: Vec<f32>,

    /// Box-filter intermediate buffer B. Size: `w*h`. Used together
    /// with `box_buf_a` for `var = corr - mean²` style ops.
    pub box_buf_b: Vec<f32>,

    /// Final pipeline output, RGBA8. Size: `w*h*4`. Written by
    /// `algorithm::compose::write_output`. This is what the host /
    /// preview thumbnail consumes.
    pub output_rgba: Vec<u8>,
}

impl BgRemovalScratch {
    /// Grow every buffer to fit a `w × h` image. Idempotent. Never
    /// shrinks (HR-3: capacity only grows so subsequent calls reuse
    /// allocations).
    pub fn ensure(&mut self, w: u32, h: u32, color_guide: bool) {
        let n = (w as usize) * (h as usize);
        let guide_n = if color_guide { n * 3 } else { n };

        // resize_with(0_default) is fine — old contents do not need to
        // survive across pipeline runs; each step writes every pixel
        // it cares about.
        self.delta_e.resize(n, 0.0);
        self.mask.resize(n, 0);
        self.alpha_f32.resize(n, 0.0);
        self.guide_f32.resize(guide_n, 0.0);
        self.box_buf_a.resize(n, 0.0);
        self.box_buf_b.resize(n, 0.0);
        self.output_rgba.resize(n * 4, 0);

        let span_cap = (w as usize).saturating_mul(4);
        if self.spans.capacity() < span_cap {
            let need = span_cap - self.spans.capacity();
            self.spans.reserve(need);
        }
        self.spans.clear();

        // Corner sample buffers are length-0 between calls; the
        // chroma segmenter `.clear()` + repush fills them. Reserve
        // once so subsequent ticks reuse the allocation.
        const CORNER_BUDGET: usize = 4096;
        if self.corner_samples.capacity() < CORNER_BUDGET {
            self.corner_samples
                .reserve(CORNER_BUDGET - self.corner_samples.capacity());
        }
        if self.corner_assignments.capacity() < CORNER_BUDGET {
            self.corner_assignments
                .reserve(CORNER_BUDGET - self.corner_assignments.capacity());
        }
        self.corner_samples.clear();
        self.corner_assignments.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_grows_buffers_to_dimensions() {
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 32, /* color_guide = */ false);
        assert_eq!(s.delta_e.len(), 64 * 32);
        assert_eq!(s.mask.len(), 64 * 32);
        assert_eq!(s.alpha_f32.len(), 64 * 32);
        assert_eq!(s.guide_f32.len(), 64 * 32);
        assert_eq!(s.output_rgba.len(), 64 * 32 * 4);
        assert!(s.spans.capacity() >= 64 * 4);
        assert!(s.corner_samples.capacity() >= 4096);
        assert!(s.corner_assignments.capacity() >= 4096);
    }

    #[test]
    fn ensure_color_guide_triples_guide_buffer() {
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 32, /* color_guide = */ true);
        assert_eq!(s.guide_f32.len(), 64 * 32 * 3);
    }

    #[test]
    fn ensure_does_not_shrink_capacity_after_smaller_call() {
        let mut s = BgRemovalScratch::default();
        s.ensure(256, 256, false);
        let cap_after_big = s.delta_e.capacity();
        s.ensure(64, 64, false);
        // resize(64*64) leaves capacity intact (Vec never shrinks
        // on resize-smaller).
        assert!(s.delta_e.capacity() >= cap_after_big);
        // Length follows the new request, though.
        assert_eq!(s.delta_e.len(), 64 * 64);
    }

    #[test]
    fn flood_span_layout_is_compact() {
        // Smoke-check: 3 u32 fields, expected to pack to 12 bytes.
        // (Drives away accidentally adding a non-Copy field later.)
        assert_eq!(std::mem::size_of::<FloodSpan>(), 12);
    }
}

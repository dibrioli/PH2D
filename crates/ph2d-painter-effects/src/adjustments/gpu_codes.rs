//! **Which GPU path an adjustment kind takes** — the tool↔shader routing contract.
//!
//! Extracted from [`super`] (workspace file-LOC cap) and the split is by
//! responsibility, not by size: these three functions are the only ones a
//! compositor asks, and `shells/desktop/render_loop/painter_gpu_flatten` reads
//! all three to decide GPU-vs-CPU for a whole document. That decision is worth
//! up to **885×** (`docs/Painter/25_avaliacao_gpu.md`), which is why they now sit
//! together with their memberships pinned by
//! `the_gpu_code_sets_are_what_the_routing_believes_they_are`.

use super::AdjustmentKind;

impl AdjustmentKind {
    /// GPU adjustment-kernel code for the real-time compositor
    /// (`ph2d-render::layer_composite.wgsl` `ADJ_*` / `apply_adjustment`), or
    /// `None` for a kind the GPU shader does not implement yet (the compositor
    /// falls back to the CPU path for those). This is the tool↔shader contract —
    /// the painter flatten emits `LayerOp::Adjustment { kind: gpu_code(), .. }`.
    /// Keep in lock-step with the WGSL `ADJ_*` consts + the GPU parity gate
    /// `gpu_adjustment_matches_cpu_reference_each_kind`.
    #[must_use]
    pub fn gpu_code(self) -> Option<u8> {
        Some(match self {
            Self::HueSaturationBrightness => 0,
            Self::BrightnessContrast => 1,
            Self::Invert => 2,
            Self::Posterize => 3,
            Self::Threshold => 4,
            Self::Exposure => 5,
            Self::Vibrance => 6,
            // W4 bespoke — display-space 1-D transfer LUTs uploaded to the
            // compositor's binding-6 `adj_luts` (Curves = 3×256, Levels = 1×256).
            Self::Curves => 7,
            Self::Levels => 8,
            // Coordinate-dependent per-pixel kinds (read the absolute canvas pixel
            // in `apply_adjustment`) — dirty-rect exact.
            Self::Noise => 9,
            Self::Halftone => 10,
            Self::ColorLookupLut => 11,
            // Not yet ported to the per-pixel GPU shader. Spatial kinds run on
            // the multi-pass pass-graph instead — see `gpu_spatial_code`.
            _ => return None,
        })
    }

    /// GPU **spatial** kernel code for the multi-pass pass-graph
    /// (`ph2d-render::LayerCompositor` `SpatialAdjustment` path), or `None` for a
    /// kind that is not a (ported) spatial/neighbourhood op. These are the kinds
    /// `gpu_code` returns `None` for AND the compositor can run as a separable /
    /// gather pass: the painter flatten emits `LayerOp::SpatialAdjustment { kernel:
    /// gpu_spatial_code(), .. }` for them (vs. the scalar `LayerOp::Adjustment` for
    /// `gpu_code` kinds, vs. the CPU fallback for kinds with neither).
    ///
    /// The codes MIRROR `ph2d_render::layer_compositor::SPATIAL_*` (kept in
    /// lock-step the same way `gpu_code` mirrors the WGSL `ADJ_*` consts — no
    /// `ph2d-render` dependency here, that would be a cycle). Reconciled with the
    /// pass-graph by the spatial parity gates (`gpu_<kind>_matches_cpu_reference`).
    ///
    /// ⚠️ This doc used to say *"`Bloom` / `ShadowsHighlights` … stay `None` (CPU
    /// fallback) until their kernel ships"* — and the `match` three lines below has
    /// been returning `Some(4)` / `Some(5)` for them since their kernels DID ship
    /// (`SPATIAL_BLOOM` / `SPATIAL_SHADOWS_HIGHLIGHTS`, exercised by the
    /// `gpu_bloom_drag_perf` gate). The comment contradicted the code it introduced,
    /// and a stale refusal list here is not cosmetic: the painter's flatten reads
    /// these two functions to decide GPU-vs-CPU, and that decision is worth up to
    /// 885× (`docs/Painter/25_avaliacao_gpu.md`). Read the `match`, not this prose —
    /// and if you change the `match`, change this.
    #[must_use]
    pub fn gpu_spatial_code(self) -> Option<u8> {
        Some(match self {
            Self::GaussianBlur => 0,        // SPATIAL_GAUSSIAN
            Self::Sharpen => 1,             // SPATIAL_SHARPEN
            Self::MotionBlur => 2,          // SPATIAL_MOTION
            Self::ChromaticAberration => 3, // SPATIAL_CHROMA
            Self::Bloom => 4,               // SPATIAL_BLOOM (bright-pass→blur→add)
            Self::ShadowsHighlights => 5,   // SPATIAL_SHADOWS_HIGHLIGHTS (luma blur + tonal)
            _ => return None,
        })
    }

    /// Does this kind's compute CHANGE COVERAGE (feather / extend alpha), so the
    /// compositor combine must ADOPT the kernel's output alpha instead of
    /// preserving the base coverage? True for the blur-family (their premultiplied
    /// blur spreads alpha — soft edges into transparency) + `Bloom` (its glow
    /// haloes outward). False for tonal / per-pixel kinds — including
    /// `ShadowsHighlights`, which blurs only an INTERNAL luma map but outputs the
    /// base coverage. Drives the `is_spatial` branch in `compositor::compose`.
    ///
    /// ⚠️ This note used to read *"BROADER than [`Self::gpu_spatial_code`]: `Bloom`
    /// feathers coverage but has no GPU spatial kernel yet"* — **both halves are
    /// false today**. Bloom has kernel 4, and this set is a strict SUBSET of the
    /// spatial set: it is the five that spread alpha, while `ShadowsHighlights` is
    /// spatial (kernel 5) yet outputs the base coverage, so it is the one spatial
    /// kind deliberately absent here. The two sets still differ on purpose — just
    /// in the other direction.
    #[must_use]
    pub fn feathers_coverage(self) -> bool {
        matches!(
            self,
            Self::GaussianBlur
                | Self::Sharpen
                | Self::MotionBlur
                | Self::ChromaticAberration
                | Self::Bloom
        )
    }
}

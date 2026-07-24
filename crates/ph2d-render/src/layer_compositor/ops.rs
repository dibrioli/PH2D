//! **What a compositor op IS** — the op-list model and the pure questions asked
//! of it, split out of [`super`] (workspace file-LOC cap).
//!
//! The split is by responsibility: this file is the vocabulary the CALLER speaks
//! (`ph2d-tool-painter` flattens its `LayerStack` into these) plus the pure
//! functions of an op-list — validation, which keys it needs resident, and the
//! kernel weights a `SPATIAL_*` code implies. [`super`] is the engine that runs
//! them. Nothing here touches wgpu, so the whole op model — including the
//! coverage modifiers whose fold order has to mirror the CPU reference exactly —
//! is readable and testable without a device.

use super::*;

/// A grayscale mask attached to a [`LayerOp::Layer`] or [`LayerOp::Adjustment`].
///
/// `key` is a layer key like any other — the compositor resolves it to a cached
/// texture-array slice through the SAME provider, so a mask costs one slice and
/// nothing else. Its value is the **Rec.601 luma of the straight sRGB bytes**
/// (no transfer function): a mask is a coverage op, and coverage does not live
/// in a colour space (`ph2d_tool_painter::compositor::mask_value`).
///
/// ⚠️ A mask whose buffer the provider cannot serve at canvas size is treated as
/// **no mask**, not as an error — mirroring the CPU reference, which guards with
/// `mrgba.len() >= …` and falls through to "fully visible". Failing the whole
/// composite instead would hand the document to the other producer over a
/// degenerate buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LayerMask {
    pub key: u64,
    /// `true` = the mask reads `1 - luma` (the mask layer's own `inverted`).
    pub inverted: bool,
}

/// One flattened compositor op, emitted by the caller from its `LayerStack`
/// (top-down, bottom-to-top within each sibling list — the same order the CPU
/// `composite_into` recurses). `key` is the caller's stable layer identifier
/// (`LayerId.0`); `blend_mode` is the `BlendMode` wire `u8`; `opacity` folds
/// into the source alpha. Groups bracket their children with
/// [`LayerOp::PushGroup`] … [`LayerOp::PopGroup`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LayerOp {
    /// Blend a raster layer's pixels over the current accumulator.
    ///
    /// `mask` and `clipping` are the two **coverage modifiers** (Painter T3.5 /
    /// T3.6). They fold into the source alpha in a fixed order that mirrors the
    /// CPU reference exactly — decode → mask → clip → opacity — because each is
    /// a multiply and the CPU's `blend_window` applies `opacity` *after* the
    /// sample closure that applies the other two.
    Layer {
        key: u64,
        blend_mode: u8,
        opacity: f32,
        /// Optional grayscale mask layer whose Rec.601 luma multiplies this
        /// layer's straight alpha (`None` = fully visible).
        mask: Option<LayerMask>,
        /// Clip to the nearest NON-clipping layer below at this depth: multiply
        /// alpha by that layer's **raw** straight alpha — raw meaning before its
        /// own mask and before its own opacity, which is what the CPU's
        /// `clip_base = Some(rgba)` hands to the next layer.
        ///
        /// Consecutive clipping layers chain to the same base; a group or an
        /// adjustment breaks the chain; a clipping layer with no base below it
        /// draws unclipped.
        clipping: bool,
    },
    /// Begin a group: push a fresh sub-accumulator.
    PushGroup,
    /// End a group: blend the sub-accumulator over the parent as one layer.
    PopGroup { blend_mode: u8, opacity: f32 },
    /// Apply a non-destructive adjustment to the current accumulator (everything
    /// below it). `kind` is an `ADJ_*` code — the caller maps its
    /// `AdjustmentKind` to a code (the render crate stays decoupled from the
    /// painter tool); an unknown code is an identity no-op in the shader.
    /// `params` are the kind's ≤3 scalar params (see the WGSL `apply_adjustment`);
    /// `blend_mode`/`opacity` are the adjustment's own — the effect blends back
    /// over the base by these. W4 (ADR-0045).
    Adjustment {
        kind: u8,
        params: [f32; 3],
        blend_mode: u8,
        opacity: f32,
        /// Optional mask: its Rec.601 luma multiplies the adjustment's own
        /// opacity per pixel (white = full effect), which is where the CPU
        /// reference puts it too — masking the STRENGTH of the effect, never
        /// the coverage of the pixels below it.
        mask: Option<LayerMask>,
    },
    /// Apply a SPATIAL (neighbourhood) adjustment to the current accumulator
    /// (everything below it). Unlike [`LayerOp::Adjustment`] — which is a
    /// per-pixel transform foldable into the single-pass compositor — a spatial
    /// effect reads a *radius* of neighbours, so it is architecturally a **pass
    /// break**: the compositor materialises the below-composite into a texture,
    /// runs the kernel as 1+ ping-pong passes, blends the result back, then
    /// continues the layers above as a new segment (Painter W4 spatial infra).
    ///
    /// `kernel` is a `SPATIAL_*` code (the caller maps its `AdjustmentKind`);
    /// `params` are the kernel's scalars (`SPATIAL_GAUSSIAN` uses `params[0]` =
    /// radius; `SPATIAL_SHADOWS_HIGHLIGHTS` uses all 8). `blend_mode`/`opacity`
    /// are the adjustment's own — the effect blends back over the base by these,
    /// mirroring the `Adjustment` arm. An unknown `kernel` is an identity no-op
    /// (forward-compatible). The 8-scalar `params` is the widest spatial kind
    /// (S/H); the 4-scalar kinds zero-pad the tail.
    SpatialAdjustment {
        kernel: u8,
        params: [f32; 8],
        blend_mode: u8,
        opacity: f32,
    },
}

/// Spatial-kernel codes — the `kernel` discriminant of
/// [`LayerOp::SpatialAdjustment`]. The painter tool maps its
/// `AdjustmentKind::GaussianBlur`/`Sharpen` (etc.) to one of these. The
/// remaining kinds are reserved so the contract is stable as they land on the
/// same pass-graph (Motion/Bloom/ShadowsHighlights/ChromaticAberration).
///
/// `SPATIAL_GAUSSIAN` reads `params[0]` = radius. `SPATIAL_SHARPEN` is unsharp
/// mask (`src + amount·(src − blur(src))`) — `params[0]` = amount, `params[1]` =
/// blur radius — and reuses the Gaussian blur passes, differing only in the
/// combine step (`COMBINE_SHARPEN`).
pub const SPATIAL_GAUSSIAN: u8 = 0;
pub const SPATIAL_SHARPEN: u8 = 1;
/// Directional (motion) blur — a single 1-D pass along `angle` of length
/// `distance` (`params[0]` = distance, `params[1]` = angle in radians). Unlike
/// Gaussian/Sharpen (axis-aligned separable H/V), this swaps the BLUR STAGE for
/// `cs_blur_dir`; the combine is the passthrough (`COMBINE_GAUSSIAN`).
pub const SPATIAL_MOTION: u8 = 2;
/// Chromatic aberration — a single GATHER pass (`cs_chroma`) that samples the
/// below-composite at per-channel RADIALLY-shifted coords (R/G/B fringe toward
/// the edges). `params[0..3]` = red/green/blue shift in px (at the canvas corner),
/// `params[3]` = falloff_center (RESERVED in the spike — the provisional model is
/// linear-radial; the impl's `apply_chromatic_aberration` defines the curve).
/// The per-channel scales + centre are precomputed CPU-side so the gather does no
/// per-pixel sqrt (parity-robust, like motion); combine is the passthrough.
pub const SPATIAL_CHROMA: u8 = 3;
/// Bloom — bright-pass → separable Gaussian blur of the bright EXCESS → additive
/// glow. `params[0]` = threshold, `params[1]` = intensity, `params[2]` = radius,
/// `params[3]` = falloff. Adds the bright-pass `cs_bloom_bright` BEFORE the
/// (premultiplied) separable blur, then the additive `COMBINE_BLOOM` step. The
/// glow feathers coverage (haloes outward), so the combine adopts its alpha.
/// Mirror of `ph2d_painter_brush::adjustments::spatial::apply_bloom`.
pub const SPATIAL_BLOOM: u8 = 4;
/// Shadows/Highlights — LOCAL tonal correction. Uses all 8 `params`:
/// `[shadows_amount, shadows_tonal_width, shadows_radius, highlights_amount,
/// highlights_tonal_width, highlights_radius, color_correction, midtone_contrast]`.
/// `cs_sh_luma` extracts the display luma; the shared scalar blur builds two
/// local-average tone maps (the two radii); `cs_combine_sh` lifts shadows /
/// recovers highlights by the neighbourhood tone. Coverage is PRESERVED (a tonal
/// op, NOT an image blur). Mirror of
/// `ph2d_painter_brush::adjustments::spatial::apply_shadows_highlights`.
pub const SPATIAL_SHADOWS_HIGHLIGHTS: u8 = 5;

/// Combine-step mode (the post-blur math in `cs_combine`) — mirrors the WGSL
/// `combine_mode`. `GAUSSIAN` passes the blurred value through; `SHARPEN`
/// computes the unsharp mask from base + blurred; `BLOOM` adds `intensity·glow`
/// onto the premultiplied base. All then blend over the base.
pub(super) const COMBINE_GAUSSIAN: u32 = 0;
pub(super) const COMBINE_SHARPEN: u32 = 1;
pub(super) const COMBINE_BLOOM: u32 = 2;

/// Largest separable-blur half-width (kernel reaches `±MAX_BLUR_HALF` texels).
/// Bounds the weights buffer + the per-pixel tap count + the dirty-rect halo.
/// A 256-radius blur is already far past any interactive use.
pub const MAX_BLUR_HALF: u32 = 256;

/// Separable-Gaussian half-kernel. Returns `(weights, half_width)` where
/// `weights[i]` is the symmetric weight for offset `±i` (`weights[0]` = centre
/// tap), normalised so the full kernel sums to 1. σ = radius/3 (radius ≈ 3σ,
/// the textbook truncation); `half = ceil(radius)`.
///
/// This is `ph2d-render`'s SELF-CONTAINED copy: the crate is foundational and
/// must not gain a production dependency on the painter tool's domain crate (the
/// decoupling, see `Cargo.toml`). The painter impl's `ph2d_painter_brush::
/// adjustments::gaussian_weights` is the canonical math and this is bit-identical
/// to it — pinned by the `spatial_weights_parity` dev-test so the two copies can
/// never drift (single source of truth without coupling the libs).
#[must_use]
pub fn gaussian_weights(radius: f32) -> (Vec<f32>, u32) {
    let r = radius.max(0.0);
    let half = (r.ceil() as u32).clamp(1, MAX_BLUR_HALF);
    let sigma = (r / 3.0).max(1e-3);
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut weights = Vec::with_capacity(half as usize + 1);
    let mut sum = 0.0f32;
    for i in 0..=half {
        let x = i as f32;
        let g = (-(x * x) / two_sigma_sq).exp();
        weights.push(g);
        // The centre tap is counted once; each ±i flank tap is counted twice.
        sum += if i == 0 { g } else { 2.0 * g };
    }
    if sum > 0.0 {
        for w in &mut weights {
            *w /= sum;
        }
    }
    (weights, half)
}

/// Motion-blur kernel — uniform box along the motion line (constant-velocity
/// linear motion exposes every position equally). Returns `(weights, half)` for
/// the symmetric `2·half+1`-tap average (`weights[i] = 1/(2·half+1)`); `half =
/// ceil(distance/2)` so the line spans ≈ `distance` px. The taps are sampled
/// along the direction by `cs_blur_dir` (see [`SPATIAL_MOTION`]).
///
/// `ph2d-render`'s self-contained copy, bit-identical to the canonical
/// `ph2d_painter_brush::adjustments::motion_weights` (decoupling preserved;
/// pinned by the `spatial_weights_parity` dev-test). See [`gaussian_weights`].
#[must_use]
pub fn motion_weights(distance: f32) -> (Vec<f32>, u32) {
    let half = ((distance.max(0.0) / 2.0).ceil() as u32).clamp(1, MAX_BLUR_HALF);
    let w = 1.0 / (2 * half + 1) as f32;
    (vec![w; half as usize + 1], half)
}

/// Does this op-list contain a spatial adjustment (a pass break)? When false,
/// the compositor takes the untouched single-pass path; when true, it takes the
/// segmented pass-graph.
#[must_use]
pub fn has_spatial(ops: &[LayerOp]) -> bool {
    ops.iter()
        .any(|o| matches!(o, LayerOp::SpatialAdjustment { .. }))
}

/// Validate group push/pop balance + depth without touching the GPU.
pub(super) fn validate_op_list(ops: &[LayerOp]) -> Result<(), LayerCompositeError> {
    let mut depth: u32 = 0;
    for op in ops {
        match op {
            LayerOp::PushGroup => {
                depth += 1;
                if depth + 1 > MAX_STACK {
                    return Err(LayerCompositeError::MalformedOpList);
                }
            }
            LayerOp::PopGroup { .. } => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(LayerCompositeError::MalformedOpList)?;
            }
            // Layers + adjustments are depth-neutral (an adjustment transforms
            // the current accumulator in place, like a layer blends over it).
            // A spatial adjustment is likewise depth-neutral at the op-list
            // level — its multi-pass machinery runs between segments, but it
            // does not push/pop a group.
            LayerOp::Layer { .. }
            | LayerOp::Adjustment { .. }
            | LayerOp::SpatialAdjustment { .. } => {}
        }
    }
    if depth != 0 {
        return Err(LayerCompositeError::MalformedOpList);
    }
    Ok(())
}

/// Every texture-array key this op needs resident: its pixels, plus its mask.
///
/// **One door.** Three callers ask this question — the cap count, the upload
/// loop that makes slices resident, and the two dirty-key checks — and a mask
/// that one of them forgot would be a slice the shader samples but nobody
/// uploaded: silently the *wrong picture*, not an error.
///
/// Returns a fixed-size array so it stays allocation-free on the hot path
/// (HR-3); `None` entries are absent keys.
fn op_keys(op: &LayerOp) -> [Option<u64>; 2] {
    match op {
        LayerOp::Layer { key, mask, .. } => [Some(*key), mask.map(|m| m.key)],
        LayerOp::Adjustment { mask, .. } => [None, mask.map(|m| m.key)],
        LayerOp::PushGroup | LayerOp::PopGroup { .. } | LayerOp::SpatialAdjustment { .. } => {
            [None, None]
        }
    }
}

/// The mask this op carries, if any — the read-side companion of [`op_keys`].
pub(super) fn op_mask(op: &LayerOp) -> Option<LayerMask> {
    match op {
        LayerOp::Layer { mask, .. } | LayerOp::Adjustment { mask, .. } => *mask,
        LayerOp::PushGroup | LayerOp::PopGroup { .. } | LayerOp::SpatialAdjustment { .. } => None,
    }
}

/// Every key referenced by `ops`, in first-seen order, with duplicates dropped.
fn op_keys_in_order(ops: &[LayerOp]) -> impl Iterator<Item = u64> + '_ {
    // Allocation-free (HR-3): yield each key only at its FIRST occurrence. O(n²)
    // in op count, but n is a few hundred at most and this is off the GPU-bound
    // cost — cheaper than a per-`composite()` BTreeSet alloc on the documented
    // real-time path (audit 2026-06-01 LOW).
    ops.iter().enumerate().flat_map(move |(i, op)| {
        op_keys(op)
            .into_iter()
            .enumerate()
            .filter_map(move |(slot, key)| {
                let key = key?;
                // First occurrence = not present in any earlier op, and not in an
                // earlier slot of THIS op (a layer masked by itself is degenerate
                // but must not claim two slices).
                let earlier_here = op_keys(op)[..slot].contains(&Some(key));
                let earlier_op = ops[..i].iter().any(|o| op_keys(o).contains(&Some(key)));
                (!earlier_here && !earlier_op).then_some(key)
            })
    })
}

/// Distinct texture-array keys referenced by `ops` (layer pixels **and** masks).
pub(super) fn distinct_layer_count(ops: &[LayerOp]) -> u32 {
    op_keys_in_order(ops).count() as u32
}

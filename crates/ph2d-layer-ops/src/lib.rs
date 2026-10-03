//! ⭐⭐ **O VOCABULÁRIO do compositor de camadas** — a lista de operações que a
//! pilha do Painter (`ph2d_tool_painter::flatten_for_gpu`) produz e que o
//! compositor de GPU (`ph2d_render::layer_compositor`) corre.
//!
//! ⚠️ Folha de propósito, só tipos com números dentro: a ferramenta que fala esta
//! língua não pode depender do motor (`ph2d-render`, `wgpu`/`vello`), e o motor
//! não depende da ferramenta — os dois dependem daqui
//! (`architecture_no_dependency_climbs_a_layer`; `docs/3D/30` §13).

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

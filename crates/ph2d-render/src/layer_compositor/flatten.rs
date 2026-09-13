//! **Da lista de [`LayerOp`] à lista de operações da GPU** — o [`GpuOpScratch`] reutilizável e o
//! [`flatten_layer_ops`] que o enche sem alocar a quente —, irmão de `layer_compositor/mod.rs` por
//! tecto de LOC. O caminho público não muda: `mod.rs` re-exporta os dois.
//!
//! Corte mecânico: os itens saíram inteiros, verbatim; abriu-se só a visibilidade dos dois campos do
//! scratch para o módulo pai (o `compositor/` e os testes leem-nos).

use super::*;

/// Reusable scratch for the flattened GPU op-list. Construct once, reuse
/// across frames: [`flatten_layer_ops`] clears and refills it without
/// allocating once it is warm (HR-3 — `layers_no_alloc_hot_compose`).
#[derive(Default)]
pub struct GpuOpScratch {
    pub(super) ops: Vec<GpuOp>,
    /// Per-adjustment params, parallel to the `OP_ADJUSTMENT` ops (each such op's
    /// `layer_slot` indexes this). Reused across frames like `ops` (HR-3).
    pub(super) adj: Vec<AdjParamsGpu>,
}

impl GpuOpScratch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of compositor ops (`len()` historically meant the op count).
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Backing op capacity — exposed for the no-alloc gate to assert stability.
    #[doc(hidden)]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.ops.capacity()
    }
}

/// Flatten [`LayerOp`]s into `scratch`, resolving each layer key to its cached
/// slice via `slot_of`. The hot per-frame CPU work; reuses `scratch`'s
/// capacity so it does NOT allocate once warm (HR-3). A key `slot_of` resolves
/// to (defaulting to 0 for an absent key) becomes the texture-array slice the
/// shader samples. `composite` calls this with the live cache as the resolver.
///
/// A **mask** key resolves through `mask_slot_of`, which is deliberately
/// fallible where `slot_of` is not: a mask the provider could not serve at
/// canvas size degrades to *no mask* (the CPU reference's behaviour), rather
/// than poisoning the whole composite. See [`LayerMask`].
pub fn flatten_layer_ops(
    ops: &[LayerOp],
    slot_of: impl Fn(u64) -> u32,
    mask_slot_of: impl Fn(u64) -> Option<u32>,
    scratch: &mut GpuOpScratch,
) {
    /// `(mask_slot, flags)` for an op's optional mask + clipping flag.
    fn coverage(
        mask: Option<LayerMask>,
        clipping: bool,
        mask_slot_of: &impl Fn(u64) -> Option<u32>,
    ) -> (u32, u32) {
        let mut flags = if clipping { FLAG_CLIPPING } else { 0 };
        let slot = match mask.and_then(|m| mask_slot_of(m.key).map(|s| (s, m.inverted))) {
            Some((slot, inverted)) => {
                if inverted {
                    flags |= FLAG_MASK_INVERTED;
                }
                slot
            }
            None => NO_MASK_SLOT,
        };
        (slot, flags)
    }

    scratch.ops.clear();
    scratch.adj.clear();
    for op in ops {
        let g = match op {
            LayerOp::Layer {
                key,
                blend_mode,
                opacity,
                mask,
                clipping,
            } => {
                let (mask_slot, flags) = coverage(*mask, *clipping, &mask_slot_of);
                GpuOp {
                    kind: OP_LAYER,
                    layer_slot: slot_of(*key),
                    blend_mode: *blend_mode as u32,
                    // Clamp to [0,1] to match the CPU reference (compositor.rs
                    // clamps layer.opacity before folding into source alpha); an
                    // out-of-range opacity would otherwise diverge (audit LOW).
                    opacity: opacity.clamp(0.0, 1.0),
                    mask_slot,
                    flags,
                    _pad0: 0,
                    _pad1: 0,
                }
            }
            LayerOp::PushGroup => GpuOp {
                kind: OP_PUSH_GROUP,
                layer_slot: 0,
                blend_mode: 0,
                opacity: 1.0,
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
            LayerOp::PopGroup {
                blend_mode,
                opacity,
            } => GpuOp {
                kind: OP_POP_GROUP,
                layer_slot: 0,
                blend_mode: *blend_mode as u32,
                opacity: opacity.clamp(0.0, 1.0),
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
            LayerOp::Adjustment {
                kind,
                params,
                blend_mode,
                opacity,
                mask,
            } => {
                // The op's `layer_slot` indexes the params we stash in parallel.
                let params_index = scratch.adj.len() as u32;
                scratch.adj.push(AdjParamsGpu {
                    kind: *kind as u32,
                    p0: params[0],
                    p1: params[1],
                    p2: params[2],
                });
                let (mask_slot, flags) = coverage(*mask, false, &mask_slot_of);
                GpuOp {
                    kind: OP_ADJUSTMENT,
                    layer_slot: params_index,
                    blend_mode: *blend_mode as u32,
                    opacity: opacity.clamp(0.0, 1.0),
                    mask_slot,
                    flags,
                    _pad0: 0,
                    _pad1: 0,
                }
            }
            // Spatial adjustments are driven CPU-side as pass breaks; emit a
            // no-op placeholder so GPU op indices mirror the `LayerOp` list 1:1
            // (the segment compute loop ignores `OP_SPATIAL`). The kernel/params
            // are read from the original op-list by the segmented orchestrator.
            LayerOp::SpatialAdjustment { .. } => GpuOp {
                kind: OP_SPATIAL,
                layer_slot: 0,
                blend_mode: 0,
                opacity: 1.0,
                mask_slot: NO_MASK_SLOT,
                flags: 0,
                _pad0: 0,
                _pad1: 0,
            },
        };
        scratch.ops.push(g);
    }
}

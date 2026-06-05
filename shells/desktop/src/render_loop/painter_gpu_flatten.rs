//! Flatten a painter `LayerStack` → `Vec<LayerOp>` for the GPU
//! `ph2d_render::LayerCompositor` (Painter GPU preview, ADR-0045 Phase 3).
//!
//! This is the **GPU-vs-CPU gate** (handoff §3): the GPU op-list
//! (`Layer`/`PushGroup`/`PopGroup`/`Adjustment`/`SpatialAdjustment`) cannot
//! represent per-layer masks, clipping, reference layers, or masked adjustments —
//! and a kind with neither a per-pixel `gpu_code()` nor a spatial
//! `gpu_spatial_code()` (e.g. Bloom / Noise / Halftone / ColorLookup /
//! ShadowsHighlights) has no GPU op. When the stack uses ANY of those,
//! [`flatten_for_gpu`] returns `None` and the bridge falls back to the CPU
//! `take_preview_arc` path (correct, just slower — and it has the cut-point
//! cache). When it returns `Some`, the GPU composites the whole stack
//! (`base + simple adjustment` = the Enio hot case, ~1.7 ms @1024² vs ~55 ms CPU).
//!
//! The walk MIRRORS `ph2d_tool_painter::compositor::composite_into` EXACTLY —
//! `root().iter().rev()` (panel order is top-first, so iterate bottom-to-top),
//! group recursion, skip invisible / zero-opacity / mask layers. Any divergence
//! from that reference is a correctness bug; keep them in lock-step.

use ph2d_painter_brush::BlendMode;
use ph2d_painter_brush::adjustments::{AdjustmentParams, curves_display_luts, levels_display_lut};
use ph2d_render::layer_compositor::LayerOp;
use ph2d_tool_painter::{LayerId, LayerKind, LayerStack};

/// Flatten `stack` into a GPU op-list, or `None` if it is not GPU-representable
/// (mask / clipping / reference layer / masked adjustment / non-ported
/// adjustment kind) — the caller then uses the CPU compositor.
// Consumed by the GPU preview path in `painter_bridge` (Phase 3 step 2): the
// GPU-vs-CPU decision (a representable stack → GPU compositor, else CPU).
pub(super) fn flatten_for_gpu(stack: &LayerStack) -> Option<(Vec<LayerOp>, Vec<f32>)> {
    let mut ops = Vec::new();
    let mut adj_luts = Vec::new();
    flatten_ids(stack, stack.root(), &mut ops, &mut adj_luts)?;
    Some((ops, adj_luts))
}

fn flatten_ids(
    stack: &LayerStack,
    ids: &[LayerId],
    ops: &mut Vec<LayerOp>,
    adj_luts: &mut Vec<f32>,
) -> Option<()> {
    // Bottom-to-top, mirror of `composite_into`'s `ids.iter().rev()`.
    for &id in ids.iter().rev() {
        let Some(layer) = stack.get(id) else { continue };
        // Mask layers compose via their parent — never their own op.
        if matches!(layer.kind, LayerKind::Mask(_)) {
            continue;
        }
        if !layer.visible || layer.opacity <= 0.0 {
            continue;
        }
        // GPU op-list v1 can't represent these → bail to the CPU path.
        if layer.mask.is_some() || layer.clipping || layer.is_reference {
            return None;
        }
        let opacity = layer.opacity.clamp(0.0, 1.0);
        let blend_mode = layer.blend_mode.to_u8();
        match &layer.kind {
            LayerKind::Raster(_) => ops.push(LayerOp::Layer {
                key: id.0,
                blend_mode,
                opacity,
            }),
            LayerKind::Group(g) => {
                ops.push(LayerOp::PushGroup);
                flatten_ids(stack, &g.children, ops, adj_luts)?;
                ops.push(LayerOp::PopGroup {
                    blend_mode,
                    opacity,
                });
            }
            LayerKind::Adjustment(adj) => {
                if !adj.visible || adj.opacity <= 0.0 {
                    continue;
                }
                // Masked adjustment + non-ported kind → CPU.
                if adj.mask.is_some() {
                    return None;
                }
                // Spatial (neighbourhood) kinds run on the pass-graph as a
                // `SpatialAdjustment` (Gaussian/Sharpen/Motion/Chroma). `kernel`
                // is the `SPATIAL_*` code; `params` the kernel's 4 scalars — both
                // kept in lock-step with `ph2d-render` by the spatial parity gates.
                if let Some(kernel) = adj.kind.gpu_spatial_code() {
                    let params = adj.params.spatial_params().unwrap_or([0.0; 4]);
                    ops.push(LayerOp::SpatialAdjustment {
                        kernel,
                        params,
                        blend_mode: BlendMode::to_u8(adj.blend_mode),
                        opacity: adj.opacity.clamp(0.0, 1.0),
                    });
                    continue;
                }
                let kind = adj.kind.gpu_code()?;
                // Curves(7)/Levels(8) are GPU display-space transfer LUTs: build
                // the table from the SAME exporter the CPU compositor reads, append
                // it to `adj_luts`, and pass its base float offset in `params[0]`
                // (the WGSL `apply_adjustment` reads `adj_luts[base + c*256 + idx]`).
                let params = match kind {
                    7 => {
                        let AdjustmentParams::Curves(cp) = &adj.params else {
                            return None;
                        };
                        let base = adj_luts.len();
                        for channel in curves_display_luts(cp) {
                            adj_luts.extend_from_slice(&channel);
                        }
                        [base as f32, 0.0, 0.0]
                    }
                    8 => {
                        let AdjustmentParams::Levels(lp) = &adj.params else {
                            return None;
                        };
                        let base = adj_luts.len();
                        adj_luts.extend_from_slice(&levels_display_lut(lp));
                        [base as f32, 0.0, 0.0]
                    }
                    _ => adj.params.gpu_params(),
                };
                ops.push(LayerOp::Adjustment {
                    kind,
                    params,
                    blend_mode: BlendMode::to_u8(adj.blend_mode),
                    opacity: adj.opacity.clamp(0.0, 1.0),
                });
            }
            LayerKind::Mask(_) => unreachable!("masks skipped above"),
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_painter_brush::adjustments::AdjustmentKind;
    use ph2d_tool_painter::LayerStack;

    #[test]
    fn base_plus_ported_adjustment_is_gpu_representable() {
        let mut s = LayerStack::new();
        let base = s.add_raster("base", 4, 4).unwrap();
        let _adj = s
            .add_adjustment(AdjustmentKind::HueSaturationBrightness)
            .unwrap();
        let (ops, _luts) = flatten_for_gpu(&s).expect("base + HSB adjustment is GPU-representable");
        // Reversed walk (root is top-first [adj, base]) → base first, then adj.
        assert!(matches!(ops[0], LayerOp::Layer { key, .. } if key == base.0));
        assert!(matches!(ops[1], LayerOp::Adjustment { .. }));
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn curves_and_levels_emit_display_luts_with_base_offset() {
        // Curves(7) emits a 3×256 table block, Levels(8) a 1×256 block; each op's
        // params[0] is its base float offset into the concatenated `adj_luts`.
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        let _curves = s.add_adjustment(AdjustmentKind::Curves).unwrap();
        let _levels = s.add_adjustment(AdjustmentKind::Levels).unwrap();
        let (ops, luts) =
            flatten_for_gpu(&s).expect("base + Curves + Levels is GPU-representable (W4 §2)");
        assert_eq!(luts.len(), 3 * 256 + 256, "Curves 3×256 + Levels 1×256");
        let mut bases: Vec<(u8, f32)> = ops
            .iter()
            .filter_map(|o| match o {
                LayerOp::Adjustment { kind, params, .. } if *kind == 7 || *kind == 8 => {
                    Some((*kind, params[0]))
                }
                _ => None,
            })
            .collect();
        bases.sort_by_key(|&(_, base)| base as u32);
        // Two LUT ops at distinct, in-range base offsets (0 and one of 256/768).
        assert_eq!(bases.len(), 2, "one Curves + one Levels LUT op");
        assert_eq!(bases[0].1, 0.0, "first LUT op starts at base 0");
        assert!(
            (bases[1].1 as usize) < luts.len(),
            "second LUT op's base is in range"
        );
    }

    #[test]
    fn group_emits_push_pop_around_children() {
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        let child = s.add_raster("child", 4, 4).unwrap();
        let g = s.add_group("group").unwrap();
        s.move_into_group(child, g);
        let (ops, _luts) = flatten_for_gpu(&s).expect("plain group is GPU-representable");
        // Somewhere: PushGroup, Layer(child), PopGroup.
        let push = ops.iter().position(|o| matches!(o, LayerOp::PushGroup));
        let pop = ops
            .iter()
            .position(|o| matches!(o, LayerOp::PopGroup { .. }));
        assert!(
            push.is_some() && pop.is_some() && push < pop,
            "group brackets its child"
        );
    }

    #[test]
    fn non_ported_adjustment_falls_back_to_cpu() {
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        // Bloom has neither a per-pixel `gpu_code()` nor a `gpu_spatial_code()`
        // (it needs mip-pyramid infra that has not landed) → not representable.
        let _adj = s.add_adjustment(AdjustmentKind::Bloom).unwrap();
        assert!(
            flatten_for_gpu(&s).is_none(),
            "a non-ported adjustment kind must force the CPU fallback"
        );
    }

    #[test]
    fn spatial_adjustment_emits_pass_graph_op() {
        // GaussianBlur is a spatial kind → emits `SpatialAdjustment` (the GPU
        // pass-graph), NOT a CPU fallback. `params[0]` carries the radius.
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        let _adj = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
        let (ops, _luts) =
            flatten_for_gpu(&s).expect("base + GaussianBlur is GPU-representable (pass-graph)");
        assert!(
            ops.iter().any(|o| matches!(
                o,
                LayerOp::SpatialAdjustment { kernel, .. } if *kernel == 0 // SPATIAL_GAUSSIAN
            )),
            "GaussianBlur must flatten to a SpatialAdjustment(SPATIAL_GAUSSIAN): {ops:?}"
        );
    }

    #[test]
    fn clipping_layer_falls_back_to_cpu() {
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        let top = s.add_raster("top", 4, 4).unwrap();
        s.set_clipping(top, true);
        assert!(
            flatten_for_gpu(&s).is_none(),
            "clipping isn't in the GPU op-list v1 -> CPU fallback"
        );
    }

    #[test]
    fn masked_layer_falls_back_to_cpu() {
        let mut s = LayerStack::new();
        let base = s.add_raster("base", 4, 4).unwrap();
        s.add_mask(base).unwrap();
        assert!(
            flatten_for_gpu(&s).is_none(),
            "a per-layer mask isn't in the GPU op-list v1 -> CPU fallback"
        );
    }
}

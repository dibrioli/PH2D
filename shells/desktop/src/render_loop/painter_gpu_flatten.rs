//! Flatten a painter `LayerStack` → `Vec<LayerOp>` for the GPU
//! `ph2d_render::LayerCompositor` (Painter GPU preview, ADR-0045 Phase 3).
//!
//! This is the **GPU-vs-CPU gate** (handoff §3): the GPU op-list v1
//! (`Layer`/`PushGroup`/`PopGroup`/`Adjustment`) cannot represent per-layer
//! masks, clipping, reference layers, or masked adjustments — and a non-ported
//! adjustment kind has no `gpu_code()`. When the stack uses ANY of those,
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
use ph2d_render::layer_compositor::LayerOp;
use ph2d_tool_painter::{LayerId, LayerKind, LayerStack};

/// Flatten `stack` into a GPU op-list, or `None` if it is not GPU-representable
/// (mask / clipping / reference layer / masked adjustment / non-ported
/// adjustment kind) — the caller then uses the CPU compositor.
// Consumed by the GPU preview path in `painter_bridge` (Phase 3 step 2): the
// GPU-vs-CPU decision (a representable stack → GPU compositor, else CPU).
pub(super) fn flatten_for_gpu(stack: &LayerStack) -> Option<Vec<LayerOp>> {
    let mut ops = Vec::new();
    flatten_ids(stack, stack.root(), &mut ops)?;
    Some(ops)
}

fn flatten_ids(stack: &LayerStack, ids: &[LayerId], ops: &mut Vec<LayerOp>) -> Option<()> {
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
                flatten_ids(stack, &g.children, ops)?;
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
                let kind = adj.kind.gpu_code()?;
                ops.push(LayerOp::Adjustment {
                    kind,
                    params: adj.params.gpu_params(),
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
        let ops = flatten_for_gpu(&s).expect("base + HSB adjustment is GPU-representable");
        // Reversed walk (root is top-first [adj, base]) → base first, then adj.
        assert!(matches!(ops[0], LayerOp::Layer { key, .. } if key == base.0));
        assert!(matches!(ops[1], LayerOp::Adjustment { .. }));
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn group_emits_push_pop_around_children() {
        let mut s = LayerStack::new();
        let _base = s.add_raster("base", 4, 4).unwrap();
        let child = s.add_raster("child", 4, 4).unwrap();
        let g = s.add_group("group").unwrap();
        s.move_into_group(child, g);
        let ops = flatten_for_gpu(&s).expect("plain group is GPU-representable");
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
        // GaussianBlur is a spatial op with no `gpu_code()` → not representable.
        let _adj = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
        assert!(
            flatten_for_gpu(&s).is_none(),
            "a non-ported adjustment kind must force the CPU fallback"
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

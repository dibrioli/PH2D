#![forbid(unsafe_code)]
//! `motion.transform` — a Motion **modifier**: scales then offsets the `P`
//! (Vec2) attribute of its input instance stream, masked per-instance by the
//! multiplicative `falloff` column (§1.2; absent → `1.0`) so a focus field makes
//! only its region move — consistent with every other spatial node
//! (orbit/stagger/oscillator/wiggle all read `falloff`). Passes every other
//! column through unchanged (count is preserved). Pure.
//!
//! Params (read via `ctx.param` — per-instance override else the manifest
//! default shown): `scale` (1.0), `offset_x` (0.0), `offset_y` (0.0). Per
//! instance: `full = P * scale + (offset_x, offset_y)`, then the falloff blends
//! `P' = lerp(P, full, falloff)` (`falloff = 0` keeps `P`, `1` takes the full
//! transform).
//!
//! **Scope vs its siblings (audit 2026-07-10):** `scale` here scales POSITIONS
//! about the world origin (the layout spreads/contracts; dot size is
//! untouched) — a different thing from `motion.scale`, which scales the `size`
//! column (each sprite grows; layout untouched). And with `scale = 1` this
//! node degenerates to exactly `motion.move` (offset·falloff). It stays: it is
//! the only node that scales the layout, and the combined `P·s + o` affine is
//! one node instead of two. Reach for `move` for a plain offset, `scale` for
//! sprite size, and this for spreading a layout about the origin.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform"),
    name: "motion.transform",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset_x",
            default: 0.0,
        },
        ParamSpec {
            name: "offset_y",
            default: 0.0,
        },
    ],
    // `lowerings` stays `Cpu`: the `LoweringKind::Wgsl` path is the scalar
    // `eval_column` route (`ph2d-expr`), and `P` is a `Vec2` column, so that
    // route still doesn't apply. The GPU lowering this node DOES have is the
    // ADR-0126 side-channel kernel (`GPU_KERNEL` below, registered via
    // `register_gpu_kernel`) — a separate mechanism that never touches the
    // frozen `NodeManifest`, exactly like `motion.move`/`motion.oscillator`.
    lowerings: &[LoweringKind::Cpu],
};

/// The per-element affine map `p' = p * scale + (ox, oy)`. Pure and isolated so
/// the arithmetic is unit-tested directly with non-identity values, alongside
/// the end-to-end cook test that drives it via per-instance param overrides.
fn apply_xform(p: [f32; 2], scale: f32, ox: f32, oy: f32) -> [f32; 2] {
    [p[0] * scale + ox, p[1] * scale + oy]
}

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`) —
/// shared masking read, identical to the other spatial Motion nodes.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Apply the affine map to `p`, then blend from the original toward the
/// transformed position by `f` (the falloff): `f = 0` keeps `p`, `f = 1` takes
/// the full transform. Mirrors `motion.orbit`'s focus blend.
fn xform_masked(p: [f32; 2], scale: f32, ox: f32, oy: f32, f: f32) -> [f32; 2] {
    let full = apply_xform(p, scale, ox, oy);
    [p[0] + (full[0] - p[0]) * f, p[1] + (full[1] - p[1]) * f]
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): the exact per-element map of
/// the CPU `eval` — `full = p·scale + (ox, oy)` then `p' = p + (full − p)·falloff`
/// — in the SAME multiply/add order, so parity holds within GPU-FMA ε (the ε
/// gate). No `applicable`: a plain affine covers the whole param space (no enum,
/// no partial coverage). `ReadWriteExisting` on `P` mirrors the CPU's
/// pattern-match — a stream WITHOUT a `P` column passes through untouched, so
/// absence means the same thing on both paths (the falloff read materializes
/// its `1.0` identity when absent = full effect).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let xf_f = read_falloff(i);\n\
        let xf_p = read_P(i);\n\
        let xf_full = vec2<f32>(\n\
            xf_p.x * params.scale + params.offset_x,\n\
            xf_p.y * params.scale + params.offset_y);\n\
        write_P(i, vec2<f32>(\n\
            xf_p.x + (xf_full.x - xf_p.x) * xf_f,\n\
            xf_p.y + (xf_full.y - xf_p.y) * xf_f));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWriteExisting,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["scale", "offset_x", "offset_y"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionTransform;

impl NodeOp for MotionTransform {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let scale = ctx.param("scale");
        let (ox, oy) = (ctx.param("offset_x"), ctx.param("offset_y"));
        let out = {
            let input = ctx.input(0);
            // The port type guarantees `P` is `Vec2`; a `P` of any other dim is
            // an upstream node-author bug. Assert it loudly in debug/test rather
            // than silently passing it through untransformed (which would emit
            // geometry the params claim to have moved).
            debug_assert!(
                !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
                "motion.transform expects `P` to be a Vec2 column (port type guarantees it)"
            );
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        // Pure per-instance map → parallel above the threshold
                        // (bit-identical, no reduction). GPU/M5 Fase 0.
                        let t: Vec<[f32; 2]> = par_build(v.len(), |i| {
                            xform_masked(v[i], scale, ox, oy, falloff_at(input, i))
                        });
                        out.set("P", Column::Vec2(t));
                    }
                    // Every other column is per-element data this node does not
                    // touch — passed through unchanged (count is preserved).
                    _ => out.set(name.clone(), col.clone()),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTransform))?;
    // M1.R1 — UI metadata for the card (a spatial modifier → blue transform,
    // rounded-rect silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Transform",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // M1.P1 — param rows: uniform scale + signed offsets.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1) for the transform rows.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_x",
        label: "Offset X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_y",
        label: "Offset Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (1,1),(2,2) with a size column to verify passthrough.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.src"),
        name: "motion.transform.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("size", Column::Vec2(vec![[5.0, 5.0], [5.0, 5.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn scales_and_offsets_p_passes_size_through() {
        // default scale=1, offset=0 → P unchanged; assert passthrough + identity.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        assert_eq!(out[0].as_stream().count(), 2);
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 1.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
        // size carried through unchanged
        match out[0].as_stream().get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[5.0, 5.0], [5.0, 5.0]]),
            _ => panic!("size"),
        }
    }

    #[test]
    fn per_instance_overrides_drive_the_affine_through_the_cook() {
        // The real authoring path: override scale + offset on the instance and
        // see P transformed end-to-end (the cook only fed identity defaults
        // before per-instance params landed). src emits (1,1),(2,2).
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(xf, "scale", 2.0);
        g.set_param(xf, "offset_x", 10.0);
        g.set_param(xf, "offset_y", 1.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // (1,1)*2+(10,1) = (12,3) ; (2,2)*2+(10,1) = (14,5)
            Column::Vec2(v) => assert_eq!(v, &vec![[12.0, 3.0], [14.0, 5.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn apply_xform_scales_then_offsets() {
        // Unit-proves the `p*scale + offset` arithmetic directly with
        // non-identity values (the end-to-end override path is covered by
        // `per_instance_overrides_drive_the_affine_through_the_cook`).
        assert_eq!(apply_xform([2.0, 3.0], 2.0, 1.0, -1.0), [5.0, 5.0]);
        assert_eq!(apply_xform([0.0, 0.0], 10.0, 4.0, 7.0), [4.0, 7.0]);
        assert_eq!(apply_xform([1.0, 1.0], 0.0, 0.0, 0.0), [0.0, 0.0]); // collapse
        assert_eq!(apply_xform([-2.0, 5.0], 1.0, 0.0, 0.0), [-2.0, 5.0]); // identity
    }

    #[test]
    fn xform_masked_blends_by_falloff() {
        // f=1 → full transform; f=0 → unmoved; f=0.5 → halfway between.
        assert_eq!(xform_masked([1.0, 1.0], 2.0, 10.0, 0.0, 1.0), [12.0, 2.0]);
        assert_eq!(xform_masked([1.0, 1.0], 2.0, 10.0, 0.0, 0.0), [1.0, 1.0]);
        // full = (12, 2); midpoint with (1,1) = (6.5, 1.5).
        assert_eq!(xform_masked([1.0, 1.0], 2.0, 10.0, 0.0, 0.5), [6.5, 1.5]);
    }

    // Source with a falloff column so the mask is exercised end to end.
    static MASK_SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.masksrc"),
        name: "motion.transform.test.masksrc",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct MaskSrc;
    impl NodeOp for MaskSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &MASK_SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.0])),
            );
        }
    }
    struct MaskOps;
    impl OpResolver for MaskOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == MASK_SRC_MAN.id => Some(&MaskSrc),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn falloff_column_masks_the_transform_per_instance() {
        // scale 2 + offset (10,1): instance 0 (falloff 1) fully transforms;
        // instance 1 (falloff 0) is untouched — the focus field gates the move,
        // consistent with orbit/stagger/oscillator/wiggle.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.masksrc");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(xf, "scale", 2.0);
        g.set_param(xf, "offset_x", 10.0);
        g.set_param(xf, "offset_y", 1.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &MaskOps, xf, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // (1,1)*2+(10,1) = (12,3) at full falloff ; (2,2) unmoved at falloff 0.
            Column::Vec2(v) => assert_eq!(v, &vec![[12.0, 3.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
    }
}

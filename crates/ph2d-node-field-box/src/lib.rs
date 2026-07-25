#![forbid(unsafe_code)]
//! `field.box` — a Motion **focus field: an axis-aligned rectangle** keyed by
//! POSITION. It writes the same multiplicative `falloff` mask `motion.falloff`
//! and `field.index_range` do (the sister-of-`accel` contract, §1.2), so fields
//! compose. Unlike `motion.falloff`'s square `Rect` (a single `radius`), the box
//! has **independent `width`/`height`** and a **soft plateau**: the mask is a flat
//! `1` inside the core and ramps to `0` over `soft` world-units at each edge,
//! shaped by the same 4-curve (Linear/Quad/Smooth/Smoother).
//!
//! Independent axes are the whole point: a **wide-`width`, thin-`height`** box is a
//! razor-**horizontal** band (flat by y — the spatial answer to the ordinal tilt of
//! `field.index_range` on a grid); a thin-`width` box is a vertical band; a square
//! is a centred region no `radius` alone can size to a rectangle. It **multiplies**
//! into any existing `falloff` (count preserved). Pure. **Transcendental-free**
//! (HR-5): only `min`/`max`/`clamp` and polynomials, so the mask is bit-identical
//! across platforms for the replay hash.
//!
//! Params (read via `ctx.param`): `width` (8), `height` (8) — the **FULL** extents,
//! centred; `soft` (1) — the edge ramp in world-units; `center_x`/`center_y` (0);
//! `curve` (2 Smooth); `invert` (0/1 — flips to `1 − f`). The default is a soft
//! square in the middle (D12 — a fresh node does something visible). The neutral
//! is a box larger than the scene with `soft = 0` and `invert = 0` ⇒ mask `1`
//! everywhere ⇒ the `falloff` column is multiplied by the identity, byte-unchanged.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{par_build, Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.box"),
    name: "field.box",
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
            name: "width",
            default: 8.0,
        },
        ParamSpec {
            name: "height",
            default: 8.0,
        },
        ParamSpec {
            name: "soft",
            default: 1.0,
        },
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        ParamSpec {
            name: "rotation",
            default: 0.0,
        },
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME set as the other fields
/// (HR-5). `0` Linear · `1` Quad · `2` Smooth · `3` Smoother. Monotone, endpoint-exact.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The plateau-and-ramp on ONE axis: `half` is the half-extent (edge at `|d| =
/// half`), `soft` the ramp width clamped so it cannot exceed the half-extent (a
/// wider `soft` just meets in the middle, never overshoots the centre). `1`
/// inside `|d| ≤ half − soft`, ramping to `0` at `|d| = half`; `soft = 0` is a
/// hard edge. `half ≤ 0` degenerates to empty.
fn edge_ramp(d: f32, half: f32, soft: f32) -> f32 {
    let a = d.abs();
    let s = soft.max(0.0).min(half);
    if s > 0.0 {
        ((half - a) / s).clamp(0.0, 1.0)
    } else if a <= half {
        1.0
    } else {
        0.0
    }
}

/// The box mask at offset `(dx, dy)` from the centre, for FULL extents
/// `width`/`height`. The box is the INTERSECTION of the two axis bands (`min`),
/// then the curve shapes the combined ramp. Because every `curve` is monotone,
/// `curve(min(rx, ry))` equals `min(curve(rx), curve(ry))` — one eval, both edges.
fn box_mask(dx: f32, dy: f32, width: f32, height: f32, soft: f32, curve_kind: i32) -> f32 {
    let rx = edge_ramp(dx, width * 0.5, soft);
    let ry = edge_ramp(dy, height * 0.5, soft);
    curve(curve_kind, rx.min(ry))
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`box_mask`] × [`curve`]
/// multiplied into the existing `falloff` — same `min`/`max`/`clamp` + polynomials
/// (HR-5), so parity holds within float ULPs. `curve` routes through `fb_round`
/// (round-half-AWAY, matching Rust's `f32::round`; WGSL `round` is half-even) and
/// `invert` through the CPU's `>= 0.5`. `P` reads its `0` identity when absent (the
/// CPU's `unwrap_or([0,0])`); `falloff` `ReadWrite` from the `1.0` identity mirrors
/// the CPU (fields multiply, the column is always written).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let fb_p = read_P(i);\n\
        let fb_dx = fb_p.x - params.center_x;\n\
        let fb_dy = fb_p.y - params.center_y;\n\
        // Rotate the offset by -rotation into the box's local frame.\n\
        let fb_b = fb_cos_sin(params.rotation / 360.0);\n\
        let fb_lx = fb_dx * fb_b.x + fb_dy * fb_b.y;\n\
        let fb_ly = -fb_dx * fb_b.y + fb_dy * fb_b.x;\n\
        let fb_m = fb_box_mask(\n\
            fb_lx, fb_ly,\n\
            params.width,\n\
            params.height,\n\
            params.soft,\n\
            i32(fb_round(params.curve)));\n\
        var fb_f = fb_m;\n\
        if (params.invert >= 0.5) { fb_f = 1.0 - fb_m; }\n\
        write_falloff(i, read_falloff(i) * fb_f);\n",
    wgsl_lib: "\
        fn fb_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn fb_sin_cycles(phase: f32) -> f32 {\n\
            // The corrected parabolic sine (see trig.rs) — the SAME polynomial as\n\
            // the CPU, so parity holds; phase is in cycles (deg / 360).\n\
            let ff = phase - floor(phase);\n\
            var p: f32;\n\
            if (ff < 0.5) { let u = ff * 2.0; p = 4.0 * u * (1.0 - u); }\n\
            else { let u = (ff - 0.5) * 2.0; p = -4.0 * u * (1.0 - u); }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn fb_cos_sin(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(fb_sin_cycles(phase + 0.25), fb_sin_cycles(phase));\n\
        }\n\
        fn fb_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn fb_edge_ramp(d: f32, half: f32, soft: f32) -> f32 {\n\
            let a = abs(d);\n\
            let s = min(max(soft, 0.0), half);\n\
            if (s > 0.0) { return clamp((half - a) / s, 0.0, 1.0); }\n\
            return select(0.0, 1.0, a <= half);\n\
        }\n\
        fn fb_box_mask(dx: f32, dy: f32, width: f32, height: f32, soft: f32, curve_kind: i32) -> f32 {\n\
            let rx = fb_edge_ramp(dx, width * 0.5, soft);\n\
            let ry = fb_edge_ramp(dy, height * 0.5, soft);\n\
            return fb_curve(curve_kind, min(rx, ry));\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "width", "height", "soft", "center_x", "center_y", "rotation", "curve", "invert",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldBox;

impl NodeOp for FieldBox {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let width = ctx.param("width");
        let height = ctx.param("height");
        let soft = ctx.param("soft");
        let (cx, cy) = (ctx.param("center_x"), ctx.param("center_y"));
        // The rotation basis, computed ONCE (a per-cook constant): cos/sin of the
        // field's rotation. Rotating a world offset by −rotation brings it into the
        // box's local frame, where the test is axis-aligned.
        let (rc, rs) = cos_sin_cycles(ctx.param("rotation") / 360.0);
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let positions: &[[f32; 2]] = match input.get("P") {
                Some(Column::Vec2(v)) => v.as_slice(),
                _ => &[],
            };
            let fall = par_build(n, |i| {
                let p = positions.get(i).copied().unwrap_or([0.0, 0.0]);
                let (dx, dy) = (p[0] - cx, p[1] - cy);
                // Rotate the offset by −rotation into the box's local frame.
                let (lx, ly) = (dx * rc + dy * rs, -dx * rs + dy * rc);
                let m = box_mask(lx, ly, width, height, soft, curve_kind);
                let f = if invert { 1.0 - m } else { m };
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                base * f
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via `ph2d-node-sync`
/// codegen) from `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FieldBox))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Box",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): full Width/Height/Softness in world-units, signed
/// centre, a named Curve selector, an Invert checkbox.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "width",
        label: "Width",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "Height",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "Softness",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rotation",
        label: "Rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.box.test.src"),
        name: "field.box.test.src",
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
    struct Src(Vec<[f32; 2]>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with("P", Column::Vec2(self.0.clone())));
        }
    }
    // Holds the source so the resolver hands out a `&self`-lifetime op keyed by
    // THIS `Ops`'s points — no shared static (which would pin the first set).
    struct Ops {
        src: Src,
    }
    impl Ops {
        fn new(pts: Vec<[f32; 2]>) -> Self {
            Ops { src: Src(pts) }
        }
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&self.src),
                t if t == MANIFEST.id => Some(&FieldBox),
                _ => None,
            }
        }
    }

    fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(g, ops, target, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("falloff must be a Scalar column"),
        }
    }

    fn chain() -> (Graph, NodeId) {
        let mut g = Graph::new();
        let src = g.add_node("field.box.test.src");
        let bx = g.add_node("field.box");
        g.connect(Edge {
            from: (src, 0),
            to: (bx, 0),
            delayed: false,
        })
        .unwrap();
        (g, bx)
    }

    #[test]
    fn default_box_is_one_at_center_zero_outside() {
        // Default width/height 8 (half-extent 4), soft 1, centre origin, curve
        // Smooth. (0,0) is deep inside → 1; (4,0) sits exactly on the edge → 0;
        // (5,0) is outside → 0.
        let (g, bx) = chain();
        let ops = Ops::new(vec![[0.0, 0.0], [4.0, 0.0], [5.0, 0.0]]);
        assert_eq!(falloff_of(&g, &ops, bx), vec![1.0, 0.0, 0.0]);
    }

    #[test]
    fn independent_axes_make_a_horizontal_band() {
        // A WIDE, THIN box: width 20 (half 10), height 4 (half 2), soft 0 (hard).
        // The mask is `1` for |y| <= 2 at ANY x within ±10, `0` above |y| = 2 —
        // a razor-horizontal band, flat by y (the spatial answer to the ordinal
        // tilt of field.index_range). It is what a single `radius` cannot express.
        let (mut g, bx) = chain();
        g.set_param(bx, "width", 20.0);
        g.set_param(bx, "height", 4.0);
        g.set_param(bx, "soft", 0.0);
        // Points: inside band at three x; just above the band; well outside.
        let ops = Ops::new(vec![
            [-8.0, 0.0],
            [0.0, 1.9],
            [8.0, 1.0],
            [0.0, 2.1],
            [0.0, 5.0],
        ]);
        assert_eq!(falloff_of(&g, &ops, bx), vec![1.0, 1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn rotation_90_swaps_the_axes() {
        // A wide (x), thin (y) box rotated 90° becomes a wide-y, thin-x box — a
        // VERTICAL band. `cos_sin_cycles(0.25)` is exactly `(0, 1)`, so the frame
        // maps (dx, dy) → (dy, −dx): a point above the horizontal band is now inside
        // the rotated (vertical) one, and a point in the old wide band is outside.
        let (mut g, bx) = chain();
        g.set_param(bx, "width", 20.0); // wide in local x
        g.set_param(bx, "height", 4.0); // thin in local y
        g.set_param(bx, "soft", 0.0);
        g.set_param(bx, "rotation", 90.0);
        // (0, 8): local (8, 0) → inside (|8| < 10, |0| < 2). (8, 0): local (0, −8)
        // → outside (|−8| > 2).
        let ops = Ops::new(vec![[0.0, 8.0], [8.0, 0.0]]);
        assert_eq!(falloff_of(&g, &ops, bx), vec![1.0, 0.0]);
    }

    #[test]
    fn rotation_zero_is_byte_identical_to_the_unrotated_box() {
        // The default (rotation 0) must be EXACTLY the axis-aligned box (the trig
        // basis is exactly (1, 0)), so every prior golden and the parity 0e0 hold.
        let (g, bx) = chain();
        let ops = Ops::new(vec![[0.0, 0.0], [4.0, 0.0], [3.5, 0.0], [5.0, 5.0]]);
        // Same as default_box: 1 at centre, 0 at/after the edge; (3.5,0) on the
        // Smooth ramp; (5,5) outside.
        let got = falloff_of(&g, &ops, bx);
        assert_eq!(got[0], 1.0);
        assert_eq!(got[1], 0.0);
        assert_eq!(got[3], 0.0);
        assert!(got[2] > 0.0 && got[2] < 1.0);
    }

    #[test]
    fn full_extent_larger_than_scene_is_the_identity() {
        // The neutral: a box far larger than the points, soft 0 ⇒ mask 1
        // everywhere ⇒ the falloff column is multiplied by the identity (D12).
        let (mut g, bx) = chain();
        g.set_param(bx, "width", 100.0);
        g.set_param(bx, "height", 100.0);
        g.set_param(bx, "soft", 0.0);
        let ops = Ops::new(vec![[0.0, 0.0], [10.0, -7.0], [-4.0, 3.0]]);
        assert_eq!(falloff_of(&g, &ops, bx), vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn invert_flips_the_mask() {
        let (mut g, bx) = chain();
        g.set_param(bx, "invert", 1.0);
        // Default box: (0,0) → 1-1 = 0; (5,0) outside → 1-0 = 1.
        let ops = Ops::new(vec![[0.0, 0.0], [5.0, 0.0]]);
        assert_eq!(falloff_of(&g, &ops, bx), vec![0.0, 1.0]);
    }

    #[test]
    fn a_prior_falloff_column_is_multiplied_not_overwritten() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("field.box.test.fsrc"),
            name: "field.box.test.fsrc",
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
        struct FSrc;
        impl NodeOp for FSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &FSRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // (0,0) inside the default box, (20,0) far outside; prior falloff.
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [20.0, 0.0]]))
                        .with("falloff", Column::Scalar(vec![0.5, 0.9])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&FieldBox),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("field.box.test.fsrc");
        let bx = g.add_node("field.box");
        g.connect(Edge {
            from: (src, 0),
            to: (bx, 0),
            delayed: false,
        })
        .unwrap();
        // Default box: mask 1 at (0,0), 0 at (20,0). Composed: 0.5·1, 0.9·0.
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, bx, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.0]),
            _ => panic!("falloff"),
        }
    }

    #[test]
    fn edge_ramp_plateaus_then_falls() {
        // half 4, soft 1: flat 1 until |d| = 3, linear ramp to 0 at |d| = 4.
        assert_eq!(edge_ramp(0.0, 4.0, 1.0), 1.0);
        assert_eq!(edge_ramp(3.0, 4.0, 1.0), 1.0);
        assert_eq!(edge_ramp(3.5, 4.0, 1.0), 0.5);
        assert_eq!(edge_ramp(4.0, 4.0, 1.0), 0.0);
        assert_eq!(edge_ramp(9.0, 4.0, 1.0), 0.0);
        // soft wider than the half-extent is clamped to it (never overshoots).
        assert_eq!(edge_ramp(0.0, 2.0, 10.0), 1.0);
    }

    #[test]
    fn curves_are_monotone_and_endpoint_exact() {
        for k in 0..=3 {
            assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
            assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
        }
        assert_eq!(curve(1, 0.5), 0.25);
        assert_eq!(curve(2, 0.5), 0.5);
    }

    #[test]
    fn degenerate_zero_extent_is_empty() {
        // width 0 ⇒ half 0 ⇒ empty on x ⇒ the box is empty everywhere but the axis.
        assert_eq!(box_mask(1.0, 0.0, 0.0, 8.0, 1.0, 0), 0.0);
        assert_eq!(box_mask(0.0, 1.0, 8.0, 0.0, 1.0, 0), 0.0);
    }
}

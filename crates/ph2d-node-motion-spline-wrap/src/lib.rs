#![forbid(unsafe_code)]
//! `motion.spline_wrap` — **wrap a layout onto a curve**: the Cinema 4D "Spline Wrap"
//! deformer (Motion Nodes M3, deformers — doc 01 §3 / doc 28). Where `motion.bend` bows
//! a layout along a *circular arc*, this bends it along an **arbitrary cubic Bézier**
//! (authored by four control-point params) — a banner following an S, text running along
//! a path. Strictly more general than `bend`.
//!
//! **Algorithm — map the layout's X onto the curve's arc length, offset by Y along the
//! normal.** Each element's `x` is normalised over the layout's bounding box to
//! `u ∈ [0, 1]`, slid by `offset` (wrapping); the point at arc position `u` gives a
//! frame `(B, tangent, normal)`, and the element lands at `B + normal · (y ·
//! height_scale)` — so the layout's rows follow the curve and its columns stack along the
//! normal. An `amount` **value** input blends flat → wrapped (unconnected → fully
//! wrapped), so a `value.lfo` flattens and re-wraps it. Falloff-masked. Self-contained:
//! the curve is in the params (not the vector document). Transcendental-free (HR-5):
//! polynomial curve + `sqrt` normalisation, no trig. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod curve;
use curve::{ArcLut, EPS, P2, arc_lut, frame_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spline_wrap"),
    name: "motion.spline_wrap",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Blend flat → wrapped (animatable). Optional: unconnected reads as 1 (fully
        // wrapped).
        PortSpec {
            name: "amount",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // How far the layout's Y offsets the point along the curve normal.
        ParamSpec {
            name: "height_scale",
            default: 1.0,
        },
        // Slides the whole layout along the arc (wraps 0..1).
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // The four control points (world units). Default: a gentle S-curve.
        ParamSpec {
            name: "p0x",
            default: -3.0,
        },
        ParamSpec {
            name: "p0y",
            default: -1.5,
        },
        ParamSpec {
            name: "p1x",
            default: -1.0,
        },
        ParamSpec {
            name: "p1y",
            default: 2.0,
        },
        ParamSpec {
            name: "p2x",
            default: 1.0,
        },
        ParamSpec {
            name: "p2y",
            default: -2.0,
        },
        ParamSpec {
            name: "p3x",
            default: 3.0,
        },
        ParamSpec {
            name: "p3y",
            default: 1.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The `amount`: unconnected (empty) → 1.0; else the first element.
fn amount_of(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(1.0)
}

/// The multiplicative falloff for element `i` (empty → 1.0).
fn falloff_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(1.0),
    }
}

/// Wrap `p` onto the curve `cp`: map each element's x (normalised over the bbox, slid by
/// `offset`) to an arc position, offset by y along the normal, blended by `amount` ·
/// `falloff`. A pure function — the whole node.
fn wrap(
    p: &[P2],
    cp: &[P2; 4],
    height_scale: f32,
    offset: f32,
    amount: f32,
    falloff: &[f32],
) -> Vec<P2> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
    for q in p {
        xmin = xmin.min(q[0]);
        xmax = xmax.max(q[0]);
    }
    let w = xmax - xmin;
    let lut: ArcLut = arc_lut(cp);
    (0..n)
        .map(|i| {
            let u = if w < EPS { 0.5 } else { (p[i][0] - xmin) / w };
            // Clamp (not wrap): the layout spans the curve [0,1]; `offset` slides it and
            // clamps at the ends, so the endpoint (u=1) stays on the curve end.
            let s = (u + offset).clamp(0.0, 1.0);
            let (b, _t, un) = frame_at(cp, &lut, s);
            let wrapped = [
                b[0] + un[0] * p[i][1] * height_scale,
                b[1] + un[1] * p[i][1] * height_scale,
            ];
            let a = (amount * falloff_at(falloff, i)).clamp(0.0, 1.0);
            [
                p[i][0] + (wrapped[0] - p[i][0]) * a,
                p[i][1] + (wrapped[1] - p[i][1]) * a,
            ]
        })
        .collect()
}

struct MotionSplineWrap;

impl NodeOp for MotionSplineWrap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let height_scale = ctx.param("height_scale");
        let offset = ctx.param("offset");
        let cp = [
            [ctx.param("p0x"), ctx.param("p0y")],
            [ctx.param("p1x"), ctx.param("p1y")],
            [ctx.param("p2x"), ctx.param("p2y")],
            [ctx.param("p3x"), ctx.param("p3y")],
        ];
        let amount = amount_of(&scalar_col(ctx.input(1), VALUE_COL));
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<P2> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let falloff = scalar_col(input, "falloff");
        let out_p = wrap(&p, &cp, height_scale, offset, amount, &falloff);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(out_p));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSplineWrap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spline Wrap",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "height_scale",
        label: "Height",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    pt("p0x", "P0 X"),
    pt("p0y", "P0 Y"),
    pt("p1x", "P1 X"),
    pt("p1y", "P1 Y"),
    pt("p2x", "P2 X"),
    pt("p2y", "P2 Y"),
    pt("p3x", "P3 X"),
    pt("p3y", "P3 Y"),
];

const fn pt(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S_CURVE: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
    const LINE: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
    // A symmetric arch (hump) — its arc-midpoint lifts clearly off the endpoint chord,
    // unlike the antisymmetric S-curve whose midpoint sits *on* the chord.
    const ARCH: [P2; 4] = [[-3.0, 0.0], [-1.0, 3.0], [1.0, 3.0], [3.0, 0.0]];

    /// `amount` 0 is the identity — the layout is untouched.
    #[test]
    fn amount_zero_is_the_identity() {
        let p = vec![[-2.0, 0.5], [0.0, -0.3], [2.0, 0.1]];
        let out = wrap(&p, &S_CURVE, 1.0, 0.0, 0.0, &[]);
        for (o, q) in out.iter().zip(&p) {
            assert!(
                (o[0] - q[0]).abs() < 1e-5 && (o[1] - q[1]).abs() < 1e-5,
                "{o:?} vs {q:?}"
            );
        }
    }

    /// Wrapping onto a straight horizontal line keeps a straight input row straight (the
    /// remap is affine there): three points at constant y stay collinear.
    #[test]
    fn a_row_on_a_straight_curve_stays_straight() {
        let p = vec![[-2.0, 0.4], [0.0, 0.4], [2.0, 0.4]];
        let out = wrap(&p, &LINE, 1.0, 0.0, 1.0, &[]);
        // Constant normal (+y) ⇒ all share the same y; collinear.
        assert!((out[0][1] - out[1][1]).abs() < 1e-3 && (out[1][1] - out[2][1]).abs() < 1e-3);
    }

    /// Wrapping onto a curved spline BENDS a straight input row: the midpoint leaves the
    /// chord between the endpoints. FALSIFIED by a flat deformer (midpoint on the chord).
    #[test]
    fn a_row_on_a_curved_spline_bends() {
        let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]; // a straight row along x
        let out = wrap(&p, &ARCH, 1.0, 0.0, 1.0, &[]);
        // Cross product of (mid−a) and (b−a): non-zero ⇒ the midpoint bent off the line.
        let (a, mid, b) = (out[0], out[1], out[2]);
        let cross = (mid[0] - a[0]) * (b[1] - a[1]) - (mid[1] - a[1]) * (b[0] - a[0]);
        assert!(cross.abs() > 0.5, "the row bent (cross {cross})");
    }

    /// Falloff masks the wrap per element: falloff 0 leaves an element where it was.
    #[test]
    fn falloff_masks_the_wrap() {
        let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]];
        let falloff = vec![1.0, 0.0, 1.0]; // middle element pinned
        let out = wrap(&p, &S_CURVE, 1.0, 0.0, 1.0, &falloff);
        assert_eq!(out[1], p[1], "falloff 0 -> unchanged");
    }

    /// Deterministic + cooks through the registry, copying columns and wrapping P.
    #[test]
    fn registers_and_wraps_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.spline_wrap.test.src"),
            name: "motion.spline_wrap.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]))
                        .with("size", Column::Vec2(vec![[0.3, 0.3]; 3])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionSplineWrap),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.spline_wrap.test.src");
        let sw = g.add_node("motion.spline_wrap");
        g.connect(Edge {
            from: (src, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sw, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("size").is_some(), "columns pass through");
        match s.get("P").unwrap() {
            Column::Vec2(v) => {
                // The wrapped row is no longer flat on y = 0 (the S-curve lifted it).
                assert!(
                    v.iter().any(|q| q[1].abs() > 0.3),
                    "wrapped off the axis: {v:?}"
                );
            }
            _ => panic!("P"),
        }
    }
}

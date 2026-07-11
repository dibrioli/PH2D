#![forbid(unsafe_code)]
//! `motion.four_point_warp` — a **projective corner-pin**: warp the whole layout so
//! its bounding rectangle maps onto an arbitrary quadrilateral, with straight lines
//! staying straight (true perspective) (Motion Nodes M3, deformers — doc 01 §3 /
//! doc 24). The After Effects "Corner Pin" / a projector keystone: pin the four
//! corners and the field billows into perspective.
//!
//! **Algorithm — the projective homography from the unit square to a quadrilateral
//! (Heckbert, *Projective Mappings for Image Warping*, 1989), the gold standard.**
//! Each element's position is normalised to `(u, v) ∈ [0,1]²` within the layout's
//! bounding box, then mapped through the 3×3 homography `H` whose image of the unit
//! square's corners is the four target corners; the perspective divide `(X/W, Y/W)`
//! is what keeps straight lines straight (unlike a bilinear patch, which bows them).
//! `H` has Heckbert's closed form (an affine branch when the quad is a parallelogram,
//! else the projective `g,h` terms). Transcendental-free (HR-5): the homography and
//! the divide are arithmetic — no trig, no `sqrt`.
//!
//! **The corners are animatable.** Each corner has an offset param (`tl_dx`…`br_dy`,
//! from the bounding-box corner); a `warp` **value** input scales them all `0→1`, so a
//! `value.lfo` billows the layout into the quad and flattens it back. Unconnected
//! `warp` reads as 1 (offsets fully applied); all-zero offsets ⇒ identity. Falloff-
//! masked (the multiplicative `falloff` column blends warped vs original per element).
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `warp` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Below this the bounding box (or a homography denominator) is treated as degenerate.
const EPS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.four_point_warp"),
    name: "motion.four_point_warp",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Warp amount, scaling every corner offset (animatable). Optional: unconnected
        // reads as 1 (the corners are fully applied).
        PortSpec {
            name: "warp",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // The four corners as offsets (world units) from the layout's bounding-box corners.
    // All zero ⇒ identity. Order: top-left, top-right, bottom-right, bottom-left.
    params: &[
        ParamSpec {
            name: "tl_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "tl_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "tr_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "tr_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "br_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "br_dy",
            default: 0.0,
        },
        ParamSpec {
            name: "bl_dx",
            default: 0.0,
        },
        ParamSpec {
            name: "bl_dy",
            default: 0.0,
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

/// The `warp` amount: unconnected (empty) → 1.0; else the first element (broadcast).
fn warp_amount(vals: &[f32]) -> f32 {
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

/// The projective homography mapping the unit square to the quadrilateral `c` (corners
/// in the order `(0,0)→c0, (1,0)→c1, (1,1)→c2, (0,1)→c3`). Returns the 9 coefficients
/// `[a,b,c, d,e,f, g,h,1]` of the 3×3 matrix (Heckbert's closed form: an affine branch
/// for a parallelogram, else the projective terms).
fn homography(c: &[[f32; 2]; 4]) -> [f32; 9] {
    let (x0, y0) = (c[0][0], c[0][1]);
    let (x1, y1) = (c[1][0], c[1][1]);
    let (x2, y2) = (c[2][0], c[2][1]);
    let (x3, y3) = (c[3][0], c[3][1]);
    let sx = x0 - x1 + x2 - x3;
    let sy = y0 - y1 + y2 - y3;
    if sx.abs() < EPS && sy.abs() < EPS {
        // Parallelogram → affine (no perspective terms).
        return [x1 - x0, x3 - x0, x0, y1 - y0, y3 - y0, y0, 0.0, 0.0, 1.0];
    }
    let dx1 = x1 - x2;
    let dx2 = x3 - x2;
    let dy1 = y1 - y2;
    let dy2 = y3 - y2;
    let den = dx1 * dy2 - dx2 * dy1;
    if den.abs() < EPS {
        // Degenerate quad — fall back to identity-ish affine on the first edge.
        return [x1 - x0, x3 - x0, x0, y1 - y0, y3 - y0, y0, 0.0, 0.0, 1.0];
    }
    let g = (sx * dy2 - sy * dx2) / den;
    let h = (dx1 * sy - dy1 * sx) / den;
    [
        x1 - x0 + g * x1,
        x3 - x0 + h * x3,
        x0,
        y1 - y0 + g * y1,
        y3 - y0 + h * y3,
        y0,
        g,
        h,
        1.0,
    ]
}

/// Apply the homography to `(u, v)` (the perspective divide keeps lines straight).
fn apply(m: &[f32; 9], u: f32, v: f32) -> [f32; 2] {
    let w = m[6] * u + m[7] * v + m[8];
    let w = if w.abs() < EPS { EPS } else { w };
    [
        (m[0] * u + m[1] * v + m[2]) / w,
        (m[3] * u + m[4] * v + m[5]) / w,
    ]
}

/// Warp `p` (all elements) into the quad given by the four `corner` offsets scaled by
/// `warp`, blended per element by `falloff`. A pure function — the whole node.
fn warp_positions(
    p: &[[f32; 2]],
    corners: &[[f32; 2]; 4],
    warp: f32,
    falloff: &[f32],
) -> Vec<[f32; 2]> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    // Bounding box of the layout.
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for q in p {
        xmin = xmin.min(q[0]);
        xmax = xmax.max(q[0]);
        ymin = ymin.min(q[1]);
        ymax = ymax.max(q[1]);
    }
    let (w, h) = (xmax - xmin, ymax - ymin);
    if w < EPS || h < EPS {
        return p.to_vec(); // a line/point has no 2D box to warp
    }
    // Target corners = bbox corner + warp·offset. Heckbert order: BL, BR, TR, TL.
    let base = [
        [xmin, ymin], // (u,v)=(0,0) bottom-left
        [xmax, ymin], // (1,0) bottom-right
        [xmax, ymax], // (1,1) top-right
        [xmin, ymax], // (0,1) top-left
    ];
    // corners param order is TL,TR,BR,BL → remap to the Heckbert BL,BR,TR,TL order.
    let off = [corners[3], corners[2], corners[1], corners[0]]; // BL,BR,TR,TL
    let quad = [
        [base[0][0] + warp * off[0][0], base[0][1] + warp * off[0][1]],
        [base[1][0] + warp * off[1][0], base[1][1] + warp * off[1][1]],
        [base[2][0] + warp * off[2][0], base[2][1] + warp * off[2][1]],
        [base[3][0] + warp * off[3][0], base[3][1] + warp * off[3][1]],
    ];
    let m = homography(&quad);
    (0..n)
        .map(|i| {
            let (u, v) = ((p[i][0] - xmin) / w, (p[i][1] - ymin) / h);
            let warped = apply(&m, u, v);
            let f = falloff_at(falloff, i).clamp(0.0, 1.0);
            [
                p[i][0] + (warped[0] - p[i][0]) * f,
                p[i][1] + (warped[1] - p[i][1]) * f,
            ]
        })
        .collect()
}

struct MotionFourPointWarp;

impl NodeOp for MotionFourPointWarp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Corner offsets, in TL,TR,BR,BL order.
        let corners = [
            [ctx.param("tl_dx"), ctx.param("tl_dy")],
            [ctx.param("tr_dx"), ctx.param("tr_dy")],
            [ctx.param("br_dx"), ctx.param("br_dy")],
            [ctx.param("bl_dx"), ctx.param("bl_dy")],
        ];
        let warp = warp_amount(&scalar_col(ctx.input(1), VALUE_COL));
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let falloff = scalar_col(input, "falloff");
        let out_p = warp_positions(&p, &corners, warp, &falloff);
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
    reg.register(Box::new(MotionFourPointWarp))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Four Point Warp",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Corner-offset sliders (world units), in TL,TR,BR,BL order.
static PARAM_HINTS: &[ParamUiHint] = &[
    hint("tl_dx", "TL X"),
    hint("tl_dy", "TL Y"),
    hint("tr_dx", "TR X"),
    hint("tr_dy", "TR Y"),
    hint("br_dx", "BR X"),
    hint("br_dy", "BR Y"),
    hint("bl_dx", "BL X"),
    hint("bl_dy", "BL Y"),
];

const fn hint(param: &'static str, label: &'static str) -> ParamUiHint {
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

    const ZERO: [[f32; 2]; 4] = [[0.0; 2]; 4];

    /// The homography maps the unit square's corners EXACTLY onto the quad — the
    /// definition, and the strongest falsifier for the closed form: any sign error
    /// and a corner lands somewhere else.
    #[test]
    fn homography_maps_the_unit_square_corners_onto_the_quad() {
        // An asymmetric perspective quad (not a parallelogram → the projective branch).
        let quad = [[0.0, 0.0], [4.0, 0.5], [3.0, 3.0], [0.5, 2.5]];
        let m = homography(&quad);
        let uv = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        for (k, [u, v]) in uv.iter().enumerate() {
            let got = apply(&m, *u, *v);
            assert!(
                (got[0] - quad[k][0]).abs() < 1e-4 && (got[1] - quad[k][1]).abs() < 1e-4,
                "corner {k}: {got:?} vs {:?}",
                quad[k]
            );
        }
    }

    /// Projective ⇒ straight lines stay straight: three collinear input points map to
    /// three collinear output points. FALSIFIED by a bilinear patch, which bows the
    /// midpoint off the line.
    #[test]
    fn straight_lines_stay_straight() {
        // A trapezoid (top edge pulled inward → keystone perspective).
        let quad = [[0.0, 0.0], [4.0, 0.0], [3.0, 3.0], [1.0, 3.0]];
        let m = homography(&quad);
        // Three collinear points along the left→right mid line (v = 0.5).
        let a = apply(&m, 0.0, 0.5);
        let mid = apply(&m, 0.5, 0.5);
        let b = apply(&m, 1.0, 0.5);
        // Cross product of (mid−a) and (b−a) ≈ 0 ⇒ collinear.
        let (ux, uy) = (mid[0] - a[0], mid[1] - a[1]);
        let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
        let cross = ux * vy - uy * vx;
        assert!(cross.abs() < 1e-3, "collinear preserved (cross {cross})");
    }

    /// Zero offsets (or `warp` 0) ⇒ identity: every element is unchanged.
    #[test]
    fn zero_offsets_are_the_identity() {
        let p = vec![
            [-2.0, -1.0],
            [2.0, -1.0],
            [2.0, 1.0],
            [-2.0, 1.0],
            [0.5, 0.3],
        ];
        let out = warp_positions(&p, &ZERO, 1.0, &[]);
        for (o, q) in out.iter().zip(&p) {
            assert!(
                (o[0] - q[0]).abs() < 1e-4 && (o[1] - q[1]).abs() < 1e-4,
                "{o:?} vs {q:?}"
            );
        }
        // warp 0 with non-zero offsets is also identity.
        let corners = [[1.0, 1.0], [-1.0, 0.5], [0.0, -2.0], [3.0, 0.0]];
        let out0 = warp_positions(&p, &corners, 0.0, &[]);
        for (o, q) in out0.iter().zip(&p) {
            assert!((o[0] - q[0]).abs() < 1e-4 && (o[1] - q[1]).abs() < 1e-4);
        }
    }

    /// A corner element lands on its warped corner: the bbox corner maps to
    /// `corner + warp·offset`. Pull the top-left corner up-left and the element there
    /// follows.
    #[test]
    fn a_corner_element_follows_its_pinned_corner() {
        // A unit square layout; top-left offset by (−1, +1).
        let p = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let tl_off = [-1.0, 1.0];
        let corners = [tl_off, [0.0; 2], [0.0; 2], [0.0; 2]]; // TL only
        let out = warp_positions(&p, &corners, 1.0, &[]);
        // The top-left element (−1, 1) should move to (−1−1, 1+1) = (−2, 2).
        let tl_idx = 3; // (−1, 1)
        assert!(
            (out[tl_idx][0] + 2.0).abs() < 1e-3 && (out[tl_idx][1] - 2.0).abs() < 1e-3,
            "TL element pinned to (−2, 2): {:?}",
            out[tl_idx]
        );
    }

    /// Falloff masks the warp per element: falloff 0 leaves an element at its original
    /// position even under a strong warp; falloff 1 applies it fully.
    #[test]
    fn falloff_masks_the_warp() {
        let p = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let corners = [[-2.0, 2.0], [0.0; 2], [0.0; 2], [0.0; 2]];
        let falloff = vec![0.0, 1.0, 1.0, 0.0]; // corners 0 and 3 masked off
        let out = warp_positions(&p, &corners, 1.0, &falloff);
        assert_eq!(out[3], p[3], "falloff 0 → unchanged");
    }

    /// Deterministic + cooks through the registry, copying columns and warping P.
    #[test]
    fn registers_and_warps_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.four_point_warp.test.src"),
            name: "motion.four_point_warp.test.src",
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
                    Stream::new(4)
                        .with(
                            "P",
                            Column::Vec2(vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]),
                        )
                        .with("size", Column::Vec2(vec![[0.4, 0.4]; 4])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionFourPointWarp),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.four_point_warp.test.src");
        let fpw = g.add_node("motion.four_point_warp");
        g.set_param(fpw, "tl_dx", -1.0);
        g.set_param(fpw, "tl_dy", 1.0);
        g.connect(Edge {
            from: (src, 0),
            to: (fpw, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, fpw, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("size").is_some(), "columns pass through");
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert!(v[3][0] < -1.5, "the TL corner warped out: {:?}", v[3]),
            _ => panic!("P"),
        }
    }
}

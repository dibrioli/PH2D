#![forbid(unsafe_code)]
//! `motion.distribute_curve` — **place instances evenly along a curve**: the Blender
//! "Curve to Points" (Motion Nodes M3, distributions — doc 01 §3 / doc 28). The path
//! counterpart to the grid/radial/spiral distributions: a cubic Bézier authored by four
//! control-point params, sampled at **even arc-length** so the dots are evenly spaced
//! along the *length* of the curve (not the parameter `t`, which bunches on the tight
//! bends). An `offset` **value** input slides them along the arc (wrapping), so a
//! `value.lfo` makes a marquee flow down the path.
//!
//! **Self-contained by design:** the curve lives in the node's own params (not the
//! vector document), so this stays a pure `ph2d-nodegraph` drop-crate — the vector-fed
//! `motion.distribute-path` is a separate, later, cross-module node. The arc-length
//! machinery is in the sibling `curve.rs` (Bézier + cumulative-length LUT). Transcendental-
//! free (HR-5): polynomial curve + `sqrt` chord lengths + the Rajan `atan2` (`trig.rs`).
//! `Effect::Pure` (the offset animation arrives through the value input).
//!
//! ## `align` — the instance turns to face the way the curve is going
//!
//! Placing a point is half of walking a curve; the other half is **which way it looks**, and it is
//! the half every reference ships (Blender *Curve to Points* hands back Tangent · Normal ·
//! Rotation; a Cavalry Duplicator over a Path distribution orients its copies). Nothing downstream
//! could supply it: `motion.look_at` aims at a **target point**, and no node in the catalogue
//! differentiates a cubic — so this was a **hole**, not a choice. The sibling that walks a DRAWN
//! shape (`motion.path`) has had it since it landed, and two nodes that do the same distribution
//! disagreeing about orienting is the divergence this closes.
//!
//! ⚠️ The rotation is the **analytic** tangent at the same `t` the point was evaluated at — never a
//! chord between neighbours, which would make the angle a fact of the *count* instead of a fact of
//! the *curve* (and would leave the last instance with no successor to differ against).
//!
//! **Default off** (`align = 0`): a graph that never heard of this param emits `P` and nothing
//! else, byte for byte, exactly as it did before.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod curve;
mod trig;
use curve::{P2, arc_lut, eval, t_at_arclen, tangent};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `offset` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.distribute_curve"),
    name: "motion.distribute_curve",
    inputs: &[
        // Slides every point along the arc (wraps 0..1), animatable. Optional:
        // unconnected reads as 0.
        PortSpec {
            name: "offset",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // count + the four control points (world units). Default: a gentle S-curve.
    params: &[
        ParamSpec {
            name: "count",
            default: 32.0,
        },
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
        // 1 = turn each instance to face the way the curve is going (the `rot` column, in
        // degrees). APPENDED, and 0 by default: a saved graph reads it as absent and cooks
        // byte-identically.
        ParamSpec {
            name: "align",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Sample `count` points at even arc-length along the Bézier `cp`, all slid by `offset`
/// (wrapping). Cell-centred (`(i+0.5)/count`) so the set stays symmetric as it flows.
///
/// With `align`, each point also reports the **heading of the curve there**, in degrees — the
/// analytic derivative at the *same* `t` the point came from, so the two can never describe
/// different places on the curve. Without it the second vector is empty and the position
/// expression is the one that always shipped, verbatim.
fn distribute(cp: &[P2; 4], count: usize, offset: f32, align: bool) -> (Vec<P2>, Vec<f32>) {
    if count == 0 {
        return (Vec::new(), Vec::new());
    }
    let lut = arc_lut(cp);
    let mut pos = Vec::with_capacity(count);
    let mut rot = Vec::with_capacity(if align { count } else { 0 });
    for i in 0..count {
        let s = ((i as f32 + 0.5) / count as f32 + offset).rem_euclid(1.0);
        let t = t_at_arclen(&lut, s);
        pos.push(eval(cp, t));
        if align {
            let d = tangent(cp, t);
            rot.push(trig::deg(trig::atan2_approx(d[1], d[0])));
        }
    }
    (pos, rot)
}

struct MotionDistributeCurve;

impl NodeOp for MotionDistributeCurve {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let cp = [
            [ctx.param("p0x"), ctx.param("p0y")],
            [ctx.param("p1x"), ctx.param("p1y")],
            [ctx.param("p2x"), ctx.param("p2y")],
            [ctx.param("p3x"), ctx.param("p3y")],
        ];
        let offset = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let align = ctx.param("align") >= 0.5;
        let (positions, rot) = distribute(&cp, count, offset, align);
        let mut out = Stream::new(positions.len()).with("P", Column::Vec2(positions));
        if align {
            out = out.with("rot", Column::Scalar(rot));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDistributeCurve))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Curve Points",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `count` — MEDIDO** (doc 88 A1 · §0). Distribuir ao longo de uma curva é um
/// laço linear, e o cook
/// mediu pela porta do produto (`measure_the_count_ceiling`), enquanto o slider fica nos **320**
/// onde a mão trabalha (default 32 ⇒ ~2 instâncias por pixel de arrasto):
///
/// | instâncias | cook |
/// |---|---|
/// | 100.000 | 0,988 ms |
/// | 400.000 | 3,936 ms |
/// | **1.000.000** | **9,826 ms** |
///
/// Um milhão de pontos custa **59% de um quadro de 60 fps** — caro, e ainda assim **3.125×** o
/// que o slider alcança. É o dobro do custo dos irmãos lineares (grade, fibonacci, radial) na mesma
/// contagem, porque cada ponto paga uma avaliação de curva.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "count",
    max: 1_000_000.0,
}];

/// As SEÇÕES deste nó (doc 88 B3). O mesmo corte do `motion.spline_wrap`, e com o mesmo nome
/// de propósito: as oito coordenadas são o polígono de controle de uma cúbica nos dois nós, e
/// dois títulos para o mesmo objeto ensinariam que são coisas diferentes.
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("p0x", "Curve"),
    ParamGroup::new("p0y", "Curve"),
    ParamGroup::new("p1x", "Curve"),
    ParamGroup::new("p1y", "Curve"),
    ParamGroup::new("p2x", "Curve"),
    ParamGroup::new("p2y", "Curve"),
    ParamGroup::new("p3x", "Curve"),
    ParamGroup::new("p3y", "Curve"),
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 320.0,
        step: 1.0,
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
    // ⚠️ "Align To **Curve**", not the sibling's "Align To Path": the two nodes walk different
    // objects, and this node's own vocabulary for its object is `Curve` (the group name it
    // shares with `motion.spline_wrap`). A label naming the wrong object is worse than a
    // label that differs from a cousin's.
    ParamUiHint {
        param: "align",
        label: "Align To Curve",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "p0x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p0y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p1x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p1y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p2x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p2y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p3x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p3y",
        unit: ParamUnit::Length,
    },
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

    const LINE: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];

    /// The exact count is emitted, and on a straight line the points are evenly spaced
    /// (equal chords). FALSIFIED by parameter-space sampling, which bunches at the ends.
    #[test]
    fn count_is_exact_and_evenly_spaced_on_a_line() {
        let pts = distribute(&LINE, 6, 0.0, false).0;
        assert_eq!(pts.len(), 6);
        let gaps: Vec<f32> = pts.windows(2).map(|w| w[1][0] - w[0][0]).collect();
        let g0 = gaps[0];
        for g in &gaps {
            assert!((g - g0).abs() < 2e-2, "even gaps: {gaps:?}");
        }
    }

    /// Every point lies ON the curve — for the S-curve default, each sample equals a
    /// Bézier evaluation (no drift). A crude on-curve check: the y at the sampled x is
    /// consistent with the curve.
    #[test]
    fn points_sit_on_the_curve() {
        let cp = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
        let pts = distribute(&cp, 10, 0.0, false).0;
        for p in &pts {
            // Find the nearest LUT-sampled curve point; it should be within the sampling
            // resolution (the point came from an eval, so this is tight).
            let near = (0..=64)
                .map(|k| eval(&cp, k as f32 / 64.0))
                .map(|q| {
                    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                    dx * dx + dy * dy
                })
                .fold(f32::MAX, f32::min);
            assert!(near < 0.05, "on the curve (nearest² {near})");
        }
    }

    /// `offset` slides the points along the arc. A small offset (no wrap-around) shifts
    /// every point uniformly along the length-3 line: `0.05 · 3 = +0.15` in x. FALSIFIED
    /// by a dead offset (identical sets).
    #[test]
    fn offset_slides_along_the_arc() {
        let a = distribute(&LINE, 6, 0.0, false).0;
        let b = distribute(&LINE, 6, 0.05, false).0;
        for (pa, pb) in a.iter().zip(&b) {
            assert!(
                (pb[0] - pa[0] - 0.15).abs() < 2e-2,
                "slid +0.15: {pa:?} {pb:?}"
            );
        }
    }

    /// The S-curve of the defaults — it turns hard both ways, which is what makes it able to
    /// tell a per-point heading from a single shared one.
    const S: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];

    /// **The heading is the direction the curve travels there** — and the oracle is the curve's
    /// own SHAPE, not the tangent formula: the centred difference `p[i+1] − p[i−1]` approximates
    /// the direction at `i` to second order, and it is computed with the standard-library
    /// `atan2` in degrees, so it shares no code with the thing under test.
    ///
    /// FALSIFIED by: one shared angle for every point · swapped `atan2` arguments · radians on
    /// the wire · the tangent read at the arc fraction `s` instead of the curve parameter `t`.
    #[test]
    fn the_heading_matches_the_direction_the_curve_travels() {
        let (pos, rot) = distribute(&S, 64, 0.0, true);
        assert_eq!(rot.len(), 64, "one heading per point");

        let mut worst = 0.0f32;
        for i in 1..pos.len() - 1 {
            let (dx, dy) = (pos[i + 1][0] - pos[i - 1][0], pos[i + 1][1] - pos[i - 1][1]);
            let want = dy.atan2(dx).to_degrees();
            let mut err = (rot[i] - want).abs();
            if err > 180.0 {
                err = 360.0 - err; // the seam is a wrap, not a disagreement
            }
            worst = worst.max(err);
        }
        assert!(worst < 1.0, "worst heading error {worst} deg");
    }

    /// Degrees, and the axis that says so: on a straight line UP every instance reads **+90**.
    /// A horizontal line reads 0 — which is also what a dead `align` and a radian wire read,
    /// so the vertical half is the one with teeth.
    #[test]
    fn a_vertical_line_reads_ninety_degrees() {
        let up = [[0.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, 3.0]];
        for r in distribute(&up, 5, 0.0, true).1 {
            assert!((r - 90.0).abs() < 0.2, "up is +90 deg, got {r}");
        }
        for r in distribute(&LINE, 5, 0.0, true).1 {
            assert!(r.abs() < 0.2, "+x is 0 deg, got {r}");
        }
    }

    /// **Aligning does not move a single point** — byte for byte, on a curve chosen for being
    /// hard to sample. The heading is something the node *also* reports, never something that
    /// re-decides where an instance goes.
    #[test]
    fn aligning_does_not_move_a_single_point() {
        let plain = distribute(&S, 33, 0.17, false);
        let aligned = distribute(&S, 33, 0.17, true);
        assert_eq!(plain.0, aligned.0, "positions are untouched by align");
        assert!(plain.1.is_empty(), "align off reports no heading");
    }

    /// **A graph that never heard of `align` emits `P` and nothing else.** It does not name the
    /// default — it exercises it, which is the only way a default is actually under test.
    #[test]
    fn a_graph_that_never_heard_of_align_emits_no_rotation() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionDistributeCurve as &dyn NodeOp)
            }
        }
        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_curve");
        g.set_param(n, "count", 8.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("P").is_some(), "still places points");
        assert!(s.get("rot").is_none(), "and reports no heading");
    }

    /// Deterministic + cooks through the registry, emitting `P` at the exact count.
    #[test]
    fn registers_and_cooks() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionDistributeCurve as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_curve");
        g.set_param(n, "count", 20.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 20),
            _ => panic!("P"),
        }
    }
}

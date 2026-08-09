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
//! free (HR-5): polynomial curve + `sqrt` chord lengths, no trig. `Effect::Pure` (the
//! offset animation arrives through the value input).

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
use curve::{P2, arc_lut, eval, t_at_arclen};

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
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Sample `count` points at even arc-length along the Bézier `cp`, all slid by `offset`
/// (wrapping). Cell-centred (`(i+0.5)/count`) so the set stays symmetric as it flows.
fn distribute(cp: &[P2; 4], count: usize, offset: f32) -> Vec<P2> {
    if count == 0 {
        return Vec::new();
    }
    let lut = arc_lut(cp);
    (0..count)
        .map(|i| {
            let s = ((i as f32 + 0.5) / count as f32 + offset).rem_euclid(1.0);
            eval(cp, t_at_arclen(&lut, s))
        })
        .collect()
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
        let positions = distribute(&cp, count, offset);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
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
        let pts = distribute(&LINE, 6, 0.0);
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
        let pts = distribute(&cp, 10, 0.0);
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
        let a = distribute(&LINE, 6, 0.0);
        let b = distribute(&LINE, 6, 0.05);
        for (pa, pb) in a.iter().zip(&b) {
            assert!(
                (pb[0] - pa[0] - 0.15).abs() < 2e-2,
                "slid +0.15: {pa:?} {pb:?}"
            );
        }
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

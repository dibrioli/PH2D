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
//!
//! ## `mode` — ask for a COUNT, or ask for a SPACING
//!
//! *"Thirty dots along this curve"* and *"a dot every 25 cm"* are different questions, and only
//! the first was askable. Blender's *Curve to Points* has had the pair since it landed
//! (`Mode: Evaluated / Count / Length`), and nothing downstream could supply the second: `count`
//! was the only knob, and no node re-spaces a set without re-sampling the curve it came from.
//!
//! ⚠️ **The mode does not change WHERE the points go — it changes which number decides HOW MANY.**
//! The layout law is untouched (even arc-length, cell-centred, wrapping); `Length` divides the
//! curve's measured length by the asked spacing and hands the result to the same loop. That is why
//! `Count` is byte-identical by CONSTRUCTION rather than by a promise: it is the same call.
//!
//! ⚠️ **A curve's length is rarely a whole multiple of the spacing**, so the count is the one whose
//! ACTUAL spacing lands closest to what was asked (`round`, not `floor`). Flooring would bias every
//! request the same way — ask for `0.5` on a `3.4` curve and get `0.57` — while rounding is off by
//! at most half a step in either direction.
//!
//! ⚠️ **The two modes share ONE guard, and it is the substrate's** (`RECOMMENDED_MAX_ELEMENTS`) —
//! not the ceiling in [`PARAM_HARD_MAX`]. The first draft of this wave clamped the derived count at
//! the measured typed-ceiling, arguing *"a second door must not skip the measurement"*; reading it
//! back against the code, the two numbers answer different questions and conflating them would have
//! silently NARROWED `count`, which nobody asked for. `PARAM_HARD_MAX` is how far the artist's TEXT
//! BOX reaches; `RECOMMENDED_MAX_ELEMENTS` is what the layout may allocate. `Length` has no text box
//! for a count at all, so there was no second door to the first number.
//!
//! ⚠️ And the floor is the `spacing` **slider's own min**, which costs no new number: a box with no
//! [`ParamHardMin`] entry clamps at it, so `spacing = 0` is not reachable by authoring. A graph
//! loaded from TEXT can still carry one — it asks for infinitely many points and gets the element
//! cap, exactly as a `count` of `1e9` already did, so the two modes stay the same shape.
//!
//! **Default `mode = Count`**: a graph that never heard of either param reads them as absent, takes
//! the defaults, and lays out exactly the set it always laid out.

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
use curve::{P2, arc_lut, eval, t_at_arclen, tangent, total_len};

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
        // APPENDED. `0` = Count, the rule that always shipped: a saved graph reads both as
        // absent and lays out the same set.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // The arc distance between neighbours, in world units — read only in `Length`.
        // **0.25 is measured, not chosen**: the default curve is 7.30 units long, so it lands
        // 29 points next to the `count` default of 32. Flipping the mode on an untouched node
        // is a nudge in density, not a jump.
        ParamSpec {
            name: "spacing",
            default: 0.25,
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

/// **How many points the artist asked for** — the one door the two modes come through.
///
/// `Count` reads the number; `Length` divides the curve's measured length ([`total_len`]) by the
/// asked spacing and ROUNDS, so the actual spacing lands as close to the request as a whole number
/// of points allows (module docs). Both end at the SAME guard, `RECOMMENDED_MAX_ELEMENTS`.
///
/// ⚠️ The `Count` arm is `param_as_count` with the argument it always had, so a graph in `Count`
/// does not merely *behave* the same — it runs the same call.
///
/// Totally defined on hostile input, because a param is an `f32` an artist can type into: `as
/// usize` in Rust saturates and maps NaN to `0`, so a zero/NaN/negative `spacing` lands on the
/// clamp instead of on a panic.
fn resolve_count(cp: &[P2; 4], mode: f32, count: f32, spacing: f32) -> usize {
    if mode.round() as i32 != 1 {
        return param_as_count(count, RECOMMENDED_MAX_ELEMENTS);
    }
    let n = (total_len(&arc_lut(cp)) / spacing).round() as usize;
    n.clamp(1, RECOMMENDED_MAX_ELEMENTS)
}

struct MotionDistributeCurve;

impl NodeOp for MotionDistributeCurve {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let cp = [
            [ctx.param("p0x"), ctx.param("p0y")],
            [ctx.param("p1x"), ctx.param("p1y")],
            [ctx.param("p2x"), ctx.param("p2y")],
            [ctx.param("p3x"), ctx.param("p3y")],
        ];
        let count = resolve_count(
            &cp,
            ctx.param("mode"),
            ctx.param("count"),
            ctx.param("spacing"),
        );
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
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
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
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Count", "Length"],
        },
    },
    // ⚠️ The slider's `min` IS this param's floor — there is no `ParamHardMin` entry, so the box
    // clamps here too, and a spacing of zero is unreachable by authoring (module docs). `0.01` on
    // the 7.30-unit default curve is 730 points; the drag range tops out where a spacing stops
    // being a spacing and becomes "one point".
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.01,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **Each mode shows only the number it reads.** `count` in `Count`, `spacing` in `Length` —
/// the other would be a knob the layout provably ignores, which is the dead control this
/// side-channel exists to prevent.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "count",
        when: "mode",
        values: &[0],
    },
    ph2d_node_registry::ParamGate {
        param: "spacing",
        when: "mode",
        values: &[1],
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
    // A world DISTANCE along the arc, by the same trace as the control points: it is
    // divided into the length they describe.
    ParamUnitDecl {
        param: "spacing",
        unit: ParamUnit::Length,
    },
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
#[path = "lib_tests.rs"]
mod tests;

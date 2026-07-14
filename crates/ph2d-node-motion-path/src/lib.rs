#![forbid(unsafe_code)]
//! `motion.path` — **walk a shape the artist DREW** (Motion Nodes M3 / doc 65).
//!
//! Place `count` instances at even arc-length along a **vector path**, slide them with `offset`,
//! and (optionally) turn them to face the way the curve is going. It is the vector document's
//! `motion.distribute_curve`: same distribution, but the curve is a real drawn shape instead of
//! four control-point params.
//!
//! ## The reason this node was blocked for two journeys
//!
//! The plan said *"integra `vector.*`"* — and that node family was **RETIRED** (ADR-0108). The
//! geometry moved into `ph2d-vec-scene`, a document the graph has **no reach into**: a node is
//! handed its params, its inputs and the playhead, and nothing else. That is not an oversight, it
//! is the property that lets the cook memoize, scrub and replay bit-exactly.
//!
//! So the question was never *"how do I import a curve"*. It was **"how does anything the app owns
//! get into the graph at all"** — and the answer is the [external channel](ph2d_nodegraph::external)
//! (doc 65): the app publishes named values into the `Cook`, the node reads one by name.
//!
//! **The name is the artist's.** The shell publishes every vector shape under the name it carries in
//! the Hierarchy — so the gesture is: draw a curve, call it `Track`, and type `Track` into this
//! node. There is no id to copy, no picker to build, and no second place for the two to disagree
//! about which shape they mean. Rename the shape and the node follows it — because the name IS the
//! reference, not a lookup into one.
//!
//! HR-5: arc-length is `sqrt` (sibling `arc.rs`), the tangent's angle is the Rajan `atan2`
//! (`trig.rs`). `Effect::Pure` — the layout is a pure function of the curve, the params, and
//! nothing else; the curve's own revision rides in the cook's fingerprint, so editing the shape
//! re-cooks this node and *only* the nodes downstream of it.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod arc;
mod trig;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The **text** param naming the shape (the channel doc 32 opened — a `ParamSpec` is f32-only, and
/// the manifest is frozen, so a string param lives in the `Graph` beside the manifest, not in it).
const PATH_PARAM: &str = "path";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.path"),
    name: "motion.path",
    inputs: &[
        // An OPTIONAL value input for the offset — so a `value.lfo` makes the set flow down the
        // curve. Same shape as `motion.distribute_curve`'s: the animation arrives through a wire,
        // which is why the node itself can stay `Pure`.
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
    params: &[
        ParamSpec {
            name: "count",
            default: 24.0,
        },
        // Slides the whole set along the arc, WRAPPING — a curve is a thing to walk around, not a
        // line to fall off the end of.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // 1 = turn each instance to face the way the curve is going (the `rot` column, in degrees).
        ParamSpec {
            name: "align",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The published curve, as a polyline. The shell flattens the drawn shape **into world space** and
/// publishes its points as `P` — the graph never sees a Bézier, and does not need to.
fn polyline(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

struct MotionPath;

impl NodeOp for MotionPath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let align = ctx.param("align") >= 0.5;
        // The offset is the param PLUS whatever a wire is putting on the `offset` input — one
        // number, so an LFO makes the set flow. (A stream with no value reads as 0.)
        let wired = match ctx.input(0).get(ph2d_nodegraph::attr::VALUE_COLUMN) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let offset = ctx.param("offset") + wired;

        let name = ctx.text_param(PATH_PARAM).unwrap_or_default().to_string();
        let pts = polyline(ctx.external(&name));
        let lut = arc::lut(&pts);

        // A shape that is not there (not drawn yet, renamed, deleted) is an EMPTY stream — the same
        // thing an unconnected input is. A node that cannot find its curve emits nothing; it does
        // not guess, and it does not fail.
        if lut.is_empty() || count == 0 {
            ctx.emit(Stream::new(0));
            return;
        }

        let mut pos = Vec::with_capacity(count);
        let mut rot = Vec::with_capacity(count);
        for i in 0..count {
            // Even ARC-LENGTH, not even parameter: sampling by parameter bunches the points on the
            // tight bends, and the eye reads that as "not even".
            let s = i as f32 / count as f32 + offset;
            let (p, tangent) = arc::at(&pts, &lut, s);
            pos.push(p);
            if align {
                rot.push(trig::deg(trig::atan2_approx(tangent[1], tangent[0])));
            }
        }

        let mut out = Stream::new(count).with("P", Column::Vec2(pos));
        if align {
            out = out.with("rot", Column::Scalar(rot));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionPath))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Path",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints (M1.P1). The **path name** is a `ParamWidget::Text` — the same widget the
/// expression node's formula uses, and the same channel underneath (doc 32).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: PATH_PARAM,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 500.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -1.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "align",
        label: "Align To Path",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

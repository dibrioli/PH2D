#![forbid(unsafe_code)]
//! `fx.glow` — the Motion module's HDR bloom, authored as a **node** (doc 67).
//!
//! ## Why a node, and not a document field or a panel
//!
//! The glow is a *pass* effect — it runs on the whole rendered Motion image, not
//! on one instance stream. The plan called for the document to "declare
//! `layer_fx`", and the graph **is** the document: a node IS a declaration in it.
//! Making the glow a node buys everything for free and keeps the module honest to
//! its own idiom:
//!
//! - **Authoring:** the existing params panel edits it the moment it is selected —
//!   no new gotcha-prone UI (a doc-level panel section is where this codebase
//!   bleeds time: the 1-px click drift, painted-≠-populated, a slider no test
//!   clicks). Its four knobs come with [`ParamUiHint`]s so the panel renders the
//!   right sliders and ranges.
//! - **Persistence / undo / driven params:** all of it comes from the graph
//!   infrastructure it already lives in. A `set_param` is a normal undo step; the
//!   textual format already serialises it.
//!
//! ## It configures the pass — it does not scope it
//!
//! `fx.glow` is a **passthrough** (`out == in`, `Effect::Pure`): dropping it in a
//! chain changes nothing about the stream, and leaving it unwired is fine too. The
//! shell finds it with [`from_graph`] and reads its params to drive
//! `ph2d_render::MotionFx` — the glow always applies to the whole Motion image
//! regardless of where the node sits or what (if anything) feeds it. Placement is
//! for readability; it never limits the effect. Zero glow nodes → the pass never
//! runs and the frame is byte-identical (the neutral point).
//!
//! Transcendental-free (HR-5): the node itself does no math — it forwards its
//! input and carries four numbers the render pass reads.

use ph2d_node_registry::{
    NodeRegistry, NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamUiHint, ParamWidget,
    RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Canonical node type — the shell matches on this when scanning the graph.
pub const TYPE_NAME: &str = "fx.glow";

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.glow"),
    name: "fx.glow",
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
        // Brightness above which a pixel glows (premult max(r,g,b)). 1.0 = only
        // genuinely HDR (emissive) pixels — an LDR scene is left untouched.
        ParamSpec {
            name: "threshold",
            default: 1.0,
        },
        ParamSpec {
            name: "knee",
            default: 0.6,
        },
        ParamSpec {
            name: "intensity",
            default: 0.8,
        },
        ParamSpec {
            name: "radius",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The bloom settings the shell reads to drive the Motion glow pass. Mirrors
/// `ph2d_render::BloomParams` (this crate has no render dep — the shell converts
/// at the boundary).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Glow {
    pub threshold: f32,
    pub knee: f32,
    pub intensity: f32,
    pub radius: f32,
}

/// The manifest default for a param name (the single source of a knob's neutral
/// value — the reader never hard-codes a second copy).
fn default_of(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.default)
        .unwrap_or(0.0)
}

/// A param's effective value: the graph override if the artist set it, else the
/// manifest default.
fn read(overrides: Option<&BTreeMap<String, f32>>, name: &str) -> f32 {
    overrides
        .and_then(|m| m.get(name))
        .copied()
        .unwrap_or_else(|| default_of(name))
}

/// Read the glow settings from the graph: the **first** `fx.glow` node's
/// (override-or-default) params. `None` when there is no glow node — the pass
/// does not run and the frame is byte-identical to no FX.
pub fn from_graph(graph: &Graph) -> Option<Glow> {
    let node = graph.nodes().iter().find(|n| n.type_name == TYPE_NAME)?.id;
    let ov = graph.node_param_overrides(node);
    Some(Glow {
        threshold: read(ov, "threshold"),
        knee: read(ov, "knee"),
        intensity: read(ov, "intensity"),
        radius: read(ov, "radius"),
    })
}

struct FxGlow;

impl NodeOp for FxGlow {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Passthrough: the node carries settings, it does not transform the
        // stream. Forwarding the input verbatim keeps `out == in` byte-identical,
        // so wiring it into a chain is always safe and never changes the cook.
        let out = ctx.input(0).clone();
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxGlow))?;
    reg.register_ui(
        MANIFEST.id,
        NodeUiManifest {
            display_name: "Glow",
            category: NodeUiCategory::Fx,
            silhouette: NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "threshold",
        label: "Threshold",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "knee",
        label: "Soft Knee",
        min: 0.0,
        max: 2.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "intensity",
        label: "Intensity",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.25,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// A source that emits a fixed two-instance stream — the "before" the glow
    /// passthrough must reproduce exactly.
    struct Source;
    const SOURCE: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.source"),
        name: "test.source",
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
    fn fixture() -> Stream {
        Stream::new(2)
            .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
            .with(
                "tint",
                Column::Vec4(vec![[6.0, 4.0, 2.0, 1.0], [1.0, 1.0, 1.0, 1.0]]),
            )
    }
    impl NodeOp for Source {
        fn manifest(&self) -> &'static NodeManifest {
            &SOURCE
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(fixture());
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == MANIFEST.id {
                Some(&FxGlow as &dyn NodeOp)
            } else if ty == SOURCE.id {
                Some(&Source as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    /// **The invariant that lets `fx.glow` live anywhere: it is byte-identical in
    /// the cook.** `source → fx.glow` produces exactly what `source` alone does —
    /// so dropping the node in a chain never changes the stream, and the glow it
    /// configures is a pure addition on the render side.
    #[test]
    fn glow_is_a_byte_identical_passthrough() {
        let mut g = Graph::new();
        let src = g.add_node("test.source");
        let glow = g.add_node(TYPE_NAME);
        g.connect(Edge {
            from: (src, 0),
            to: (glow, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let through = cook.cook(&g, &Ops, glow, 0.0).unwrap()[0]
            .as_stream()
            .clone();
        assert_eq!(
            through.get("P"),
            fixture().get("P"),
            "the glow node must not move a point"
        );
        assert_eq!(
            through.get("tint"),
            fixture().get("tint"),
            "the glow node must not touch a colour (the HDR tint survives verbatim)"
        );
    }

    #[test]
    fn no_glow_node_reads_none() {
        // The neutral point: a graph without the node never runs the pass.
        let g = Graph::new();
        assert_eq!(from_graph(&g), None);
    }

    #[test]
    fn reads_defaults_then_overrides() {
        let mut g = Graph::new();
        let n = g.add_node(TYPE_NAME);
        // Untouched → manifest defaults.
        assert_eq!(
            from_graph(&g),
            Some(Glow {
                threshold: 1.0,
                knee: 0.6,
                intensity: 0.8,
                radius: 1.0,
            })
        );
        // A dragged slider overrides just that knob; the rest stay at default.
        g.set_param(n, "intensity", 2.5);
        g.set_param(n, "threshold", 1.5);
        let glow = from_graph(&g).unwrap();
        assert_eq!(glow.intensity, 2.5);
        assert_eq!(glow.threshold, 1.5);
        assert_eq!(glow.radius, 1.0, "untouched knob keeps its default");
    }

    #[test]
    fn passthrough_default_matches_the_manifest_param_count() {
        // Four authored knobs — the render pass reads exactly these.
        assert_eq!(MANIFEST.params.len(), 4);
        assert_eq!(default_of("knee"), 0.6);
    }
}

#![forbid(unsafe_code)]
//! `source.object` — **bring an engine object into the graph** (doc 86 §2).
//!
//! The graph produces WHERE (a stream of positions); it never had a way to say
//! WHAT — no sprite, no vector, no Flip could be the thing an instancer stamps.
//! This node is that door: it names an object the app owns, and emits **one
//! instance carrying that object's appearance** — its `texture_id` (which
//! texture), `uv_rect` (its atlas cell), `size` and `tint`. Cross it with a
//! `motion.grid` through a `motion.duplicator` and the object is stamped at
//! every point.
//!
//! ## The same door `motion.path` uses, for the same reason
//!
//! A node is handed its params, its inputs and the playhead — and nothing else
//! (the property that lets the cook memoize and replay bit-exactly). So it
//! **cannot reach into the ECS**. The app publishes the object under its **name**
//! into the cook's [external channel](ph2d_nodegraph::external), and this node
//! reads that name — exactly as `motion.path` reads a drawn curve. The membrane
//! (`shells/desktop/.../motion_bridge_objects.rs`) resolves *sprite → tile
//! directly*, and *vector/Flip/group → a baked tile* (later waves); this node is
//! **media-agnostic** — it reads whatever stream the membrane published, so a
//! new media type grows the membrane, never this node.
//!
//! **The name is the artist's, and it is the whole reference.** Name a sprite
//! `Ball` in the Hierarchy and type `Ball` here — no id to copy, nowhere for the
//! two to disagree. Rename the object and the node follows it, because the name
//! IS the reference. An object that is not there (unnamed, deleted, not yet
//! resolvable) is an **empty external** → an empty stream: the node emits
//! nothing, it does not guess and it does not fail.
//!
//! `Effect::Pure` — the output is a pure function of the named external, whose
//! own content-revision rides in the cook's fingerprint, so moving/retinting the
//! object re-cooks this node and only what is downstream of it.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The **text** param naming the object (the channel doc 32 opened — a
/// `ParamSpec` is f32-only and the manifest is frozen, so a string param lives
/// in the `Graph` beside the manifest, not in it; the same slot `motion.path`
/// uses for its `path` name).
const OBJECT_PARAM: &str = "object";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.object"),
    name: "source.object",
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

struct SourceObject;

impl NodeOp for SourceObject {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let name = ctx.text_param(OBJECT_PARAM).unwrap_or_default().to_string();
        // The membrane published `(P, size, tint, uv_rect, texture_id)` under
        // this name. Clone is refcount, not a copy (columns are `Arc`); a name
        // with no published object is the empty external → an empty stream.
        let stream = ctx.external(&name).clone();
        ctx.emit(stream);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceObject))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Object",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Circle,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// One row: the object name, a `ParamWidget::Source` (a picker of the objects
/// the app has published, doc 65) with the raw text field as the escape.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: OBJECT_PARAM,
    label: "Object",
    min: 0.0,
    max: 0.0,
    step: 0.0,
    widget: ParamWidget::Source,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

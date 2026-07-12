#![forbid(unsafe_code)]
//! **`value.attribute`** — read a named COLUMN of the stream as a value field (Motion Nodes,
//! doc 50). Blender's *Named Attribute* node; Houdini's `@age`, `@id`, `@speed`.
//!
//! ## The glue that was missing
//!
//! The stream carries a dozen per-element columns — `age`, `life`, `id`, `size`, `inv_mass`,
//! whatever a node wrote — and until now **nothing could read one back out**. `value.lfo` mints a
//! global, `value.instance_field` mints a field from *identity* (index / ramp / hash), and both
//! stop there: a number an element already CARRIES was unreachable to the value graph.
//!
//! So "colour the sparks by how old they are" — the most ordinary sentence in motion graphics —
//! had no path through this library at all, however many nodes it had.
//!
//! One node fixes it, and it fixes it for every column at once: age, life, speed, mass, id,
//! anything anyone adds later. That is why it is a *named* attribute and not an enum of the
//! columns we happen to have today.
//!
//! ## The name is a TEXT param
//!
//! `NodeManifest.params` is `f32`-only and **frozen** (§6), so the column's name rides the
//! graph's text channel (`Graph::set_text_param` / `EvalCtx::text_param`) — the canonical pattern
//! for a non-`f32` param, established by `motion.expression` (doc 32). The panel renders it as a
//! text field. No contract was bumped to get a string into a node.
//!
//! ## A missing column is `0`, not a crash
//!
//! Reading an attribute nothing wrote yields zeros — the value field is still length-N, so
//! everything downstream keeps its shape. A node that errored would take the whole graph down
//! because an artist typed `ag` instead of `age`, and a node that emitted an EMPTY field would
//! silently broadcast a global zero into a per-element slot, which is worse: it looks like it
//! worked.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The text param carrying the column's name (read via `EvalCtx::text_param`).
pub const ATTR_KEY: &str = "attr";

/// 0 = the scalar column itself · 1 = the LENGTH of a Vec2 column (so `vel` reads as *speed*,
/// which is what an artist asking for "speed" means).
const MODE_LENGTH: i32 = 1;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.attribute"),
    name: "value.attribute",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Scalar (read a scalar column) · 1 Length (read a Vec2 column's magnitude).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The named column as a length-N field. Missing / mistyped → zeros (see the module docs).
fn field(s: &Stream, name: &str, mode: i32) -> Vec<f32> {
    let n = s.count();
    match (s.get(name), mode) {
        (Some(Column::Scalar(v)), m) if m != MODE_LENGTH && v.len() == n => v.clone(),
        (Some(Column::Vec2(v)), MODE_LENGTH) if v.len() == n => v
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt())
            .collect(),
        _ => vec![0.0; n],
    }
}

struct ValueAttribute;

impl NodeOp for ValueAttribute {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        let out = {
            let name = ctx.text_param(ATTR_KEY).unwrap_or("").to_string();
            let input = ctx.input(0);
            let v = field(input, &name, mode);
            Stream::new(v.len()).with("v", Column::Scalar(v))
        };
        ctx.emit(out);
    }
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: ATTR_KEY,
        label: "Attribute",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: "mode",
        label: "mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueAttribute))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Attribute",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

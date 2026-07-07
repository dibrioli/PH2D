#![forbid(unsafe_code)]

//! `ph2d-tool-vector` — the Vector drawing tool (ADR-0108 cutover, Fase R).
//!
//! The single first-class tool of the repositioned Vector Module: a
//! `vector_tools`-cluster pill that activates the `ph2d-vec-*` engine (scene /
//! edit / render / boolean). It replaces the five retired
//! `ph2d-tool-vector-*` tools (pen / pencil / shape / select / direct) with one
//! unified pen+edit tool.
//!
//! Per ADR-0040 (tool-as-isolated-feature-crate): the tool carries only its
//! **style model** (stroke / fill palette + width). The document (`VecScene`)
//! and interaction state (`PenTool`, `History`) live in the shell — document ≠
//! tool. The shell's `vector_bridge` downcasts this tool via
//! [`Tool::as_any_mut`] to read the current style (mirror of the Pen / Painter
//! tools).
//!
//! ## Smoke contract
//!
//! Click the VECTOR pill → the pen activates → the docked `ph2d-panel-vector`
//! Style panel appears (right dock) → click/drag on the canvas draws a Bézier
//! path → the panel's Width slider + Stroke / Fill colour swatches (OKLCH
//! picker) restyle new (and the selected) paths.

pub mod icon;
pub mod params;
pub mod tool;

pub use params::{
    DrawMode, StrokeCap, StrokeJoin, VectorDrawConfig, VectorStyleSnapshot, VertexType,
    px_to_slider, slider_to_px,
};
pub use tool::{
    DEFAULT_CORNER_RADIUS_PX, DEFAULT_POLYGON_SIDES, DEFAULT_STAR_INNER, DEFAULT_STAR_POINTS,
    DEFAULT_STROKE_WIDTH_PX, PALETTE, VectorTool,
};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector tool as a boxed trait object. Codegen target for
/// `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorTool::default())
}

/// Placeholder handler for the `ToolHandler::Stateful` slot — never runs; the
/// real semantics live in `impl Tool for VectorTool`.
fn noop_manifest_handler() {}

/// Tool manifest. Cluster `vector_tools`; `order = 10` (sole member of the
/// family after the cutover — the retired Pen/Pencil/Shape/Select/Direct held
/// 10/20/30/40/50).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector",
    label_key: "tool.vector.label",
    icon_fn: icon::vector_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 10,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // The tool holds only a few scalars of style; the document lives in the
    // shell. Reserve 1 MB against the HR-13 boot check.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Vector tool with a manifest `Registry`.
pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::floating_panel::ToolId;

    #[test]
    fn register_attaches_manifest() {
        let mut reg = Registry::default();
        register(&mut reg);
        assert_eq!(reg.manifests().len(), 1);
        assert_eq!(reg.manifests()[0].id, "vector");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key() {
        assert_eq!(MANIFEST.id, "vector");
        assert_eq!(MANIFEST.label_key, "tool.vector.label");
    }

    #[test]
    fn make_constructs_boxed_tool_with_vector_id() {
        assert_eq!(make().id(), ToolId::new("vector"));
    }

    #[test]
    fn vector_tool_is_not_the_editor_default() {
        assert!(!make().is_default());
    }

    #[test]
    fn icon_returns_non_empty_path() {
        assert!(!icon::vector_bezpath().elements().is_empty());
    }
}

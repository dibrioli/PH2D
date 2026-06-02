#![forbid(unsafe_code)]

//! `ph2d-tool-vector-shape` — Shape tool (rect / ellipse / polygon / star /
//! spiral), the third interactive tool of the Vector Module.
//!
//! Per plan **T2.2** (`docs/Vector Module/17_plano_de_implementacao.md` §5)
//! and ADR-0040 (tool-as-isolated-feature-crate). A drag generates a live
//! preview; pointer-up commits the configured primitive. The five
//! sub-modes are a [`RadioGroup`](ph2d_editor_core::widget::RadioGroup) in
//! the tool's panel (plan §5 "toggle no panel"). Geometry comes from
//! [`ph2d_vector_doc::primitives`] — the same generators the W3
//! `vector-source` node (T3.2) consolidates.
//!
//! ## Smoke W2 Day-8 contract
//!
//! Activate the SHAPE pill → pick a sub-mode in the panel → drag on the
//! canvas → a live preview tracks the drag → release commits the shape.
//! Closed shapes render filled; the spiral renders stroked.

pub mod icon;
pub mod tool;

pub use tool::{
    MAX_COORD_MAGNITUDE, MIN_DRAG_DIST_PX, ShapeKind, ShapeOutcome, VectorShapeTool,
};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector Shape tool as a boxed trait object. Codegen target
/// for `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorShapeTool::default())
}

/// Placeholder handler for the `ToolHandler::Stateful` slot — never runs;
/// the real semantics live in `impl Tool for VectorShapeTool`.
fn noop_manifest_handler() {}

/// Tool manifest. Cluster `vector_tools` (shared with Pen/Pencil). Order
/// `30`: third of the family (Pen `10`, Pencil `20`, Select `40` next).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector_shape",
    label_key: "tool.vector_shape.label",
    icon_fn: icon::vector_shape_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 30,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // A committed shape is small (≤ 128 vertices for polygons; a 64-turn
    // spiral at 64 spt is the outlier ~4 k vertices ≈ 64 KB live). Reserve
    // 1 MB against the HR-13 boot check.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Shape tool with a manifest `Registry`.
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
        assert_eq!(reg.manifests()[0].id, "vector_shape");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key_shape() {
        assert_eq!(MANIFEST.id, "vector_shape");
        assert_eq!(MANIFEST.label_key, "tool.vector_shape.label");
    }

    #[test]
    fn shape_order_follows_pencil() {
        assert_eq!(MANIFEST.order, 30);
    }

    #[test]
    fn make_constructs_boxed_tool_with_shape_id() {
        assert_eq!(make().id(), ToolId::new("vector_shape"));
    }

    #[test]
    fn shape_tool_is_not_the_editor_default() {
        assert!(!make().is_default());
    }

    #[test]
    fn icon_returns_non_empty_path() {
        assert!(!icon::vector_shape_bezpath().elements().is_empty());
    }
}

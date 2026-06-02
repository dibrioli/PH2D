#![forbid(unsafe_code)]

//! `ph2d-tool-vector-direct` — Direct Select tool (vertex + tangent edit).
//!
//! Per plan **T2.3** + ADR-0040. Grabs a vertex or cubic tangent handle of
//! a committed network and drags it, applying event-sourced
//! [`VectorOp::MoveVertex`](ph2d_vector_doc::VectorOp) /
//! [`VectorOp::MoveTangent`](ph2d_vector_doc::VectorOp) ops to that asset's
//! edit log (replay-safe → T2.5 Undo). Alt-drag breaks a smooth tangent.
//! Shares the [`VectorSelection`](ph2d_vector_doc::VectorSelection) with
//! its sibling `ph2d-tool-vector-select` (both passed by-ref by the shell).

pub mod icon;
pub mod tool;

pub use tool::{
    DEFAULT_GRAB_TOLERANCE_PX, DirectGrab, DirectOutcome, GrabTarget, VectorDirectTool,
};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector Direct Select tool as a boxed trait object.
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorDirectTool::default())
}

fn noop_manifest_handler() {}

/// Tool manifest. Cluster `vector_tools`. Order `50` (Pen 10, Pencil 20,
/// Shape 30, Select 40, Direct 50).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector_direct",
    label_key: "tool.vector_direct.label",
    icon_fn: icon::vector_direct_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 50,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // Holds only a transient drag grab — negligible. 1 MB HR-13 reserve.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Direct Select tool with a manifest `Registry`.
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
        assert_eq!(reg.manifests()[0].id, "vector_direct");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key_shape() {
        assert_eq!(MANIFEST.id, "vector_direct");
        assert_eq!(MANIFEST.label_key, "tool.vector_direct.label");
    }

    #[test]
    fn direct_order_follows_select() {
        assert_eq!(MANIFEST.order, 50);
    }

    #[test]
    fn make_constructs_boxed_tool_with_direct_id() {
        assert_eq!(make().id(), ToolId::new("vector_direct"));
    }

    #[test]
    fn icon_returns_non_empty_path() {
        assert!(!icon::vector_direct_bezpath().elements().is_empty());
    }
}

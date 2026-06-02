#![forbid(unsafe_code)]

//! `ph2d-tool-vector-select` — network-level Select tool (click + marquee).
//!
//! Per plan **T2.3** + ADR-0040 (tool-as-isolated-feature-crate). Reads the
//! shared committed vector scene + mutates the shared
//! [`VectorSelection`](ph2d_vector_doc::VectorSelection) (both owned by the
//! shell, passed by-ref). Click picks the topmost filled region's network;
//! marquee picks by bounding box. Its sibling `ph2d-tool-vector-direct`
//! does vertex/tangent editing; both share the one selection.

pub mod icon;
pub mod tool;

pub use tool::{MarqueeMode, SelectOutcome, VectorSelectTool};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector Select tool as a boxed trait object.
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorSelectTool::default())
}

fn noop_manifest_handler() {}

/// Tool manifest. Cluster `vector_tools`. Order `40` (Pen 10, Pencil 20,
/// Shape 30, Select 40, Direct 50).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector_select",
    label_key: "tool.vector_select.label",
    icon_fn: icon::vector_select_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 40,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // Holds only a transient marquee rect — negligible. 1 MB HR-13 reserve.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Select tool with a manifest `Registry`.
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
        assert_eq!(reg.manifests()[0].id, "vector_select");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key_shape() {
        assert_eq!(MANIFEST.id, "vector_select");
        assert_eq!(MANIFEST.label_key, "tool.vector_select.label");
    }

    #[test]
    fn select_order_follows_shape() {
        assert_eq!(MANIFEST.order, 40);
    }

    #[test]
    fn make_constructs_boxed_tool_with_select_id() {
        assert_eq!(make().id(), ToolId::new("vector_select"));
    }

    #[test]
    fn icon_returns_non_empty_path() {
        assert!(!icon::vector_select_bezpath().elements().is_empty());
    }
}

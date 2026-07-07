#![forbid(unsafe_code)]

//! `ph2d-tool-motion` — the Motion Nodes tool (Motion Nodes M0.T9).
//!
//! A `motion_tools`-cluster pill (`IconId::MotionNodes`) that activates the
//! node-graph editor. While active the center region splits into scene ⟂ graph
//! (`CenterSplit`) and the docked `ph2d-panel-motion-graph` / `-params` panels
//! appear — all driven by the shell's `render_loop::motion_bridge`.
//!
//! Per ADR-0040 the tool is a thin activation handle: the document
//! (`MotionDoc`), the transport and the persistent `Cook` live in the shell's
//! `MotionState` (document ≠ tool). This crate ships the registered skeleton;
//! graph interaction + params rows land in M1.

pub mod icon;
pub mod tool;

pub use tool::MotionTool;

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Motion tool as a boxed trait object. Codegen target for
/// `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(MotionTool::default())
}

/// Placeholder handler for the `ToolHandler::Stateful` slot — never runs; the
/// real semantics live in `impl Tool for MotionTool` + the shell bridge.
fn noop_manifest_handler() {}

/// Tool manifest. Cluster `motion_tools` (sole member); `order = 10`.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "motion",
    label_key: "tool.motion.label",
    icon_fn: icon::motion_bezpath,
    zone: Zone::TopRight,
    cluster: "motion_tools",
    order: 10,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // Activation-only skeleton; the document lives in the shell. Reserve 1 MB
    // against the HR-13 boot check.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Motion tool with a manifest `Registry`.
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
        reg.build().expect("registry should build");
        assert!(reg.by_id("motion").is_some());
    }

    #[test]
    fn make_builds_motion_tool() {
        let t = make();
        assert_eq!(t.id(), ToolId::new("motion"));
    }

    #[test]
    fn manifest_id_matches_label_key_slug() {
        assert_eq!(MANIFEST.id, "motion");
        assert_eq!(MANIFEST.label_key, "tool.motion.label");
    }
}

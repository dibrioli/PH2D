//! Padding — [`ToolManifest`] declaration.
//!
//! Stateful tool: clicking `[Padding]` in the Image Tools row opens a
//! panel with four signed per-edge fields (top/right/bottom/left) and
//! arms gizmo edge-drag handles (the directional-expand UX); a live
//! preview shows the resized canvas; Apply bakes a new texture and
//! fixes up the sprite pivot/Transform. The pure resize is in
//! [`crate::algorithm`]; the panel + gizmo + shell bake are Coordinator
//! integration on a clean tree.
//!
//! ### Handler — shadow mode
//! Like every Image Tools manifest, the real handler runs shell-side
//! (it mutates `SimWorld` + swaps the sprite texture), so the manifest
//! `handler` slot points to no-op [`shadow_handler`]s until the generic
//! dispatcher (PR 9) lands. The manifest still registers so the
//! registry-derived TopBar pill + the HR-12/13/15 gates pick it up.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::padding_bezpath;

/// Shadow-mode handler trio. Real work happens in the desktop shell's
/// padding tool integration (Coordinator wiring).
fn shadow_handler() {}

/// The canonical Padding manifest.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "padding",
    label_key: "tool.padding.label",
    icon_fn: padding_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // Image Tools row order: trim 40 → make_square 50 → bgremoval 60 →
    // real_size 70 → padding 80.
    order: 80,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    // Stateful preview bakes a resized canvas (≈ source size + padding).
    // The Coordinator sets the accurate reserve when wiring the live
    // preview; the leaf manifest declares a conservative 0 (the shell
    // stages the output into the already-budgeted texture pool, like
    // make_square / trim).
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
        assert!(MANIFEST.label_key.starts_with("tool."));
        assert!(MANIFEST.label_key.ends_with(".label"));
        let stripped = MANIFEST
            .label_key
            .strip_prefix("tool.")
            .and_then(|s| s.strip_suffix(".label"))
            .expect("label_key shape");
        assert_eq!(stripped, MANIFEST.id);
    }

    #[test]
    fn manifest_lives_in_image_tools_cluster() {
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }

    #[test]
    fn manifest_order_is_right_of_real_size() {
        const { assert!(MANIFEST.order > 70) };
    }
}

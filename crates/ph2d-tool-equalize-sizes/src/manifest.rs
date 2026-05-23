//! Equalize Sizes — [`ToolManifest`] declaration.
//!
//! Stateful tool: clicking `[Equalize]` in the Image Tools row opens a
//! right-docked panel (target mode + W/H or grid unit + algorithm choice,
//! plus Apply/Cancel); Apply iterates `hero.gizmo.iter_selected()` and
//! bakes each sprite per the params. The pure cross-sprite math is in
//! [`crate::algorithm`]; the panel + shell wiring + commit are
//! Coordinator integration on a clean tree (not part of this fan-out
//! drop-crate).
//!
//! Cluster + order line up with `docs/design/tools/equalize_sizes.toml`
//! (gated by the `tool_manifest_design_sync` test in
//! `ph2d-tool-registry-init`).

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::equalize_sizes_bezpath;

/// Shadow-mode handler trio. Real dispatch happens shell-side: the
/// generic `EditorAction::ActivateTool { tool_id: "equalize_sizes" }`
/// arms the tool + opens the panel; the panel pushes
/// `EditorAction::ToolPanelEvent`s for edits; Apply pushes a generic
/// `OneShotImageOp { tool_id, entity_bits: primary }`, which the shell
/// resolves into a downcast call on `EqualizeSizesTool` that iterates
/// the full selection (bgremoval/padding template). The manifest still
/// registers so the registry-derived TopBar pill + the HR-12/13/15
/// gates pick it up.
fn shadow_handler() {}

/// The canonical Equalize Sizes manifest.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "equalize_sizes",
    label_key: "tool.equalize_sizes.label",
    icon_fn: equalize_sizes_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // Image Tools row order — Coord reserved 100 (right of
    // color_equalization at 90, left of rasterize at 110, upscale at 120).
    order: 100,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    // No global state outside the tool's own params struct (sub-KB);
    // the bake reads the host-provided per-sprite snapshots and writes
    // back via texture_edit (which is budgeted by the texture pool, not
    // this tool's budget). Conservative 0/0/0.
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
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
    fn manifest_order_sits_between_color_equalization_and_rasterize() {
        // color_equalization = 90, rasterize = 110.
        const { assert!(MANIFEST.order > 90 && MANIFEST.order < 110) };
    }
}

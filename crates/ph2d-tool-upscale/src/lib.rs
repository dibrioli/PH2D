#![forbid(unsafe_code)]
//! ph2d-tool-upscale — multi-sprite resolution upscaling, isolated crate.
//!
//! Stateful tool, active in the TopBar `image_tools` cluster. Opens a
//! docked panel (`ph2d-panel-upscale`) with a 3-way algorithm selector
//! (Lanczos3 / Nearest / xBR), a scale slider (1×–16×), and Apply /
//! Cancel. **Apply-only** — there is NO live preview overlay and NO
//! realtime feedback during slider drag (Wave 10 Etapa 2 follow-up).
//! Pixel work runs at full resolution only when the Apply button is
//! pressed; until then the canvas shows the original sprite untouched.
//!
//! ADR-0040 — this crate owns the whole feature: the [`MANIFEST`]
//! (data), [`UpscaleTool`] (behavior, `impl ph2d_editor_core::tool::Tool`),
//! the three pure [`algorithm`] entry points, [`icon`], and the
//! UI/action vocabulary in [`params`] (`UpscaleAlgorithm` /
//! `UpscaleUiEdit` / `UpscaleUiSnapshot` / `UpscaleParams` + scale
//! consts). Depends on `ph2d-editor-core` (the `Tool` / widget /
//! `FloatingPanel` contract — the satellite direction), never the
//! reverse.

pub mod algorithm;
pub mod icon;
pub mod params;
pub mod tool;

pub use tool::UpscaleTool;

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Shadow-mode handler trio. Real dispatch goes through the
/// `ToolRegistry` in `shells/desktop` and the [`UpscaleTool`] `Tool`
/// impl; the manifest still registers so the registry-derived TopBar
/// pill + the HR-12/13/15 gates pick it up.
fn shadow_handler() {}

/// The canonical Upscale manifest. Image Tools row, order 120
/// (rightmost — reserved by Coord 2026-05-23 pre-work).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "upscale",
    label_key: "tool.upscale.label",
    icon_fn: icon::upscale_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 120,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    // Apply-only model (Wave 10 Etapa 2 follow-up): no preview buffers
    // — the full-res bake stages into the existing texture pool (same
    // as the other Image Tools), so the leaf manifest declares 0
    // (host-side pool is already budgeted).
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Upscale manifest with the runtime registry. Codegen
/// target for `ph2d-tool-sync` (`register_all`).
pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

/// Construct the Upscale tool as a boxed trait object for the behavior
/// `ToolRegistry`. Codegen target for `ph2d-tool-sync`
/// (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(UpscaleTool::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = Registry::default();
        register(&mut reg);
        reg.build().expect("registry should build with upscale");
        let found = reg.by_id("upscale").expect("upscale registered by id");
        assert_eq!(found.id, "upscale");
        assert_eq!(found.label_key, "tool.upscale.label");
    }

    #[test]
    fn register_is_idempotent() {
        let mut reg = Registry::default();
        register(&mut reg);
        register(&mut reg);
        reg.build().expect("idempotent registration should build");
        assert_eq!(reg.manifests().len(), 1);
    }

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
    fn manifest_lives_in_image_tools_cluster_and_is_rightmost() {
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
        // Coord reserved 120 as the rightmost of the row (trim 40,
        // make_square 50, bgremoval 60, real_size 70, padding 80,
        // color_equalization 90, equalize_sizes 100, rasterize 110,
        // upscale 120). `const` block so a manifest tweak that drops
        // order fails the build, not just the test.
        const { assert!(MANIFEST.order == 120) };
    }

    #[test]
    fn make_returns_boxed_upscale_tool() {
        let t = make();
        assert_eq!(
            t.id(),
            ph2d_editor_core::floating_panel::ToolId::new("upscale")
        );
    }
}

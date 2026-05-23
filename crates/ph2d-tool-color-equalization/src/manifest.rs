//! Color Equalization — [`ToolManifest`] declaration.
//!
//! Stateful tool: clicking the pill in the Image Tools row opens a docked
//! panel with CLAHE / brightness / contrast / saturation / auto-WB controls
//! and an Apply button. The pure pipeline lives in [`crate::algorithm`];
//! the panel + shell bake follow the BgRemoval / Padding precedent
//! (downcast via `as_any_mut`, full-resolution commit on the pending-apply
//! latch — `ph2d-tool-color-equalization` does NOT implement `ImageEditTool`
//! yet, per DIRETRIZ §3.8.3.1).
//!
//! ### Handler — shadow mode
//! Like every Image Tools manifest, the real handler runs shell-side (it
//! mutates `SimWorld` + swaps the sprite texture), so the manifest
//! `handler` slot points to no-op [`shadow_handler`]s. The manifest still
//! registers so the registry-derived TopBar pill + the HR-12/13/15 gates
//! pick it up.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::color_equalization_bezpath;

/// Shadow-mode handler trio. Real work happens in the desktop shell's
/// color equalization tool integration (Coordinator wiring).
fn shadow_handler() {}

/// The canonical Color Equalization manifest. Kept byte-for-byte in sync
/// with `docs/design/tools/color_equalization.toml` (the
/// `tool_manifest_design_sync` arch test pins the contract).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "color_equalization",
    label_key: "tool.color_equalization.label",
    icon_fn: color_equalization_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // Image Tools row order: trim 40 → make_square 50 → bgremoval 60 →
    // real_size 70 → padding 80 → color_equalization 90.
    order: 90,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    // Preview thumbnail caps at 512²; the preview RGBA + scratch
    // luminance histogram fit well under the existing texture pool
    // headroom. The full-res bake stages into the asset-cooker output
    // path like make_square / trim / padding.
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
    fn manifest_order_is_right_of_padding() {
        const { assert!(MANIFEST.order > 80) };
    }
}

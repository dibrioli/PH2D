//! Rasterize — [`ToolManifest`] declaration.
//!
//! Action one-shot: clicking `[▦ Rasterize]` in the Image Tools row
//! bakes the selected sprite's Transform (scale + rotation + flip) into
//! its pixel buffer and resets the Transform to identity. Pure logic
//! lives in [`crate::algorithm`]; the ECS write (sprite source +
//! Transform mutation) happens shell-side, same pattern as the sibling
//! Image Tools (`trim_transparency`, `make_square`, `real_size`).
//!
//! ### Handler — shadow mode
//!
//! The real handler runs in the desktop shell drain
//! (`shells/desktop/src/render_loop/mod.rs::OneShotImageOp` arm) because
//! it needs `Renderer` + `Sprite`/`Transform` ECS mutations. Until that
//! arm is wired (Coordinator follow-up — outside this crate's pasta),
//! the manifest's `handler` slot points to [`shadow_handler`], a no-op.
//! The manifest still registers so the registry-derived TopBar pill +
//! HR-12/13/15 CI gates pick it up.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::rasterize_bezpath;

/// Shadow-mode handler. Real work happens in the desktop shell's
/// `EditorAction::OneShotImageOp { tool_id: "rasterize", entity_bits }`
/// drain (ADR-0040 TG-A genericized the per-tool variant).
fn shadow_handler() {}

/// The canonical Rasterize manifest.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "rasterize",
    label_key: "tool.rasterize.label",
    icon_fn: rasterize_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // Image Tools row order (sorted left→right by `order`):
    //   trim 40 → make_square 50 → bgremoval 60 → real_size 70 →
    //   color_equalization 90 → equalize_sizes 100 → rasterize 110 →
    //   upscale 120.
    // Coordinator reserved `110` for this slot.
    order: 110,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    // The bake allocates a transient Mitchell-Netravali work buffer
    // sized to the output pixels (`(w * |sx|) * (h * |sy|)` worst case,
    // plus a rotation bbox), but that lives on the stack of the action
    // call and is freed immediately. No tool-side standing budget.
    memory_budget: MemoryBudget::new(0, 0, 0),
    // Mutates sprite source + Transform shell-side; under shadow mode
    // the handler stub itself touches no sim state. Flip to `true` once
    // the generic dispatcher routes the real handler through here.
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
        // HR-15 i18n CI gate shape: `tool.<id>.label`.
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
    fn manifest_a11y_role_is_button() {
        // One-shot action → canonical role is Button (HR-12).
        assert_eq!(MANIFEST.a11y_role, Role::Button);
    }

    #[test]
    fn manifest_lives_in_image_tools_cluster() {
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }

    #[test]
    fn manifest_order_slots_between_equalize_sizes_and_upscale() {
        // Image Tools row left→right: equalize_sizes(100) → rasterize(110)
        // → upscale(120). `const {}` so the const-value assertion does
        // not trip clippy::assertions_on_constants.
        const { assert!(MANIFEST.order > 100) };
        const { assert!(MANIFEST.order < 120) };
    }
}

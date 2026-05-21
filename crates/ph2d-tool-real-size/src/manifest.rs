//! Real Size — [`ToolManifest`] declaration.
//!
//! Action one-shot: clicking `[1:1 Real Size]` in the Image Tools row
//! resets the selected sprite's Transform scale to 1:1 (preserving flip
//! sign). The pure logic is in [`crate::algorithm`]; the ECS write lives
//! in the desktop shell drain (Coordinator wiring) the same way Make
//! Square / Trim do.
//!
//! ### Handler — shadow mode
//!
//! Like the other Image Tools manifests (`make_square`, `trim_transparency`,
//! `bgremoval`), the real handler runs shell-side (it mutates the
//! `SimWorld` `Transform`), so the manifest `handler` slot points to a
//! no-op [`shadow_handler`] until the generic dispatcher (PR 9) lands.
//! The manifest still registers so the registry-derived TopBar pill +
//! the HR-12/13/15 CI gates pick it up.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::real_size_bezpath;

/// Shadow-mode handler. Real work happens in the desktop shell's
/// `EditorAction::RealSize` drain (Coordinator wiring).
fn shadow_handler() {}

/// The canonical Real Size manifest.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "real_size",
    label_key: "tool.real_size.label",
    icon_fn: real_size_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    // Sits right of `bgremoval` (order 60) on the Image Tools row:
    // trim 40 → make_square 50 → bgremoval 60 → real_size 70.
    order: 70,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    // Pure transform reset — no working buffer, no pixel allocation.
    memory_budget: MemoryBudget::new(0, 0, 0),
    // Mutates the sprite's `Transform.scale` shell-side; under shadow
    // mode the handler stub itself touches no sim state. Flip to `true`
    // once PR 9 routes the real handler through the dispatcher.
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
        // Chrome derivation groups the pill alongside the other image
        // actions; the cluster id must stay exact.
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }

    #[test]
    fn manifest_order_is_right_of_bgremoval() {
        // Paint order on the Image Tools row (sorted by `order`):
        // real_size sits to the right of bgremoval (60).
        assert!(MANIFEST.order > 60);
    }
}

//! Make Square — [`ToolManifest`] declaration.
//!
//! Action one-shot: clicking `[▢ Make Square]` in the Image Tools row
//! pads the selected sprite's source bitmap to a perfect square. The
//! algorithm is in [`crate::algorithm`]; this file is purely the
//! manifest tying it to the registry-driven chrome.
//!
//! ### Handler — shadow mode (PR 4 piloto)
//!
//! The full one-shot handler that the generic dispatcher (PR 9) will
//! eventually call lives inside the desktop shell because it needs
//! types only available there (`Renderer`, `ImageEditSnapshot`,
//! `ToastQueue`, `Sprite`/`Transform` ECS mutations). Wiring that
//! handler through the dispatcher requires the full `ToolCtx` design
//! (PR 9). Until then, the manifest's `handler` slot points to
//! [`shadow_handler`] — a no-op stub — and the legacy
//! `pending_make_square` drain in `shells/desktop/src/main.rs` keeps
//! running. The manifest still registers, so:
//!  - PR 8 (chrome derived from registry) can derive the TopBar
//!    `[▢ Make Square]` button from this manifest.
//!  - CI lints (HR-13/HR-15/HR-12) pick up the manifest.
//!  - NodeId for the chrome button is hash-derived (HR-5 stable).

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::square_bezpath;

/// PR 4 shadow-mode handler. Real work happens in
/// `shells/desktop/src/main.rs` `pending_make_square` drain. Replaced
/// by the dispatcher path in PR 9.
fn shadow_handler() {}

/// The canonical Make Square manifest. Tool crates expose a `pub const
/// MANIFEST` so `register()` can hand a `&'static ToolManifest` to the
/// registry without per-call allocation.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "make_square",
    label_key: "tool.make_square.label",
    icon_fn: square_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 50, // Sits right of `trim_transparency` (order 40) per
    // `tools/make_square/INTEGRATION.md` §2.
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    // Pure-CPU action; the bitmap output is staged into an
    // `Individual` texture by the shell, charged against the asset
    // texture pool already accounted for in core baseline. The action
    // itself reserves zero additional budget (HR-13).
    memory_budget: MemoryBudget::new(0, 0, 0),
    // Mutates sprite's `source` + `Transform` on commit — that's a
    // shell-side ECS write in `SimWorld`. But the manifest's
    // `touches_sim` flag describes whether the *handler invocation*
    // path itself reads/writes sim state via the dispatcher; under
    // shadow mode the handler is a stub. Set to `true` for accuracy
    // once PR 9 routes the real handler through here.
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
        // HR-15 i18n CI gate (PR 3) enforces this shape; this is the
        // crate-local smoke catching it before CI.
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
        // Make Square is a one-shot action, not a toggle/menu, so the
        // canonical role is `Button` (HR-12).
        assert_eq!(MANIFEST.a11y_role, Role::Button);
    }

    #[test]
    fn manifest_lives_in_image_tools_cluster() {
        // Chrome derivation (PR 8) expects this exact cluster id to
        // group the button alongside `trim_transparency`. If the
        // cluster name moves, both manifests have to move together —
        // this guard catches drift early.
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }
}

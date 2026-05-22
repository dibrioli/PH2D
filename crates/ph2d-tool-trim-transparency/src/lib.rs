#![forbid(unsafe_code)]
//! ph2d-tool-trim-transparency — Trim Transparency one-shot, isolated crate.
//!
//! Declares the `ToolManifest` (chrome pill) AND owns the pure trim
//! [`algorithm`] (ADR-0040 T3): scan a sprite's RGBA for the opaque
//! bounding box and crop to it. The shell drains `EditorAction::Trim`
//! and calls [`trim_transparency`]. Std-only, no editor-core dep — the
//! recenter geometry editor-core needs is its own `PixelBounds`, built
//! from this crate's [`Bounds`] fields at the shell (ADR-0040: the
//! decoupling that lets editor-core stay foundation ⊥ tools).

pub mod algorithm;

pub use algorithm::{Bounds, TrimResult, trim_transparency};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{
    BezPath, HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone,
};

/// Lucide-derived crop glyph for the action pill. Two open subpaths
/// forming the corner brackets that signify "trim to bounds".
///
/// SVG source: `docs/design/icons/trim-transparency.svg`. Hand-ported
/// to a `BezPath` here so this crate stays decoupled from the editor's
/// SVG codegen pipeline (`ph2d-editor/build.rs`). The 24-unit design
/// space matches the manifest convention across every tool crate.
fn icon() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Top-left bracket: down the left edge, across the bottom.
    p.move_to(Point::new(6.0, 2.0));
    p.line_to(Point::new(6.0, 16.0));
    p.line_to(Point::new(8.0, 18.0));
    p.line_to(Point::new(22.0, 18.0));
    // Bottom-right bracket: across the top, down the right edge.
    p.move_to(Point::new(18.0, 22.0));
    p.line_to(Point::new(18.0, 8.0));
    p.line_to(Point::new(16.0, 6.0));
    p.line_to(Point::new(2.0, 6.0));
    p
}

/// PR 9 dispatcher target. Until then, the host shell drains
/// `HeroScreen::pending_trim` directly when the chrome pill is
/// clicked — see `shells/desktop/src/main.rs`.
fn shadow_handler() {}

/// Manifest. `id = "trim_transparency"` is the canonical slug
/// hashed by `hash_node_id` to produce both the chrome `NodeId`
/// (in `screens/hero/ids.rs::IMAGE_ACTION_TRIM`) and the registry
/// key used by `image_action_pills`.
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "trim_transparency",
    label_key: "tool.trim_transparency.label",
    icon_fn: icon,
    // TopBar Image Tools action row — leftmost pill (order 40 is
    // below make_square 50 and bgremoval 60).
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 40,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    // Pure CPU pixel scan + memcpy. Output bitmap is staged into an
    // Individual texture by the shell; the action itself reserves
    // no working buffer (HR-13).
    memory_budget: MemoryBudget::new(0, 0, 0),
    // Mutates sprite's `source` + `Transform` on commit (shell-side
    // ECS write). Manifest's `touches_sim` is false under shadow
    // mode because the handler stub doesn't touch sim; set to `true`
    // once PR 9 routes the real handler through the dispatcher.
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Trim Transparency manifest with the runtime registry.
/// Called from `crates/ph2d-tool-registry-init/src/lib.rs::register_all`.
pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = Registry::default();
        register(&mut reg);
        reg.build()
            .expect("registry should build with trim_transparency");
        let found = reg
            .by_id("trim_transparency")
            .expect("trim_transparency should be registered by id");
        assert_eq!(found.id, "trim_transparency");
        assert_eq!(found.label_key, "tool.trim_transparency.label");
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
    fn manifest_lives_in_image_tools_cluster() {
        // Wave 2 PR 11.4 contract: all three image-action pills live
        // in the `image_tools` cluster so `Registry::cluster` returns
        // the full set with one call.
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }

    #[test]
    fn icon_is_non_empty() {
        assert!(!icon().elements().is_empty());
    }
}

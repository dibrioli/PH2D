#![forbid(unsafe_code)]
//! ph2d-tool-grid-snap — Grid Settings tool manifest.
//!
//! Stateful Tool: cycles among 11 grid kinds, applies snap-to-grid in
//! gizmo Translate + drag-drop, exposes a Procreate-style panel with
//! per-kind config widgets.
//!
//! **PR 6 scope** (convention-by-discovery migration): this crate
//! ships the manifest only. The implementation (state, panel, render,
//! snap math) continues to live at
//! `crates/ph2d-editor/src/grid_snap/` — moving the ~5500 LOC there
//! would require refactoring `screens/hero.rs` first because the
//! state is stored on `HeroScreen`. The manifest is enough to wire
//! the registry-derived chrome (PR 8) so the Grid Settings TopBar
//! entry stops being hand-coded in `screens/hero/fixture.rs`.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{
    BezPath, HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone,
};

/// Lucide-derived 3×3 grid glyph for the TopBar button. Three
/// horizontal + three vertical line segments forming a window into a
/// 24×24 design space (matches the rest of the icon system).
fn icon() -> BezPath {
    use ph2d_vector::Shape;
    // Two perpendicular line trios — kurbo's BezPath supports
    // multiple disjoint subpaths.
    let mut p = BezPath::new();
    // Vertical lines at x = 8, 12, 16 (from y=4 to y=20).
    for x in [8.0_f64, 12.0, 16.0] {
        let mut seg = BezPath::new();
        seg.move_to((x, 4.0));
        seg.line_to((x, 20.0));
        p.extend(seg.iter());
    }
    // Horizontal lines at y = 8, 12, 16.
    for y in [8.0_f64, 12.0, 16.0] {
        let mut seg = BezPath::new();
        seg.move_to((4.0, y));
        seg.line_to((20.0, y));
        p.extend(seg.iter());
    }
    let _ = p.bounding_box(); // smoke: confirm path is non-empty.
    p
}

/// Shadow-mode handler. Real grid-snap logic lives in
/// `crates/ph2d-editor/src/grid_snap/` and is dispatched through
/// `hero_screen.grid_snap_state` + `screens/hero/topbar.rs`. PR 9
/// generic dispatcher will route this manifest's invocation to a
/// proper handler.
fn shadow_handler() {}

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "grid_snap",
    label_key: "tool.grid_snap.label",
    icon_fn: icon,
    // Settings cluster on the right edge of the TopBar (Save / Open /
    // Image Tools row sits left of it). cluster id matches the
    // existing `topbar_clusters()` slot in `screens/hero/fixture.rs`.
    zone: Zone::TopRight,
    cluster: "settings",
    order: 10,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

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
        reg.build().expect("registry should build with grid-snap");
        assert_eq!(reg.by_id("grid_snap").unwrap().id, "grid_snap");
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
    fn icon_returns_non_empty_path() {
        let p = icon();
        assert!(!p.elements().is_empty());
    }
}

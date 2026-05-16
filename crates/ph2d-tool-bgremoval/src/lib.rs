#![forbid(unsafe_code)]
//! ph2d-tool-bgremoval — Background Removal tool manifest.
//!
//! Stateful Tool — active in the TopBar `image_tools` cluster, presents a
//! Procreate-style panel with mode toggles (chroma + flood / GrabCut)
//! and an Apply Toggle. Pixel work happens at full resolution on
//! commit; live preview runs on a downscaled 1024² Oklab thumbnail
//! computed by the M1 path.
//!
//! **PR 7 scope** (convention-by-discovery migration): manifest only.
//! The implementation (`BgRemovalTool`, M1 algorithm, M2 GrabCut
//! scaffold, panel build) continues to live at
//! `crates/ph2d-editor/src/tools/bgremoval/` while slot 2 iterates on
//! the M2 body. Manifest unlocks registry-derived chrome (PR 8) for
//! the Bg Removal LeftRail entry. Same pattern as grid-snap PR 6.

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{
    BezPath, HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone,
};

/// Eraser glyph re-uses the canonical Lucide port already shipped at
/// `ph2d_editor::eraser_bezpath` (M1 slot 2 work). No reason to
/// duplicate the ~50-line BezPath construction — `ph2d-editor` is a
/// safe dep direction (it does not depend on tool crates per PR 6.0).
fn icon() -> BezPath {
    ph2d_editor::eraser_bezpath()
}

/// Shadow-mode handler trio. Real dispatch goes through the legacy
/// `ToolRegistry` in `shells/desktop` and `BgRemovalTool::Tool` trait
/// impl. PR 9 generic dispatcher will replace these stubs.
fn shadow_handler() {}

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "bgremoval",
    label_key: "tool.bgremoval.label",
    icon_fn: icon,
    // `image_tools` cluster — pill in the TopBar's image-action row
    // (Image Tools toggle in `TOPBAR_IMAGE_TOOLS` flips it on). Shares
    // the cluster with `make_square` (order 50) and `trim_transparency`
    // (order 40); bgremoval at 60 sits to the right of both per
    // ImageToolsV1 spec. Wave 2 PR 11.4 renamed this from
    // `image_tools_rail` to align with the canonical chrome cluster
    // name that `make_square` already used.
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 60,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: shadow_handler as HandlerFn,
        on_deactivate: shadow_handler as HandlerFn,
        on_panel_event: shadow_handler as HandlerFn,
    },
    // Live preview path allocates a 1024² Oklab buffer in worst case
    // (~16 MB working buffer when active). Reserved against ram_mb so
    // HR-13 boot check fires if a small-budget platform is over.
    memory_budget: MemoryBudget::new(0, 16, 0),
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
        reg.build().expect("registry should build with bgremoval");
        assert_eq!(reg.by_id("bgremoval").unwrap().id, "bgremoval");
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

    #[test]
    fn memory_budget_reserves_working_buffer() {
        // M1 path allocates a 1024² Oklab buffer (~16 MB worst case).
        // Make sure the manifest declares this so HR-13 boot check
        // fires on small-budget platforms. `const` block makes this
        // a compile-time assertion — manifest mutation that drops
        // the budget below 16 MB fails the build, not the test.
        const { assert!(MANIFEST.memory_budget.ram_mb >= 16) };
    }
}

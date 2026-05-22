#![forbid(unsafe_code)]
//! ph2d-tool-bgremoval — Background Removal: the full tool, isolated crate.
//!
//! Stateful Tool — active in the TopBar `image_tools` cluster, presents a
//! Procreate-style panel with mode toggles and an Apply Toggle. Pixel work
//! happens at full resolution on commit; live preview runs on a downscaled
//! thumbnail.
//!
//! ADR-0040 T2 — this crate now owns the whole feature: the `ToolManifest`
//! (data), [`BgRemovalTool`] (behavior, `impl ph2d_editor_core::tool::Tool`),
//! the pure [`algorithm`] pipeline, [`scratch`] buffers, and [`icon`]. It
//! depends on `ph2d-editor-core` (the `Tool`/widget/`FloatingPanel` contract,
//! the allowed satellite direction), never the reverse.
//!
//! The UI/action **vocabulary** (`BgRemovalUiEdit`/`BgRemovalUiSnapshot`/
//! `BrushFalloff`/`BgRemovalParams` + scale consts) still lives in
//! `ph2d_editor_core::tools::bgremoval::params` because the editor-core action
//! bus and the `ph2d-panel-bgremoval` crate read it. It is re-exported here as
//! [`params`] so the moved files' `super::params` paths resolve unchanged; it
//! migrates into this crate when the generic action channel lands (T1.2/T1.3).

pub mod algorithm;
pub mod icon;
pub mod scratch;
pub mod tool;

/// UI/action vocabulary, re-exported from editor-core (see module docs). The
/// re-export makes `crate::params` (hence `super::params` in the moved files)
/// resolve to the foundation-side single source of truth.
pub use ph2d_editor_core::tools::bgremoval::params;

pub use tool::BgRemovalTool;

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Shadow-mode handler trio. Real dispatch goes through the `ToolRegistry`
/// in `shells/desktop` and the `BgRemovalTool` `Tool` impl. The generic
/// action channel (ADR-0040 T1.2/T1.3) will replace these stubs.
fn shadow_handler() {}

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "bgremoval",
    label_key: "tool.bgremoval.label",
    icon_fn: icon::eraser_bezpath,
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
        let p = icon::eraser_bezpath();
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

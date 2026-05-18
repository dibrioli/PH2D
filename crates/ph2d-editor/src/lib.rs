#![forbid(unsafe_code)]
//! ph2d-editor — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! Foundation:
//! - [`zones::Layout`] — 4-zone canonical positioning (top-left
//!   creates / top-right edits / sidebar modulates / center 100 %
//!   canvas). Per ADR-0023 §3.
//! - [`floating_panel::FloatingPanel`] — Procreate-style draggable
//!   tool drawer primitive. Per ADR-0023 §5.
//! - [`widget`] — primitives (Button, Slider, Toggle, RadioGroup,
//!   ColorSwatch). Each follows the same pattern: data + state enum
//!   + tokens + a11y::Node + colocated `paint_X` helper.
//! - [`tool::Tool`] + [`tool::ToolRegistry`] — canonical contract
//!   every editor tool implements (id / label / icon / build_panel
//!   / activate hooks). Registry tracks the active tool.
//! - [`tools`] — seed implementations ([`tools::BrushTool`],
//!   [`tools::MoveTool`]) proving the trait shape.
//! - [`zen::ZenMode`] — Tab-toggle workspace state. Per ADR-0023 §2.
//! - [`toast::ToastQueue`] — non-modal notification stream. Per
//!   ADR-0023 §2 ("Notificações flutuantes não-modais").
//! - [`paint`] — Vello lowering (`Paint` trait + `paint_text`
//!   helper). M11 widget paint pass + this PR's text rendering.
//!
//! Out of scope (M13+):
//! - QuickMenu radial (ADR-0023 §6)
//! - Gesture-mapping editor UI (ADR-0023 §4)
//! - Single-Touch Companion overlay

pub mod action_bus;
pub mod grid_snap;
pub mod image_edit;
pub mod panel_registry;
pub mod screens;

/// Test-only helpers exposed for integration tests + downstream
/// crate tests that construct `HeroScreen`. Not part of the stable
/// public API; see [`test_support::ensure_panel_registry`] for the
/// Wave 8 boot-order helper.
#[doc(hidden)]
pub mod test_support;

// Wave 6+7 Phase 2: leaf/utility modules promoted to `ph2d-editor-core`.
// Re-exported here so `crate::zones::Rect` (and `crate::icons::*` etc.)
// continue to resolve from inside ph2d-editor, and
// `ph2d_editor::zones::Rect` continues to resolve from downstream
// consumers (shells, tool crates). Non-leaf modules (action_bus,
// floating_panel — both reference hero state / widget primitives)
// stay in ph2d-editor until a future phase extracts those dependencies.
pub use ph2d_editor_core::{
    floating_panel, gizmo, grid, icons, interaction, paint, project, toast, widget, zen, zones,
};

/// Re-export of `ph2d-tool-registry` under the path
/// `ph2d_editor::registry` so existing callers
/// (`ph2d_editor::registry::Registry`, etc.) keep working byte-for-byte
/// after the PR 4.0 extraction. See
/// `docs/Migracao/2026-05-convention-by-discovery.md`.
pub use ph2d_tool_registry as registry;

/// Process-wide handle on the runtime [`registry::Registry`]. Set once
/// by the host shell at boot via [`install_registry`]; consumed by the
/// hero painters that derive chrome from manifests (Wave 2 PR 11.4 —
/// `image_action_pills`, `topbar_clusters`).
///
/// `OnceLock` was picked over a per-`HeroScreen` field because the
/// registry is shell-scoped (one Registry per process) and threading
/// it through every painter signature would touch 30+ call sites for a
/// value that never varies. Tests that need a registry-driven path
/// call `install_registry` themselves; tests that don't see `None` and
/// fall back to the legacy hardcoded list — the contract is checked by
/// the `chrome_manifest_coverage` integration test.
static EDITOR_REGISTRY: std::sync::OnceLock<registry::Registry> = std::sync::OnceLock::new();

/// Install the process-wide registry. Returns `true` on first install,
/// `false` if a registry was already installed (so re-init in test
/// harnesses or split-binary boot flows is safe — the second registry
/// is silently dropped). Callers that care about which registry won
/// can inspect the return value.
pub fn install_registry(reg: registry::Registry) -> bool {
    EDITOR_REGISTRY.set(reg).is_ok()
}

/// Read the installed registry, if any. Painters use the fallback path
/// (legacy hardcoded chrome) when this returns `None` — see callers
/// in `screens/hero/topbar.rs`.
pub fn installed_registry() -> Option<&'static registry::Registry> {
    EDITOR_REGISTRY.get()
}
pub mod tool;
pub mod tools;

pub use floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelControl, PanelTab, ToolId};
pub use gizmo::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoView,
    TransformSnapshot, anchor_pivot_world, compute_gizmo_transform, gizmo_kind_for_id,
    is_gizmo_handle_id, paint_sprite_gizmo,
};
pub use grid::{GridConfig, GridLineCounts, GridView, count_visible_lines, paint_grid};
pub use icons::{IconCmd, IconId, cmd_to_path};
pub use interaction::{
    HitIndex, InteractiveState, WidgetEvent, WidgetStore, dispatch_key, dispatch_pointer,
    dispatch_text_input, dispatch_tick,
};
pub use paint::{
    Paint, PaintCtx, fill_rounded_rect, paint_icon, paint_text, paint_text_centered,
    paint_text_title, paint_tool_palette_icons, resolve, stroke_rect, stroke_rounded_rect,
};
pub use project::{
    DEFAULT_PIXELS_PER_METER, DisplayUnit, MAX_PIXELS_PER_METER, MIN_PIXELS_PER_METER,
    ProjectSettings,
};
pub use screens::{
    BottomHudStats, HeroScreen, HeroSelection, InspectorNameInfo, InspectorSpriteInfo,
    InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    RequestedSpriteStrategy, ViewFocusKind, paint_hero_screen, set_live_component_count,
};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use tool::{PanelEvent, Tool, ToolRegistry};
// Re-export so the shell can name the dragging node id without
// taking a direct ph2d-a11y dep just for one type.
pub use ph2d_a11y::NodeId;
// Wave 2.5 PR 11.11: `tools::*`, `image_edit::*`, and the 70-item
// `widget::*` re-export block were removed to eliminate the merge
// zone they created (every new tool / widget / image-edit helper
// edited this file). Consumers reach those types via their module
// paths instead:
//
//   ph2d_editor::widget::Button       (was ph2d_editor::Button)
//   ph2d_editor::tools::trim_transparency
//   ph2d_editor::tools::bgremoval::BgRemovalTool
//   ph2d_editor::image_edit::recenter_after_crop
//
// SKILL §12.3 marks PH2D pré-1.0 ("0.x.y aceita quebras em x") so
// the path change is on-policy for Wave 2.5.

pub use zen::ZenMode;
pub use zones::{Layout, Zone};

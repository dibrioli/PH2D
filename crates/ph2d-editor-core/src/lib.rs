//! ph2d-editor-core — Procreate-style canvas-first editor (M12, ADR-0023).
//!
//! ADR-0029 Phase B.2 absorbed the orchestrator content from `ph2d-editor`
//! (HeroScreen, action_bus, grid_snap, image_edit, screens/*) so panel
//! crates can depend ONLY on this crate without pulling in the legacy
//! `ph2d-editor` shim. Phase D deleted the legacy fn-pointer panel registry
//! (every in-tree panel migrated to `panel::PANEL_REGISTRY` typed Panel<State>).
//! ADR-0040 TG-B/C/D moved every tool impl AND vocabulary into satellite
//! `ph2d-tool-*` crates and deleted `editor-core/src/tools/` outright.
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
//!   every editor tool implements. Tool impls and their UI/action
//!   vocabulary live in `ph2d-tool-*` satellite crates (ADR-0040 — the
//!   foundation no longer hosts a `tools` module).
//! - [`zen::ZenMode`] — Tab-toggle workspace state.
//! - [`toast::ToastQueue`] — non-modal notification stream.
//! - [`paint`] — Vello lowering (`Paint` trait + `paint_text` helper).
//! - [`panel`] — ADR-0029 trait-driven panel host. All four in-tree
//!   panels (Inspector, Hierarchy, Widget Gallery, Grid Snap) live
//!   here as typed `Panel<State>` impls in their own crates; the
//!   process-wide [`panel::PANEL_REGISTRY`] is installed at boot by
//!   `ph2d_panel_registry_init::register_all_panels()`.

#![forbid(unsafe_code)]

pub mod action_bus;
pub mod floating_panel;
pub mod gizmo;
pub mod grid;
pub mod grid_snap;
pub mod icons;
pub mod ids;
pub mod image_edit;
pub mod interaction;
pub mod math;
pub mod paint;
pub mod panel;
pub mod project;
pub mod screens;
pub mod toast;

/// Test-only helpers exposed for integration tests + downstream
/// crate tests that construct `HeroScreen`. Not part of the stable
/// public API; see [`test_support::ensure_panel_registry`] for the
/// Wave 8 boot-order helper.
#[doc(hidden)]
pub mod test_support;

pub mod tool;
pub mod widget;
pub mod zen;
pub mod zones;

/// Re-export of `ph2d-tool-registry` under the path
/// `ph2d_editor_core::registry` so existing callers
/// (`ph2d_editor::registry::Registry`, etc.) keep working byte-for-byte
/// after the ADR-0029 Phase B.2 absorption.
pub use ph2d_tool_registry as registry;

/// Process-wide handle on the runtime [`registry::Registry`]. Set once
/// by the host shell at boot via [`install_registry`]; consumed by the
/// hero painters that derive chrome from manifests.
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

pub use floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelControl, PanelTab, ToolId};
pub use gizmo::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoHit, GizmoModifiers, GizmoSnap, GizmoTarget,
    GizmoView, TransformSnapshot, anchor_pivot_world, compose_snapshot, compute_gizmo_transform,
    gizmo_kind_for_id, is_gizmo_handle_id, move_pivot_transform, paint_gizmo_outline,
    paint_sprite_gizmo, paint_sprite_gizmo_keyed, pivot_snap_candidates, world_delta_to_local,
    world_translation_to_local,
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
    DEFAULT_PIXELS_PER_METER, DisplayUnit, ImageFilterMode, MAX_PIXELS_PER_METER,
    MIN_PIXELS_PER_METER, ProjectSettings, image_quality_for,
};
pub use screens::{
    BottomHudStats, HeroScreen, HeroSelection, InspectorNameInfo, InspectorSpriteInfo,
    InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    RequestedSpriteStrategy, SpriteFieldEdit, ViewFocusKind, paint_hero_screen,
};
pub use toast::{Toast, ToastQueue, ToastSeverity};
pub use tool::{PanelEvent, Tool, ToolRegistry};
// Re-export so the shell can name the dragging node id without
// taking a direct ph2d-a11y dep just for one type.
pub use ph2d_a11y::NodeId;

pub use zen::ZenMode;
pub use zones::{Layout, Zone};

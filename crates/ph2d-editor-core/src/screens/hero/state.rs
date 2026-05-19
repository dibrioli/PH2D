//! Wave 5 stage B — HeroScreen sub-state groups.
//!
//! Replaces the flat 30+-field god-struct with 6 cohesive groups.
//! Cross-group access still goes through the parent `HeroScreen`
//! (`hero.inspector.sprite`, `hero.hierarchy.live_entries`, etc.);
//! no inter-group method dependencies — each group is plain data.
//!
//! ## Group inventory
//!
//! - [`InspectorState`] — visibility + 4 per-frame snapshot fields
//!   (sprite/transform/visibility/name) + `last_entity` rewrite guard.
//! - [`HierarchyState`] — visibility + live entity map + rename target.
//! - [`ImageEditState`] — TopBar Image-Tools mode + undo availability.
//! - [`ViewState`] — UI mirror toggle + 3 overlay visibility flags
//!   (stats HUD / widget gallery / grid overlay) + gallery rect.
//! - [`GizmoStateGroup`] — selection + per-frame view + in-progress drag.
//! - [`GridState`] — per-frame projection view + config + snap subsystem.
//!
//! Default impls match the pre-decomp `HeroScreen::new` defaults
//! (inspector/hierarchy visible by default; stats + grid overlay
//! visible; everything else off / None). Pre-existing snapshot types
//! (`InspectorSpriteInfo` etc.) keep their place in `hero.rs` for
//! reach — moving them would churn the import surface for no gain.

use ph2d_a11y::NodeId;

use super::fixture;
use crate::zones::Rect;

// ADR-0029 Phase C.1: `InspectorState` migrated to
// `ph2d_panel_inspector::state::InspectorState`. Snapshots
// (`InspectorSpriteInfo` etc.) keep their definitions in
// [`super::HeroScreen`] for now; future panel cleanups may move them
// alongside the state.

/// Hierarchy panel state — visibility, live entity map injected by
/// the host (ADR-0025 M14.4a), and the inline-rename target.
#[derive(Clone, Debug, Default)]
pub struct HierarchyState {
    /// Visibility — toggled by the `RAIL_SHOW_HIERARCHY` left-rail
    /// button. Default `true`.
    pub visible: bool,
    /// Live-mode entity rows published by the host via
    /// [`super::HeroScreen::sync_from_hierarchy`]. When `Some`, the
    /// hierarchy panel renders these entries instead of
    /// `fixture::hierarchy()`. `None` keeps the fixture behavior
    /// (used by tests + standalone hero demo).
    pub live_entries: Option<std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>>,
    /// M14.7 polish: row currently in inline-rename mode. The
    /// hierarchy painter replaces the row's name label with a
    /// TextInput when this matches. `None` = no row in rename.
    pub rename_target_row: Option<NodeId>,
}

/// Image-edit subsystem state — TopBar Image-Tools mode flag + a
/// read-only signal mirroring the shell's image-edit undo snapshot.
#[derive(Copy, Clone, Debug, Default)]
pub struct ImageEditState {
    /// `true` when the TopBar is in **Image Tools mode**. Right-side
    /// clusters hide; image-editing action pills surface. Toggled by
    /// `TOPBAR_IMAGE_TOOLS` clicks (handled in `apply_event` before
    /// the topbar's stub). Default `false`.
    pub mode_on: bool,
    /// Read-only signal from the host: `true` when the host has a
    /// stored image-edit snapshot that Cmd+Z would restore. Lets the
    /// UI dim the `TOOL_UNDO` chip when no undo is available. Shell
    /// writes this each frame after its drain pass.
    pub has_undoable: bool,
}

/// View-state flags — mirror toggle + 4 overlay visibility flags
/// (stats HUD, widget gallery, grid overlay, plus the gallery rect
/// when shown). All purely UI presentation — no business logic.
#[derive(Clone, Debug)]
pub struct ViewState {
    /// When `true`, the Inspector and Hierarchy panels swap sides
    /// (Inspector left, Hierarchy right). Toggled via the "Mirror UI"
    /// entry in the theme context menu. Default `false`.
    pub ui_mirrored: bool,
    /// Visibility of the bottom statistics HUD — toggled by the
    /// "Show Statistics" entry in the theme context menu. Default
    /// `true`.
    pub stats_visible: bool,
    /// Visibility of the floating **Widget Gallery** panel — toggled
    /// by clicks on the `TOPBAR_WIDGET_GALLERY` palette button.
    /// Default `false`.
    pub widget_gallery_visible: bool,
    /// Rect of the Widget Gallery panel in viewport pixels. Set on
    /// first toggle to a centered default; persisted across frames
    /// so dragging keeps the position.
    pub widget_gallery_rect: Option<Rect>,
    /// World-space grid overlay toggle (ADR-0025 M14.4b). Default
    /// `true`. Toggled via the "Show Grid" context-menu entry and the
    /// `G` key.
    pub grid_visible: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            ui_mirrored: false,
            stats_visible: true,
            widget_gallery_visible: false,
            widget_gallery_rect: None,
            grid_visible: true,
        }
    }
}

/// Canvas gizmo state — current selection bits, per-frame projection
/// view, in-progress drag. Wave 2.5 promoted these from scattered
/// fields; Wave 5 groups them.
#[derive(Copy, Clone, Debug, Default)]
pub struct GizmoStateGroup {
    /// M14.7 A: sim-entity bits of the sprite currently selected for
    /// gizmo manipulation. Host's canvas-click handler runs
    /// `pick_sprite_at_world` against PresentWorld and writes here.
    /// `None` = nothing selected.
    pub selection: Option<u64>,
    /// M14.7 B: per-frame projection input for the gizmo painter.
    /// Host computes from `selection_bbox_world(present, selection)`
    /// plus current camera/window and pushes here just before
    /// `paint_hero_screen`. `None` ⇒ no gizmo painted this frame.
    pub view: Option<crate::gizmo::GizmoView>,
    /// M14.7 C: in-progress drag on the gizmo. Host's MouseInput
    /// handler fills on Down landing on a handle; Move advances
    /// `cursor_screen`, calls `compute_gizmo_transform`, writes back
    /// to SimWorld; Up clears the field.
    pub drag: Option<crate::gizmo::GizmoDragState>,
}

/// Grid subsystem state — per-frame projection view + paint config +
/// snap state (overlay + per-kind config + snap policy). `grid_visible`
/// stays on [`ViewState`] since it's an overlay toggle rather than a
/// grid-subsystem field.
#[derive(Clone, Debug, Default)]
pub struct GridState {
    /// Per-frame grid projection. `None` means host hasn't supplied a
    /// view yet → grid stays hidden even if `ViewState::grid_visible`
    /// is `true`. Set each frame via
    /// [`super::HeroScreen::set_grid_view`].
    pub view: Option<crate::grid::GridView>,
    /// Spacing + color config for the grid painter. Mutate via
    /// [`super::HeroScreen::grid_config_mut`] for project-level
    /// customization.
    pub config: crate::grid::GridConfig,
    /// Grid-snap subsystem state — kind selector, per-kind config,
    /// snap policy, overlay display + opacity. Canonical source for
    /// the canvas grid overlay (paints via
    /// [`crate::grid_snap::render::paint`]) and snapping world
    /// positions (via [`crate::grid_snap::GridSnapState::snap_world`]).
    /// Panel opens/closes via `TOPBAR_GRID_SETTINGS`.
    pub snap_state: crate::grid_snap::GridSnapState,
}

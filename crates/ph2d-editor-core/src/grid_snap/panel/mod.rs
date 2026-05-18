//! Floating-panel paint + event handler for the grid-snap subsystem.
//!
//! Layout: vertical stack of title bar / Grid Kind section (kind
//! cycler + per-kind parameter rows) / Snap section / Display
//! section (Show overlay + Opacity slider) / Inspect section.
//! Drag / resize handles match [`crate::screens::hero::widget_gallery`]
//! so the existing `BlenderHitKind::DragHandle` / `ResizeHandle`
//! dispatch moves the panel.
//!
//! Per-kind config rows are populated for **all** kinds at startup
//! (every NumberInput / cycle Button is registered) but painted +
//! hit-tested only for the active `state.kind`. Switching kind
//! preserves every other kind's params untouched.
//!
//! ## Wave 2.5 PR 11.7a decomposition
//!
//! Originally a 2869-LOC monolith; split by responsibility into
//! sibling files:
//!
//! - [`orchestrator`] — `paint()` top-level orchestrator
//! - [`populate`] — WidgetStore registration at boot
//! - [`paint_kinds`] — per-kind config painters (8 grid kinds)
//! - [`paint_helpers`] — generic paint primitives (segmented buttons, labeled rows, swatches)
//! - [`paint_rows`] — number rows + origin/aabb/overlay/opacity/labeled-toggle
//! - [`events`] — dispatch: apply_toggle / apply_value_changed / apply_click

mod events;
mod orchestrator;
mod paint_helpers;
mod paint_kinds;
mod paint_rows;
mod populate;

pub use orchestrator::paint;
pub use populate::populate;

use super::ids;
use super::state::{GridKind, GridSnapState, HexCfg};
use crate::interaction::{BlenderHitKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{paint_icon, paint_text, paint_text_title, resolve};
use crate::screens::hero::style::{
    paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect, panel_resize_handle_rect,
};
use crate::widget::{
    Button, ButtonKind, ButtonState, NumberInput, Slider, SliderOrientation, SliderState,
    TextInputState, Toggle, ToggleState, paint_button, paint_number_input_with_buffer,
    paint_slider, paint_toggle,
};
use crate::zones::Rect;
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggerParity;
use ph2d_grid::tri::TriNeighborhood;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

// Wave 2.5 PR 11.7a: sibling files reach each other's `pub(super) fn`s
// via `use super::*;` — these re-exports pull those names into the
// `panel/` namespace so call sites stay unchanged from before the
// decomposition.
pub(super) use events::{apply_click, apply_toggle, apply_value_changed};
pub(super) use paint_helpers::{
    NeighborhoodFamily, kind_option_ids_in_order, paint_color_swatch_row, paint_kind_button_grid,
    paint_labeled_segmented_row, paint_neighborhood_button_row, paint_section_label,
    paint_snap_top_toggle, paint_target_button_stack, set_neighborhood_for_active_kind,
};
pub(super) use paint_kinds::paint_kind_config;
pub(super) use paint_rows::{
    paint_aabb_rows, paint_number_row, paint_number_row_from_state, paint_opacity_slider_row,
    paint_origin_rows, paint_show_overlay_row,
};

thread_local! {
    /// Last computed scrollable content height of the panel (set by
    /// `paint`, read by the host caller in `paint_hero_screen`).
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Active project DisplayUnit + pixels-per-meter for THIS paint /
    /// apply_event call. State storage is always meters; this pair
    /// drives the meter↔display conversion at the panel boundary.
    /// `paint_hero_screen` calls `set_current_display_unit` before
    /// invoking `paint` and `apply_event` so the conversion sees the
    /// live project value. Default `Meters` matches the project
    /// default — a stale Some() can't outlive these calls anyway
    /// because the host updates them every frame.
    static CURRENT_DISPLAY_UNIT: std::cell::Cell<crate::project::DisplayUnit> =
        const { std::cell::Cell::new(crate::project::DisplayUnit::Meters) };
    static CURRENT_PPM: std::cell::Cell<f32> =
        const { std::cell::Cell::new(crate::project::DEFAULT_PIXELS_PER_METER) };
}

/// Last computed content height — call from the host after `paint`
/// to publish via `store.set_panel_content_h(GS_PANEL, …)`.
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

/// Last computed visible height — call from the host after `paint`
/// to publish via `store.set_panel_visible_h(GS_PANEL, …)`.
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

/// Set by `paint_hero_screen` once per frame before any grid_snap
/// `paint` / `apply_event` call. The two values are read inside the
/// panel by the meter↔display conversion helpers below.
pub fn set_current_display_unit(unit: crate::project::DisplayUnit, pixels_per_meter: f32) {
    CURRENT_DISPLAY_UNIT.with(|c| c.set(unit));
    CURRENT_PPM.with(|c| c.set(pixels_per_meter));
}

/// Reseed the store's NumberInput values for every meter-domain
/// field from the live state, converted through the active
/// DisplayUnit. Called by the host before `paint` so the rows that
/// read from the store (via `read_number_input`) display the right
/// magnitude for the current unit. Focused fields are skipped by
/// `WidgetStore::set_number_value` so in-progress user edits don't
/// get clobbered mid-typing.
///
/// Must be called AFTER `set_current_display_unit` so the conversion
/// sees the live unit. Unitless / integer fields (subdivisions,
/// max_per_leaf, RGB channels, etc.) are not touched.
pub fn sync_meter_inputs_to_display_unit(
    state: &GridSnapState,
    store: &mut crate::interaction::WidgetStore,
) {
    // CELL_SIZE is shared across most kinds — reseed it from whichever
    // cfg the active kind uses (Square/Hex/Tri/Chunks single
    // cell_size; StaggeredSquare uses cell_w; Quadtree/Voronoi don't
    // surface a cell_size row at all).
    let cell_size_m: Option<f32> = match state.kind {
        GridKind::Square => Some(state.square_cfg.cell_size),
        GridKind::Hex => Some(state.hex_cfg.cell_size),
        GridKind::StaggeredSquare => Some(state.staggered_square_cfg.cell_w),
        GridKind::StaggeredHex => Some(state.staggered_hex_cfg.hex.cell_size),
        GridKind::Tri => Some(state.tri_cfg.edge_length),
        GridKind::Chunks => Some(state.chunks_cfg.cell_size),
        // Iso uses tile_w / tile_h (NOT the shared CELL_SIZE input);
        // Quadtree / Voronoi don't paint a CELL_SIZE row at all.
        GridKind::Iso | GridKind::Quadtree | GridKind::Voronoi => None,
    };
    if let Some(m) = cell_size_m {
        store.set_number_value(ids::GS_CFG_CELL_SIZE, meters_to_display(m));
    }
    // Iso tile size — only painted when the Iso kind is active, but
    // the field is independent of any other kind so always reseed.
    store.set_number_value(
        ids::GS_CFG_ISO_TILE_W,
        meters_to_display(state.iso_cfg.tile_w),
    );
    store.set_number_value(
        ids::GS_CFG_ISO_TILE_H,
        meters_to_display(state.iso_cfg.tile_h),
    );
    // Quadtree bounds (4 floats — soft-clamped on commit).
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MIN_X,
        meters_to_display(state.quadtree_cfg.bounds.min[0]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MIN_Y,
        meters_to_display(state.quadtree_cfg.bounds.min[1]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MAX_X,
        meters_to_display(state.quadtree_cfg.bounds.max[0]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MAX_Y,
        meters_to_display(state.quadtree_cfg.bounds.max[1]),
    );
    // Voronoi bounds.
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
        meters_to_display(state.voronoi_cfg.bounds.min[0]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
        meters_to_display(state.voronoi_cfg.bounds.min[1]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
        meters_to_display(state.voronoi_cfg.bounds.max[0]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
        meters_to_display(state.voronoi_cfg.bounds.max[1]),
    );
    // Probe A / B (world meters) inputs.
    store.set_number_value(ids::GS_PROBE_A_X, meters_to_display(state.probe_a[0]));
    store.set_number_value(ids::GS_PROBE_A_Y, meters_to_display(state.probe_a[1]));
    store.set_number_value(ids::GS_PROBE_B_X, meters_to_display(state.probe_b[0]));
    store.set_number_value(ids::GS_PROBE_B_Y, meters_to_display(state.probe_b[1]));
    // Magnetism radius (meters) — keeps the panel input in sync after
    // unit toggles (m ↔ px).
    store.set_number_value(
        ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
        meters_to_display(state.snap_magnetism_radius),
    );
}

fn current_display_unit() -> crate::project::DisplayUnit {
    CURRENT_DISPLAY_UNIT.with(|c| c.get())
}

fn current_ppm() -> f32 {
    CURRENT_PPM.with(|c| c.get())
}

/// Convert a sim-stored meter value to the value to DISPLAY in a
/// NumberInput (under the active DisplayUnit).
#[inline]
fn meters_to_display(meters: f32) -> f64 {
    current_display_unit().from_meters(meters, current_ppm()) as f64
}

/// `pub(super)` re-export so sibling modules (`inspect`) can apply
/// the same conversion without touching the thread-local directly.
#[inline]
pub(super) fn meters_to_display_pub(meters: f32) -> f64 {
    meters_to_display(meters)
}

/// Convert a value the user TYPED (in display unit) back to meters
/// before writing into state.
#[inline]
fn display_to_meters(value: f64) -> f32 {
    current_display_unit().to_meters(value as f32, current_ppm())
}

/// "(m)" or "(px)" suffix for labels that show a length.
#[inline]
fn unit_suffix_paren() -> String {
    format!(" ({})", current_display_unit().suffix())
}

const ROW_H: f32 = 28.0;
/// Top inset that frees space above the title for the drag pill +
/// matches the spacing other panels (Inspector/Hierarchy/Gallery) use
/// between the drag handle and the first label row.
const HEAD_PAD: f32 = 18.0;
const PAD: f32 = 12.0;
const ROW_GAP: f32 = 6.0;
const LABEL_FONT_SIZE: f32 = 13.0;
const TITLE_FONT_SIZE: f32 = 15.0;
/// Column where the widget (right side of a "Label: [widget]" row)
/// starts, measured from the inner-x of the row.
/// Width reserved for the label column in NumberInput rows. Widened
/// from 110 → 150 on 2026-05-15 so the longest labels ("QT bounds max
/// X / Y", "Chunk size (cells)") fit on one line. The remaining
/// `inner_w - LABEL_COL_W` (≈ 130 at the default 304-px panel width)
/// is more than enough for a 4-digit NumberInput.
const LABEL_COL_W: f32 = 150.0;

/// Default panel rect when first opened — sized for title + 4
/// sections (Kind + per-kind config + Snap + Display + Inspect).
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    // Width matches Inspector (`style::INSPECTOR_W = 304`) per Enio's
    // 2026-05-15 redesign so the two floating panels read as siblings.
    let w = 304.0_f32.min(viewport_w - 16.0);
    let h = 640.0_f32.min(viewport_h - 16.0).max(440.0);
    let x = ((viewport_w - w) * 0.5).max(8.0);
    let y = ((viewport_h - h) * 0.5).max(8.0);
    Rect::new(x, y, w, h)
}

/// Paint the panel into `rect`. Reads `state` for current values;
/// mutations flow through [`apply_event`] from the dispatcher.
///
/// Layout (top → bottom): title row → big Snap toggle → Kind 3×3
/// button grid → per-kind config → Target vertical stack →
/// Subdivisions → Display (overlay + opacity + color swatch) →
/// Inspect. Body is clipped to the panel rect and scroll-aware:
/// content height is published via `store.set_panel_content_h` so
/// `dispatch_wheel` knows the scroll bound, and the running Y
/// position is offset by `store.panel_scroll(GS_PANEL)`.
fn button_state(store: &WidgetStore, id: crate::NodeId) -> ButtonState {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    }
}

fn toggle_state(store: &WidgetStore, id: crate::NodeId) -> ToggleState {
    match store.get(id) {
        Some(InteractiveState::Toggle { state, .. }) => *state,
        _ => ToggleState::Normal,
    }
}

fn read_number_input(
    store: &WidgetStore,
    id: crate::NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

// =============================================================================
// Event dispatch
// =============================================================================

/// Handle a widget event for the grid-snap panel. Returns `true`
/// when the event mutates `state`. The dispatcher pre-flips Toggle
/// `on` and updates Slider `value` / NumberInput `value` in the
/// store BEFORE calling us; we just mirror those back into state.
pub fn apply_event(state: &mut GridSnapState, event: WidgetEvent, store: &WidgetStore) -> bool {
    match event {
        WidgetEvent::Toggled(id) => apply_toggle(state, id, store),
        WidgetEvent::ValueChanged(id) => apply_value_changed(state, id, store),
        WidgetEvent::Click(id) => apply_click(state, id),
        _ => false,
    }
}

fn active_hex_cfg_mut(state: &mut GridSnapState) -> &mut HexCfg {
    if state.kind == GridKind::StaggeredHex {
        &mut state.staggered_hex_cfg.hex
    } else {
        &mut state.hex_cfg
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

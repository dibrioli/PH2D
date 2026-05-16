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
mod tests {
    use super::*;

    fn populated_store() -> WidgetStore {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        store
    }

    fn flip_toggle(store: &mut WidgetStore, id: crate::NodeId) {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get_mut(id) {
            *on = !*on;
        }
    }

    #[test]
    fn paint_hex_segmented_buttons_register_distinct_rects() {
        // Regression for "click Flat does nothing / Offset stuck on
        // OddR" reported 2026-05-15: each of the 6 hex segmented
        // buttons (Pointy, Flat, OddR, EvenR, OddQ, EvenQ) must have
        // its own non-overlapping rect in hit_index after a paint.
        use crate::interaction::HitIndex;
        let mut text_system = TextSystem::default();
        let mut scene = VectorScene::default();
        let mut hit_index = HitIndex::default();
        let store = populated_store();
        let state = GridSnapState {
            kind: GridKind::Hex,
            panel_visible: true,
            ..Default::default()
        };
        let panel_rect = Rect::new(100.0, 50.0, 304.0, 800.0);
        paint(
            panel_rect,
            &mut scene,
            &mut text_system,
            Theme::default(),
            &mut hit_index,
            &store,
            &state,
        );

        let ids_under_test = [
            ("Pointy", ids::GS_CFG_HEX_POINTY),
            ("Flat", ids::GS_CFG_HEX_FLAT),
            ("OddR", ids::GS_CFG_HEX_OFFSET_ODDR),
            ("EvenR", ids::GS_CFG_HEX_OFFSET_EVENR),
            ("OddQ", ids::GS_CFG_HEX_OFFSET_ODDQ),
            ("EvenQ", ids::GS_CFG_HEX_OFFSET_EVENQ),
        ];
        let mut rects: Vec<(&str, crate::NodeId, Rect)> = Vec::new();
        for (label, id) in ids_under_test {
            let r = hit_index
                .rect_for(id)
                .unwrap_or_else(|| panic!("{label} ({id:?}) not registered in hit_index"));
            rects.push((label, id, r));
        }
        // No two buttons may share the same rect, and clicking the
        // center of each rect must hit the matching id.
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                assert_ne!(
                    rects[i].2, rects[j].2,
                    "{} and {} share rect {:?}",
                    rects[i].0, rects[j].0, rects[i].2
                );
            }
            let (label, id, r) = rects[i];
            let cx = r.x + r.w * 0.5;
            let cy = r.y + r.h * 0.5;
            let hit = hit_index.hit(cx, cy);
            assert_eq!(
                hit,
                Some(id),
                "click on {label} center ({cx}, {cy}) hit {hit:?}, expected {id:?}"
            );
        }
    }

    #[test]
    fn apply_event_close_clears_visible() {
        let mut s = GridSnapState {
            panel_visible: true,
            ..Default::default()
        };
        let store = populated_store();
        assert!(apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CLOSE),
            &store
        ));
        assert!(!s.panel_visible);
    }

    #[test]
    fn apply_event_kind_option_click_sets_active_kind() {
        let mut s = GridSnapState::default();
        let store = populated_store();
        assert_eq!(s.kind, GridKind::Square);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_HEX), &store);
        assert_eq!(s.kind, GridKind::Hex);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_VORONOI), &store);
        assert_eq!(s.kind, GridKind::Voronoi);
    }

    #[test]
    fn apply_event_chip_click_is_passthrough_for_dispatcher() {
        // The dispatcher handles Dropdown.open toggling on chip
        // clicks — apply_event must NOT mutate kind on a chip click.
        let mut s = GridSnapState::default();
        let store = populated_store();
        let before = s.kind;
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_DROPDOWN), &store);
        assert_eq!(s.kind, before);
    }

    #[test]
    fn apply_event_snap_toggle_via_toggled() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        flip_toggle(&mut store, ids::GS_SNAP_ENABLED);
        apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SNAP_ENABLED), &store);
        assert!(s.snap_enabled);
    }

    #[test]
    fn apply_event_overlay_toggle_via_toggled() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        let initial = s.show_overlay;
        flip_toggle(&mut store, ids::GS_SHOW_OVERLAY);
        apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SHOW_OVERLAY), &store);
        assert_eq!(s.show_overlay, !initial);
    }

    #[test]
    fn apply_event_unrelated_id_returns_false() {
        let mut s = GridSnapState::default();
        let store = populated_store();
        let unrelated = crate::NodeId(42);
        assert!(!apply_event(&mut s, WidgetEvent::Click(unrelated), &store));
    }

    #[test]
    fn apply_event_number_changed_writes_to_square_cell_size() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        // Set the store's value as the dispatcher would, then fire
        // ValueChanged.
        if let Some(InteractiveState::NumberInput {
            value,
            last_committed,
            buffer,
            ..
        }) = store.get_mut(ids::GS_CFG_CELL_SIZE)
        {
            *value = 4.5;
            *last_committed = 4.5;
            *buffer = "4.5".to_string();
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_CELL_SIZE),
            &store,
        );
        assert!((s.square_cfg.cell_size - 4.5).abs() < 1e-5);
    }

    #[test]
    fn apply_event_neighborhood_options_set_value() {
        // Post-segmented-button semantics: each option id explicitly
        // SETS its value (Von4 / Moore8), no cycling. Idempotent.
        let mut s = GridSnapState::default();
        let store = populated_store();
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_8),
            &store,
        );
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Moore8);
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_4),
            &store,
        );
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
    }

    #[test]
    fn apply_event_hex_orientation_options_set_value() {
        let mut s = GridSnapState {
            kind: GridKind::Hex,
            ..Default::default()
        };
        let store = populated_store();
        // Default is Pointy. Clicking Flat sets it; clicking Pointy
        // again restores it. No cycle behavior — each option lands
        // on its own value.
        apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_FLAT), &store);
        assert_eq!(s.hex_cfg.orientation, HexOrientation::Flat);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_POINTY), &store);
        assert_eq!(s.hex_cfg.orientation, HexOrientation::Pointy);
    }

    #[test]
    fn apply_event_hex_offset_options_set_value() {
        let mut s = GridSnapState {
            kind: GridKind::Hex,
            ..Default::default()
        };
        let store = populated_store();
        for (id, expected) in [
            (ids::GS_CFG_HEX_OFFSET_EVENR, HexOffset::EvenR),
            (ids::GS_CFG_HEX_OFFSET_ODDQ, HexOffset::OddQ),
            (ids::GS_CFG_HEX_OFFSET_EVENQ, HexOffset::EvenQ),
            (ids::GS_CFG_HEX_OFFSET_ODDR, HexOffset::OddR),
        ] {
            apply_event(&mut s, WidgetEvent::Click(id), &store);
            assert_eq!(s.hex_cfg.offset_variant, expected);
        }
    }

    #[test]
    fn apply_event_hex_orientation_routes_to_staggered_hex_when_active() {
        let mut s = GridSnapState {
            kind: GridKind::StaggeredHex,
            ..Default::default()
        };
        let store = populated_store();
        apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_FLAT), &store);
        assert_eq!(
            s.staggered_hex_cfg.hex.orientation,
            HexOrientation::Flat,
            "StaggeredHex should receive the orientation update"
        );
        // Plain hex_cfg untouched.
        assert_eq!(s.hex_cfg.orientation, HexOrientation::Pointy);
    }

    #[test]
    fn apply_event_stagger_parity_options_set_value() {
        let mut s = GridSnapState {
            kind: GridKind::StaggeredSquare,
            ..Default::default()
        };
        let store = populated_store();
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_STAGGER_PARITY_EVEN),
            &store,
        );
        assert_eq!(s.staggered_square_cfg.parity, StaggerParity::EvenRows);
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_STAGGER_PARITY_ODD),
            &store,
        );
        assert_eq!(s.staggered_square_cfg.parity, StaggerParity::OddRows);
    }

    #[test]
    fn apply_event_layer_options_flip_grid_in_front() {
        let mut s = GridSnapState::default();
        let store = populated_store();
        assert!(s.grid_in_front, "default is In front");
        apply_event(&mut s, WidgetEvent::Click(ids::GS_LAYER_BEHIND), &store);
        assert!(!s.grid_in_front);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_LAYER_IN_FRONT), &store);
        assert!(s.grid_in_front);
    }

    #[test]
    fn apply_event_color_r_writes_state() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_COLOR_R)
        {
            *value = 200.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_R),
            &store,
        );
        assert_eq!(s.color_rgba[0], 200);
        // Other channels untouched.
        let defaults = GridSnapState::default();
        assert_eq!(s.color_rgba[1], defaults.color_rgba[1]);
        assert_eq!(s.color_rgba[2], defaults.color_rgba[2]);
        assert_eq!(s.color_rgba[3], defaults.color_rgba[3]);
    }

    #[test]
    fn apply_event_subdivisions_clamps_to_unit_floor() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_SNAP_SUBDIVISIONS)
        {
            *value = 0.0; // floor enforced to 1
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_SNAP_SUBDIVISIONS),
            &store,
        );
        assert_eq!(s.snap_subdivisions, 1);
    }

    #[test]
    fn apply_event_subdivisions_actually_subdivides_snap() {
        // With cell_size=1 and subdivisions=2, snap_to_center should
        // pull to half-cell centers. World [0.3, 0.3] is in cell
        // (0, 0) of the sub-grid (cells span 0..0.5) → center
        // (0.25, 0.25), not (0.5, 0.5).
        let mut s = GridSnapState {
            snap_enabled: true,
            snap_target: SnapTarget::Center,
            snap_subdivisions: 2,
            ..Default::default()
        };
        let p = s.snap_world([0.3, 0.3], [0.0, 0.0]);
        assert!(
            (p[0] - 0.25).abs() < 1e-5 && (p[1] - 0.25).abs() < 1e-5,
            "expected [0.25, 0.25] with subdivisions=2; got {p:?}"
        );
    }

    #[test]
    fn apply_event_probe_a_x_writes_state() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(ids::GS_PROBE_A_X)
        {
            *value = 7.5;
        }
        apply_event(&mut s, WidgetEvent::ValueChanged(ids::GS_PROBE_A_X), &store);
        assert!((s.probe_a[0] - 7.5).abs() < 1e-5);
    }

    #[test]
    fn apply_event_qt_bounds_max_soft_clamps_to_min_plus_eps() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        // Try to set MAX below MIN — should snap up to min+eps.
        let min_x = s.quadtree_cfg.bounds.min[0];
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_QT_BOUNDS_MAX_X)
        {
            *value = min_x as f64 - 5.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_QT_BOUNDS_MAX_X),
            &store,
        );
        assert!(
            s.quadtree_cfg.bounds.max[0] > s.quadtree_cfg.bounds.min[0],
            "MAX must stay > MIN after clamp; got min={} max={}",
            s.quadtree_cfg.bounds.min[0],
            s.quadtree_cfg.bounds.max[0]
        );
    }

    #[test]
    fn apply_event_color_clamps_above_255() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_COLOR_B)
        {
            *value = 999.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_B),
            &store,
        );
        assert_eq!(s.color_rgba[2], 255);
    }

    #[test]
    fn apply_event_voronoi_reseed_changes_rng_seed() {
        let mut s = GridSnapState {
            kind: GridKind::Voronoi,
            ..Default::default()
        };
        let store = populated_store();
        let before = s.voronoi_cfg.rng_seed;
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_VORONOI_RESEED),
            &store,
        );
        assert_ne!(s.voronoi_cfg.rng_seed, before);
    }

    #[test]
    fn opacity_slider_value_changed_clamps_to_unit() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(ids::GS_OPACITY_SLIDER)
        {
            *value = 1.5;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_OPACITY_SLIDER),
            &store,
        );
        assert!((s.opacity - 1.0).abs() < 1e-5);
    }
}

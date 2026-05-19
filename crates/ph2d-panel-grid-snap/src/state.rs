//! Grid Snap panel state — owned by `ErasedPanel<GridSnapPanel>` after
//! ADR-0029 Phase C.4. Pre-Phase-C.4 the panel-chrome flags lived on
//! `HeroScreen::grid.snap_state.panel_visible` and
//! `HeroScreen::grid.snap_state.panel_rect`.
//!
//! Phase C.4 split:
//!
//! - `panel_visible` → moved to `HeroScreen::panel_visibility` map
//!   keyed by `GridSnapPanel::ID` ("grid_snap").
//! - `panel_rect` → REMAINS on `crate::grid_snap::GridSnapState`
//!   (the canvas-renderer state). Splitting it out across two
//!   structs would force every site that touches `GridSnapState`
//!   to also reach into a separate `GridSnapPanelState`; the
//!   canvas renderer doesn't read it, but the panel paint thunk
//!   does need to write back the lazy default. Phase C.4 keeps
//!   `panel_rect` on `GridSnapState` and reads/writes it via
//!   `host.store_mut()` accessors below — `GridSnapPanelState` is
//!   intentionally a marker struct.
//!
//! Thread-locals that the legacy `grid_snap/panel/mod.rs` defined
//! (`LAST_CONTENT_H`, `LAST_VISIBLE_H`, `CURRENT_DISPLAY_UNIT`,
//! `CURRENT_PPM`) move here so the panel's `paint` and `apply_event`
//! can still publish + consume them without depending on editor-core
//! internals.

use ph2d_editor_core::project::{DEFAULT_PIXELS_PER_METER, DisplayUnit};
use std::cell::Cell;

thread_local! {
    /// Last computed scrollable content height of the panel (set by
    /// `paint`, read by the orchestrator's content_h publish).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
    /// Active project DisplayUnit + pixels-per-meter for THIS paint /
    /// apply_event call. State storage is always meters; this pair
    /// drives the meter↔display conversion at the panel boundary.
    /// `paint`/`apply_event` call `set_current_display_unit` first.
    static CURRENT_DISPLAY_UNIT: Cell<DisplayUnit> =
        const { Cell::new(DisplayUnit::Meters) };
    static CURRENT_PPM: Cell<f32> =
        const { Cell::new(DEFAULT_PIXELS_PER_METER) };
}

/// Retained state slot for `GridSnapPanel`. Intentionally empty —
/// the canvas-renderer `GridSnapState` (lives in editor-core) holds
/// the per-kind config + `panel_rect`. Phase C.4 keeps that fork to
/// avoid splitting a fat struct just for the sake of trait purity.
///
/// `Default` is required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct GridSnapPanelState;

/// Last computed content height — published via
/// `store.set_panel_content_h(GS_PANEL, …)` at the end of paint.
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

/// Last computed visible height — published via
/// `store.set_panel_visible_h(GS_PANEL, …)` at the end of paint.
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

/// Internal setter used by `paint` once the body has been measured.
pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

/// Internal setter used by `paint` once the body has been measured.
pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

/// Set by `paint` / `apply_event` once per call BEFORE any inner
/// painter or event helper reads the unit. Mirrors the pre-Phase-C.4
/// thread-local pair from `grid_snap/panel/mod.rs`.
pub fn set_current_display_unit(unit: DisplayUnit, pixels_per_meter: f32) {
    CURRENT_DISPLAY_UNIT.with(|c| c.set(unit));
    CURRENT_PPM.with(|c| c.set(pixels_per_meter));
}

pub(crate) fn current_display_unit() -> DisplayUnit {
    CURRENT_DISPLAY_UNIT.with(|c| c.get())
}

pub(crate) fn current_ppm() -> f32 {
    CURRENT_PPM.with(|c| c.get())
}

/// Convert a sim-stored meter value to the value to DISPLAY in a
/// NumberInput (under the active DisplayUnit).
#[inline]
pub(crate) fn meters_to_display(meters: f32) -> f64 {
    current_display_unit().from_meters(meters, current_ppm()) as f64
}

/// Convert a value the user TYPED (in display unit) back to meters
/// before writing into state.
#[inline]
pub(crate) fn display_to_meters(value: f64) -> f32 {
    current_display_unit().to_meters(value as f32, current_ppm())
}

/// "(m)" or "(px)" suffix for labels that show a length.
#[inline]
pub(crate) fn unit_suffix_paren() -> String {
    format!(" ({})", current_display_unit().suffix())
}

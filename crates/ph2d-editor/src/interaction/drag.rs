//! Drag/stepper/long-press state types + timing constants.
//!
//! Extracted from [`super::state`] as part of the post-M14 refactor
//! (Track E1). Re-exported via `interaction::mod` so external
//! call-sites continue to use `interaction::NumberInputDragState`
//! etc. unchanged.
//!
//! All types here are tiny `Copy` snapshots seeded by pointer-Down
//! and consumed by pointer-Move/Up/tick. None of them own heap
//! state — HR-3 safe.

use ph2d_a11y::NodeId;

/// State of an in-progress drag on a NumberInput field (M14.A).
///
/// Down on the box body seeds this with `crossed_threshold = false`
/// and the rest of the snapshot. Pointer-move flips the flag once
/// the cursor moves > 4 px from the start, then applies a value
/// delta on every subsequent move. Up either commits the new value
/// (when the threshold was crossed) or falls through to caret-place
/// + focus (when it wasn't — the click-to-edit path).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NumberInputDragState {
    /// NumberInput being dragged.
    pub id: NodeId,
    /// Cursor x/y at the moment of Down.
    pub start_x: f32,
    pub start_y: f32,
    /// Value snapshotted at Down. The drag computes `new = start +
    /// (dx * h_rate + (-dy) * v_rate) * shift_mul * step`.
    pub start_value: f64,
    /// Cached step from the buffer ("contains '.'" → 0.01, else 1.0)
    /// at Down — kept stable for the drag duration so behavior stays
    /// predictable across mid-drag value changes.
    pub step: f64,
    /// Set to `true` once cursor moves > 4 px from start. Below
    /// threshold, the Down is still a candidate for the "click to
    /// edit" path; past threshold it's committed to slider mode.
    pub crossed_threshold: bool,
    /// **Axis lock**, decided once at the moment `crossed_threshold`
    /// flips. `true` = horizontal-only (`dy` zeroed), `false` =
    /// vertical-only (`dx` zeroed). Stays fixed for the rest of the
    /// drag — a new click (Down → end_number_input_drag → fresh
    /// state) is the only way to reset the axis. This prevents the
    /// off-axis from contaminating the scrub even if the user wobbles
    /// past the original dominance late in the drag.
    pub axis_horizontal: bool,
    /// The byte offset at which the deferred caret-place lands if
    /// Up arrives before threshold is crossed (click-to-edit path).
    pub caret_offset_at_down: usize,
}

/// State of an in-progress continuous-hold on a NumberInput stepper
/// arrow (M14.A).
///
/// Repeats the up / down increment while the pointer stays inside the
/// arrow rect. Initial 250 ms delay, then 30 ms repeat — matches
/// macOS Aqua and most desktop text-field steppers. The host calls
/// [`super::dispatch_tick`] each frame to drive the repeat.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NumberStepperHoldState {
    /// NumberInput whose value is being adjusted.
    pub id: NodeId,
    /// `+1.0` for up arrow, `-1.0` for down arrow.
    pub direction: f64,
    /// Cached step from the buffer (same heuristic as drag mode).
    pub step: f64,
    /// `timestamp_ns` of the Down event.
    pub press_ns: u128,
    /// `timestamp_ns` of the most recent tick that fired (initially
    /// equal to `press_ns` — the Down itself counts as the first
    /// tick). The repeat path checks `now - last_tick_ns` against the
    /// `STEPPER_REPEAT_INTERVAL_NS` budget.
    pub last_tick_ns: u128,
}

/// Initial delay (ns) before continuous-hold repeats start firing.
/// `250 ms` — match macOS Aqua text-field stepper feel.
pub const STEPPER_HOLD_INITIAL_DELAY_NS: u128 = 250_000_000;

/// Repeat interval (ns) once the initial delay elapsed. `30 ms` —
/// ~33 ticks per second, a comfortable fast-but-readable rate.
pub const STEPPER_REPEAT_INTERVAL_NS: u128 = 30_000_000;

/// Distance (in physical px) the cursor must move from the Down
/// position before a NumberInput drag flips into slider mode.
pub const NUMBER_INPUT_DRAG_THRESHOLD_PX: f32 = 4.0;

/// Pixels-per-step rates for the drag-slider mode (Blender-style).
///
/// - **Horizontal (`DRAG_RATE_X`)**: 50 step-units per cursor pixel
///   moved right (negative when moved left). Fast — small drag covers
///   large range.
/// - **Vertical (`DRAG_RATE_Y`)**: 5 step-units per cursor pixel
///   moved **up** (`-dy`). Slow — for precision.
/// - **Shift multiplier (`DRAG_SHIFT_MUL`)**: when Shift is held,
///   multiply the combined delta by `0.001`. So horizontal+Shift =
///   `0.05 step/px` (very fine), vertical+Shift = `0.005 step/px`
///   (ultra-fine).
pub const DRAG_RATE_X: f64 = 50.0;
pub const DRAG_RATE_Y: f64 = 5.0;
pub const DRAG_SHIFT_MUL: f64 = 0.001;

/// Internal state of an in-progress hierarchy drag.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HierarchyDragState {
    /// Row being dragged.
    pub dragged: NodeId,
    /// Cursor x/y at Down — used to detect "drag started" via the
    /// distance threshold.
    pub down_x: f32,
    pub down_y: f32,
    /// Latest cursor x/y (updated on Move) so the painter can render
    /// a drop-indicator that matches what the dispatch will resolve
    /// on Up (x-aware to distinguish "inside indented row" from
    /// "sibling at root level").
    pub cursor_x: f32,
    pub cursor_y: f32,
    /// `true` once the cursor has moved past the threshold; until
    /// then the gesture is "maybe-click, maybe-drag".
    pub active: bool,
    /// Wall-clock timestamp the Down event landed at (ns). Used by
    /// the Up handler to detect long-press (Up - Down >= 600 ms with
    /// `!active`) → emits `WidgetEvent::LongPress` for inline rename.
    pub down_timestamp_ns: u128,
}

/// Hold duration that turns a still pointer-Down on a hierarchy row
/// into a `WidgetEvent::LongPress`. 600 ms matches macOS Finder /
/// iOS rename gestures — short enough to feel responsive, long
/// enough that a regular slow click doesn't accidentally fire it.
pub const LONG_PRESS_THRESHOLD_NS: u128 = 600_000_000;

/// State of an in-progress drag on a scrollbar thumb.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScrollbarDragAnchor {
    /// Panel whose `panel_scroll` the drag updates.
    pub panel: NodeId,
    /// Cursor y at the moment of Down.
    pub cursor_y_at_down: f32,
    /// `panel_scroll(panel)` at the moment of Down.
    pub scroll_at_down: f32,
    /// Track height used to convert cursor delta → scroll delta.
    pub track_h: f32,
    /// Total content height (= `panel_content_h(panel)`).
    pub content_h: f32,
    /// Visible body height (= `panel_visible_h(panel)`).
    pub visible_h: f32,
}

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
    /// Cursor x/y at the moment of Down. Used ONLY for the initial
    /// threshold check (`crossed_threshold`) — once axis-locked, value
    /// updates use the incremental `last_x` / `last_y` anchor instead
    /// so a reversal after a clamp immediately reverses the value
    /// (the absolute model would keep the value pegged at the clamp
    /// edge until the cursor returned all the way to `start_x`).
    pub start_x: f32,
    pub start_y: f32,
    /// Value snapshotted at Down. Surface for tests / introspection;
    /// the live drag math reads the chip's current `value` and applies
    /// an incremental delta each Move (Blender/AE scrub pattern).
    pub start_value: f64,
    /// Cursor x/y from the PREVIOUS Move (or Down, before any Move
    /// fired). Each Move computes `delta = event - last`, applies the
    /// delta to the chip's current value, then writes `last = event`.
    /// This is the standard "scrub" model — reversal after a clamp
    /// hits zero accumulated dx so the next Move immediately moves
    /// the value in the new direction.
    pub last_x: f32,
    pub last_y: f32,
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
    /// **The CONTINUOUS running value of the scrub** — used only by chips registered via
    /// [`super::WidgetStore::link_slider_number_mapped_integer`].
    ///
    /// Those chips round on every write. The scrub reads the chip's current value back as the
    /// base for the next Move, so the rounding sat INSIDE the feedback loop and the residue was
    /// thrown away every Move: `round(round(v) + d) == round(v)` for any `d < 0.5`, forever.
    ///
    /// The arithmetic made the vertical axis dead, not merely coarse. `DRAG_RANGE_PX_V` is 2500
    /// (it is the PRECISE axis on purpose), so a `1..128` count scrubs at 0.05 units per pixel
    /// and a typical Move carries ~0.15 — never half a unit. Every integer chip in the app was
    /// affected (Enio reported it on the Zig Zag ridges, 2026-07-18).
    ///
    /// So the accumulator is continuous and the snap happens at the WRITE: the chip still shows
    /// an integer, and the drag still travels. It is clamped exactly like the written value, so
    /// a reversal after hitting a bound still moves on the very next Move.
    ///
    /// ⚠️ **Continuous chips do not read this** — they keep reading the chip back, byte for byte
    /// as before. The residue only exists where something discards it.
    pub accum: f64,
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

/// **Range-proportional** drag pixels (used when the box has a registered `number_range`): the cursor
/// displacement spans the WHOLE `[min, max]` range over this many pixels — horizontal is fast
/// (`DRAG_RANGE_PX_H`), vertical is precise (`DRAG_RANGE_PX_V`, ~10× the distance). So the scrub is
/// proportional to the box's range, NOT the value magnitude — a `±1` box no longer races past 100 on a
/// few pixels (Enio 2026-06-25). Shift still multiplies by [`DRAG_SHIFT_MUL`] for super-precision.
pub const DRAG_RANGE_PX_H: f64 = 250.0;
pub const DRAG_RANGE_PX_V: f64 = 2500.0;

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
/// into a `WidgetEvent::LongPress`. 400 ms — Enio 2026-05-26 pediu
/// pra reduzir um pouco do 600 ms anterior (macOS Finder padrão).
/// Continua longo o suficiente pra não disparar em click normal.
pub const LONG_PRESS_THRESHOLD_NS: u128 = 400_000_000;

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

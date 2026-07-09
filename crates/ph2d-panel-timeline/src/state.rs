//! Timeline panel state + the shell→panel view snapshot.
//!
//! The authoritative timeline document lives in the shell (`AppGfx.timeline`);
//! each frame the shell publishes a [`TimelineViewSnapshot`] via
//! [`set_current_timeline`] BEFORE the panel paints, and `paint` reads it. Edits
//! flow back as `TimelineIntent`s the shell drains (mirror of the vector/motion
//! panels). Per-instance state holds only view transform (pan/zoom of the time
//! axis), which is panel-local and not undoable.

use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until the
    /// first push (panel paints an empty timeline).
    static CURRENT_SNAPSHOT: RefCell<Option<TimelineViewSnapshot>> = const { RefCell::new(None) };
    /// Dope-sheet edit intents the panel raised this frame (key select / move /
    /// clear), drained by the shell into its `timeline_intents` (mirror of the
    /// motion-graph panel's thread-local `INTENTS`). Transport events still flow
    /// through the widget bus as `PanelEvent`s; only the surface-gesture edits —
    /// which carry a `(target, key)` identity `PanelEvent` cannot express — use
    /// this channel.
    static INTENTS: RefCell<Vec<TimelineIntent>> = const { RefCell::new(Vec::new()) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus header + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
    /// A pending "fit the view to the keys" request (F over the panel). The
    /// shortcut is read by the shell's keyboard handler, but the view transform
    /// it fits is panel state — so the shell raises the request here and `paint`
    /// consumes it once the time area's pixel width is known.
    static FIT_REQUESTED: Cell<bool> = const { Cell::new(false) };
    /// A pending "pan the time axis until the playhead is visible" request,
    /// raised by the shell after a transport command that jumps the playhead
    /// (go-to-start/end, a frame step, a typed time). Consumed by `paint`, which
    /// alone knows the visible span.
    static REVEAL_REQUESTED: Cell<bool> = const { Cell::new(false) };
}

/// Retained per-instance state for `TimelinePanel`: the horizontal view of the
/// time axis (pan + zoom). Wired in E6; `Default` satisfies the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug)]
pub struct TimelinePanelState {
    /// Seconds at the left edge of the lanes area (pan).
    pub view_start_s: f64,
    /// Pixels per second (zoom). `> 0`.
    pub px_per_s: f64,
    /// Seconds currently visible across the ruler width. Written by `paint`
    /// (from the ruler pixel width ÷ zoom) and read by `event` to map a ruler
    /// scrub value `0..1` back to an absolute time.
    pub view_span_s: f64,
    /// Whether the "+Track" property dropdown is open (panel-local; toggled by
    /// the +Track button, closed on picking a property).
    pub add_track_open: bool,
    /// In-progress key-diamond drag (dope-sheet), while the pointer is down on a
    /// key: the pointer x at Begin and the latest x, so `paint` can draw the
    /// selected diamonds shifted by the live delta and `event`/`interact` can
    /// emit the final `MoveSelectedKeys` on End. `None` when not dragging keys.
    pub key_drag: Option<KeyDrag>,
    /// Committed-move preview (px) held for exactly the End frame, so the
    /// selected diamonds stay at the dropped position across the one-frame gap
    /// between raising `MoveSelectedKeys` and the shell re-publishing the moved
    /// snapshot — without it the keys flash back to their old spot for a frame.
    /// Cleared at the top of the next `interact::process` (snapshot has caught up).
    pub pending_move_dx: Option<f32>,
    /// Vertical scroll of the track rows, in px from the top of the list.
    pub scroll_y: f32,
    /// Scrollable overflow (`content_h - rows_h`), recomputed by `paint`. Kept so
    /// the wheel/scrollbar can clamp before `paint` re-measures.
    pub scroll_max: f32,
    /// Middle-drag pan anchor (last pointer position) while the wheel button is
    /// held over the dope sheet.
    pub pan_drag: Option<(f32, f32)>,
    /// In-progress box-select (marquee) drag over an empty lane.
    pub box_drag: Option<BoxDrag>,
    /// A box-select that just finished, waiting to be resolved against the key
    /// diamonds. Set by `interact` at End, consumed by `paint` the SAME frame —
    /// only `paint` knows the row geometry a key's `y` depends on.
    pub box_commit: Option<BoxDrag>,
    /// User-resized panel rect. `None` = use the docked rect from the layout.
    pub rect: Option<Rect>,
    /// In-progress edge/corner resize drag.
    pub resize: Option<ResizeDrag>,
}

/// An in-progress resize: which edges move, and the rect + pointer at Begin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeDrag {
    /// Bitmask of the edges being dragged (`geom::EDGE_*`).
    pub edges: u8,
    /// The panel rect when the drag began (deltas apply to THIS, not to the
    /// live rect, so a slow drag never accumulates rounding).
    pub start_rect: Rect,
    /// The pointer position when the drag began.
    pub start_pointer: (f32, f32),
}

/// An in-progress box-select: the pointer at Begin and the latest pointer, in
/// global px. Keys whose diamond centre falls inside [`BoxDrag::rect`] join the
/// selection; without `additive` (Shift) the previous selection is replaced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxDrag {
    /// Pointer position when the drag began.
    pub start: (f32, f32),
    /// Latest pointer position.
    pub cur: (f32, f32),
    /// Shift was held at Begin: add to the selection instead of replacing it.
    pub additive: bool,
}

impl BoxDrag {
    /// The marquee as a normalized rect (any drag direction).
    #[must_use]
    pub fn rect(&self) -> Rect {
        let (x0, x1) = (self.start.0.min(self.cur.0), self.start.0.max(self.cur.0));
        let (y0, y1) = (self.start.1.min(self.cur.1), self.start.1.max(self.cur.1));
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

/// An in-progress dope-sheet key drag: pointer x at Begin + the latest x. The
/// time delta is `(cur_x - start_x) / px_per_s`, frame-snapped on commit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyDrag {
    /// Pointer x (global px) when the drag began.
    pub start_x: f32,
    /// Latest pointer x (global px).
    pub cur_x: f32,
    /// The pressed key was already part of a multi-selection and was pressed
    /// without Shift: keep the whole selection so a drag moves the group, but
    /// collapse to just this key on a plain click (no drag) — the standard
    /// dope-sheet disambiguation. `None` when the press already set the
    /// selection (an unselected key, or Shift-toggle).
    pub collapse_to: Option<ph2d_timeline::SelectedKey>,
}

impl Default for TimelinePanelState {
    fn default() -> Self {
        Self {
            view_start_s: 0.0,
            px_per_s: DEFAULT_PX_PER_S,
            view_span_s: 0.0,
            add_track_open: false,
            key_drag: None,
            pending_move_dx: None,
            scroll_y: 0.0,
            scroll_max: 0.0,
            pan_drag: None,
            box_drag: None,
            box_commit: None,
            rect: None,
            resize: None,
        }
    }
}

/// Default zoom: pixels per second of timeline.
pub const DEFAULT_PX_PER_S: f64 = 120.0; // LITERAL-PX-OK: default time-axis zoom (px per second), a functional view scale, not a design spacing token
/// Zoomed all the way out: ~2 px per second (a long clip fits the strip).
pub const MIN_PX_PER_S: f64 = 2.0; // LITERAL-PX-OK: time-axis zoom floor
/// Zoomed all the way in: 4000 px per second (sub-frame precision at 60 fps).
pub const MAX_PX_PER_S: f64 = 4000.0; // LITERAL-PX-OK: time-axis zoom ceiling

/// Publish the current timeline view snapshot. Called by the shell once per
/// frame; pass `None` to clear.
pub fn set_current_timeline(snapshot: Option<TimelineViewSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// The snapshot the host published this frame, or a default empty one.
pub(crate) fn current_snapshot() -> TimelineViewSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().clone().unwrap_or_default())
}

/// Raise a dope-sheet edit intent (called by `interact` while draining gestures).
pub(crate) fn push_intent(intent: TimelineIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the dope-sheet edit intents raised since the last call. The shell calls
/// this each frame and feeds them through `apply_intent` (capacity-retaining).
#[must_use]
pub fn drain_intents() -> Vec<TimelineIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Ask the panel to fit its time view to the extent of the keys on the next
/// paint (the `F` shortcut, raised by the shell while the cursor is over the
/// panel — Blender's per-area focus).
pub fn request_fit() {
    FIT_REQUESTED.with(|c| c.set(true));
}

/// Consume a pending fit request.
pub(crate) fn take_fit_request() -> bool {
    FIT_REQUESTED.with(|c| c.replace(false))
}

/// Ask the panel to pan the time axis until the playhead is visible on the next
/// paint. Raised by the shell right after it queues a transport command that
/// jumps the playhead — the panel only page-follows while PLAYING, so a paused
/// go-to-end would otherwise leave the view where it was.
pub fn request_reveal_playhead() {
    REVEAL_REQUESTED.with(|c| c.set(true));
}

/// Consume a pending reveal request.
pub(crate) fn take_reveal_request() -> bool {
    REVEAL_REQUESTED.with(|c| c.replace(false))
}

/// Last scrollable content height measured by `paint`.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Last visible body height measured by `paint`.
#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

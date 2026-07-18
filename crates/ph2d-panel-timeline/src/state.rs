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
    /// **The panel is on the Keys tab** — published every paint, read by the shell
    /// next frame. This is the panel→shell mirror of the tab (the reverse of
    /// [`CURRENT_SNAPSHOT`]): the shell needs it to drive the CLIP playhead and solo
    /// the active clip while the animator edits keys (the AE precomp model, Enio
    /// 2026-07-16). A view bit, not an intent — it names WHICH clock the timeline
    /// runs on, not a document edit. `false` (Arrange / normal) until the first
    /// paint, so a hidden panel leaves the timeline on its usual timeline clock.
    static KEYS_MODE: Cell<bool> = const { Cell::new(false) };
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
    /// The move emitted THIS frame but not yet visible in the published snapshot
    /// (intents land one frame later). The selected diamonds ride it so they do
    /// not lag the cursor by a frame. Cleared at the top of the next
    /// `interact::process`, by when the snapshot has caught up.
    pub pending_move_dx: Option<f32>,
    /// Vertical scroll of the track rows, in px from the top of the list.
    pub scroll_y: f32,
    /// Scrollable overflow (`content_h - rows_h`), recomputed by `paint`. Kept so
    /// the wheel/scrollbar can clamp before `paint` re-measures.
    pub scroll_max: f32,
    /// Middle-drag pan anchor (last pointer position) while the wheel button is
    /// held over the dope sheet.
    pub pan_drag: Option<(f32, f32)>,
    /// Width of the track-name column, dragged by the splitter. Clamped into the
    /// panel's bounds on every paint (`geom::clamp_label_w`), so a resize of the
    /// panel itself never strands it.
    pub label_w: f32,
    /// In-progress splitter drag: `(label_w, pointer x)` at Begin. Deltas apply
    /// to THOSE, so a slow drag never accumulates rounding.
    pub label_drag: Option<(f32, f32)>,
    /// Height of every expanded row's graph band, dragged by the grip along its
    /// bottom edge. Shared by all of them, so one drag resizes the lot.
    pub graph_h: f32,
    /// In-progress graph-height drag: `(graph_h, pointer y)` at Begin.
    pub graph_resize: Option<(f32, f32)>,
    /// Tracks (by raw `AnimTarget`) whose graph editor is expanded. Panel-local
    /// view state — never undoable, never saved.
    pub expanded: Vec<u64>,
    /// Speed-graph view (W5): every expanded graph band plots VELOCITY
    /// (`d(value)/dt`) instead of the value curve, toggled from the transport
    /// bar. Panel-local view state — never undoable, never saved.
    pub speed_view: bool,
    /// Which half of the document is on screen: the clip's keys, or the clip
    /// stack ([`crate::tab::Tab`]). Panel-local VIEW state — never undoable,
    /// never saved, and never the meaning of an edit.
    pub tab: crate::tab::Tab,
    /// The Summary channel's column lock, toggled by its padlock. **Open by
    /// default** (Enio, 2026-07-11): clicking a key selects just that key.
    /// Close it and grabbing any single key grabs its whole time column, so
    /// keys stay vertically aligned. Either way, grabbing the Summary diamond
    /// itself always moves the whole column.
    pub column_lock: bool,
    /// In-progress bézier-handle drag in an expanded track's graph.
    pub handle_drag: Option<HandleDrag>,
    /// In-progress anchor drag in an expanded track's graph (W3.E5) — the
    /// gesture that retunes a key's VALUE.
    pub anchor_drag: Option<AnchorDrag>,
    /// The Summary column the pointer went down on, and whether it was already
    /// fully selected — a plain click on such a column collapses the selection
    /// to it, while a drag keeps whatever else was selected. `None` when the
    /// press did not start on the Summary channel.
    pub summary_press: Option<SummaryPress>,
    /// In-progress loop-brace drag on the ruler (W4.T3): which edge and the range
    /// captured at Begin, so a slow drag applies deltas to a fixed origin.
    pub loop_drag: Option<LoopDrag>,
    /// The clip strip being dragged or trimmed, if any.
    pub strip_drag: Option<StripDrag>,
    /// A lane weight field is being edited (dragged or typed) and an undo bracket
    /// is open for it. Without it, dispatch's per-Move `ValueChanged` would make
    /// each frame of the drag its own atomic undo step — sliding the weight across
    /// its range would leave dozens of Ctrl+Z steps behind. Every other
    /// document-mutating gesture in this panel brackets; this was the one that did
    /// not.
    pub weight_edit: Option<usize>,

    /// Storage index of the marker being dragged on the ruler (W4.T3), if any.
    pub marker_drag: Option<usize>,
    /// The marker whose inline rename field is open (W4.T3), opened by a
    /// double-click on its pennant. `None` when no rename is in flight.
    pub marker_rename: Option<MarkerRename>,
    /// The clip whose inline rename field is open (W5), opened by the pencil in
    /// the transport bar. `None` when no rename is in flight. Same shape as
    /// [`MarkerRename`] and for the same reason: the text lives in the
    /// `WidgetStore`, so this only remembers WHICH clip and whether the field has
    /// been seeded (re-seeding every frame would stomp the user's typing).
    pub clip_rename: Option<ClipRename>,
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

/// An in-progress bézier-handle drag. The pointer is recorded in global px; the
/// band's value↔pixel mapping only exists during `paint`, so `graph::resolve_drag`
/// turns this into a `SetInterp` there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HandleDrag {
    /// Raw `AnimTarget` of the track being edited.
    pub target: u64,
    /// Raw `KeyId` of the segment's OUTGOING key (the one owning the `Interp`).
    pub key: u64,
    /// `0` = out handle (`P1`), `1` = in handle (`P2`).
    pub which: u8,
    /// Latest pointer x (global px).
    pub x: f32,
    /// Latest pointer y (global px).
    pub y: f32,
    /// The band's value range, frozen on the drag's first paint. A band that
    /// refit every frame would feed back into the value the pointer maps to —
    /// drag the handle up, the range grows, the same screen y now means a
    /// smaller value, and the handle crawls away from the cursor. `None` until
    /// the first paint resolves it.
    pub range: Option<(f64, f64)>,
    /// The gesture ended this frame: `paint` resolves it once more, pushes
    /// `EndEdit` to close the undo bracket, and clears the drag.
    pub ending: bool,
}

/// An in-progress anchor drag in an expanded track's graph: the gesture that
/// edits a key's VALUE (W3.E5).
///
/// The value axis is **band-local** — every row auto-fits its own range — so a
/// pixel offset only carries meaning inside the band it was made in. That is why
/// the drag records the keys it retunes UP FRONT ([`AnchorDrag::base`], one entry
/// per selected key on this track) instead of leaning on the live selection: the
/// sideways half of the same gesture moves the whole selection in time, across
/// tracks, along the shared ruler.
///
/// Each frame re-derives the value from `base + delta`, never from the key's
/// current value: an incremental `v += delta` would round in `f32` once per
/// frame and let a slow drag drift away from the cursor.
#[derive(Clone, Debug, PartialEq)]
pub struct AnchorDrag {
    /// Raw `AnimTarget` of the track whose band is being dragged.
    pub target: u64,
    /// Pointer position (global px) when the drag began.
    pub start: (f32, f32),
    /// Latest pointer position (global px).
    pub cur: (f32, f32),
    /// The keys this drag retunes, as `(raw KeyId, value at Begin)`.
    pub base: Vec<(u64, f32)>,
    /// Total time delta already emitted as `MoveSelectedKeys` — each frame emits
    /// only the difference, so the streamed moves sum to exactly the drag.
    pub applied_s: f64,
    /// The value offset already emitted, so a frame that moved nothing vertically
    /// emits no `SetKeyValue`. `None` until the first resolved frame.
    pub applied_v: Option<f64>,
    /// The pressed key, when it was already part of a multi-selection and was
    /// pressed without Shift: keep the group so a drag moves it, but collapse to
    /// this key on a plain click. Mirrors [`KeyDrag::collapse_to`].
    pub collapse_to: Option<ph2d_timeline::SelectedKey>,
    /// The band's value range, frozen on the drag's first paint — see
    /// [`HandleDrag::range`] for why a refitting band would chase the cursor.
    pub range: Option<(f64, f64)>,
    /// The gesture ended this frame: `paint` resolves it once more, closes the
    /// undo bracket and clears the drag.
    pub ending: bool,
}

/// An in-progress loop-brace drag. `edge` is `0` = start, `1` = end, `2` = the
/// band (move both). The range is captured at Begin so deltas apply to a fixed
/// origin (no drift on a slow drag).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopDrag {
    /// `0` = start handle, `1` = end handle, `2` = body.
    pub edge: u8,
    /// Pointer x (global px) when the drag began.
    pub start_x: f32,
    /// The `(start, end)` loop range in seconds when the drag began.
    pub start_range: (f64, f64),
}

/// An in-progress clip-strip drag. `edge` is `0` = start, `1` = end, `2` = body.
/// The span is captured at Begin so deltas apply to a fixed origin — a drag that
/// reads back its own output drifts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StripDrag {
    /// Which lane the strip sits on.
    pub lane: usize,
    /// The strip's stable identity — NOT its index: the lane re-sorts as the strip
    /// crosses its neighbour, and an index-anchored drag would swap victims mid-air.
    pub id: ph2d_timeline::StripId,
    /// `0` = start edge (trim), `1` = end edge (trim), `2` = body (slide),
    /// `3` = fade-in grip, `4` = fade-out grip (B4).
    pub edge: u8,
    /// Pointer x (global px) when the drag began.
    pub start_x: f32,
    /// The strip's `(t_start, t_end)` when the drag began.
    pub start_span: (f64, f64),
    /// The fade at the dragged edge when the drag began, in seconds — read off the
    /// WEDGE the panel drew (`blend_in`/`blend_out`), never recomputed. Zero for the
    /// gestures that are not a fade.
    ///
    /// Captured at `Begin` like `start_span`, and for the same reason: a drag that
    /// re-reads the value it is writing accumulates its own rounding and drifts
    /// (`arch_no_absolute_drag_pattern`).
    pub start_ease: f64,
    /// Cmd/Ctrl was held at Begin: the edge RETIMES instead of trimming (a rate
    /// stretch). Latched at Begin and never re-read — a modifier released mid-drag
    /// must not change what the gesture means halfway through.
    pub stretch: bool,
}

/// An open marker rename. The field text lives in the `WidgetStore` (like every
/// other `TextInput`); this only tracks WHICH marker is being renamed and whether
/// `paint` has already seeded the field + claimed focus (done once, on the first
/// frame the rename is open — re-seeding every frame would stomp the user's typing
/// and reset the caret).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkerRename {
    /// Storage index of the marker being renamed.
    pub index: usize,
    /// The field has been registered + seeded with the current label + focused.
    pub opened: bool,
}

/// An open clip rename — see [`TimelinePanelState::clip_rename`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipRename {
    /// Index of the clip being renamed.
    pub index: usize,
    /// The field has been registered + seeded with the current name + focused.
    pub opened: bool,
}

/// A press that landed on a Summary column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SummaryPress {
    /// The column's time, as `f64::to_bits` (the handle dispatch carries).
    pub t_bits: u64,
    /// The whole column was already selected when the press landed.
    pub was_selected: bool,
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
    /// Total time delta already emitted as `MoveSelectedKeys`. Each frame emits
    /// only the difference from here, so the streamed moves sum to exactly the
    /// snapped delta of the whole drag.
    pub applied_s: f64,
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
            label_w: crate::tracks::LABEL_COL_W,
            label_drag: None,
            graph_h: crate::graph::GRAPH_H_DEFAULT,
            graph_resize: None,
            expanded: Vec::new(),
            speed_view: false,
            tab: crate::tab::Tab::default(),
            column_lock: false,
            handle_drag: None,
            anchor_drag: None,
            summary_press: None,
            loop_drag: None,
            strip_drag: None,
            weight_edit: None,
            marker_drag: None,
            marker_rename: None,
            clip_rename: None,
            box_drag: None,
            box_commit: None,
            rect: None,
            resize: None,
        }
    }
}

impl TimelinePanelState {
    /// Whether this track's graph editor is open.
    #[must_use]
    pub fn is_expanded(&self, target: u64) -> bool {
        self.expanded.contains(&target)
    }

    /// Open/close a track's graph editor. Collapsing also drops an in-flight
    /// handle or anchor drag on that track — its band is about to stop existing,
    /// so the `paint`-side resolver that would close the undo bracket will never
    /// run again.
    pub fn toggle_expanded(&mut self, target: u64) {
        if let Some(i) = self.expanded.iter().position(|&t| t == target) {
            self.expanded.remove(i);
            if self.handle_drag.is_some_and(|d| d.target == target) {
                self.handle_drag = None;
                push_intent(TimelineIntent::EndEdit);
            }
            if self
                .anchor_drag
                .as_ref()
                .is_some_and(|d| d.target == target)
            {
                self.anchor_drag = None;
                push_intent(TimelineIntent::EndEdit);
            }
        } else {
            self.expanded.push(target);
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

/// Publish whether the panel is on the Keys tab (called by `paint` each frame).
pub(crate) fn publish_keys_mode(on: bool) {
    KEYS_MODE.with(|c| c.set(on));
}

/// **Is the timeline panel on the Keys tab?** Read by the shell to drive the CLIP
/// playhead and solo the active clip (the AE precomp model). Reflects the panel's
/// last paint, so it lags a tab switch by at most one frame — imperceptible, and
/// the same latency every panel→shell mirror here already has.
#[must_use]
pub fn keys_mode() -> bool {
    KEYS_MODE.with(Cell::get)
}

/// Raise a dope-sheet edit intent (called by `interact` while draining gestures).
pub(crate) fn push_intent(intent: TimelineIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drop every in-flight row gesture and close the undo bracket it left open.
///
/// Called wherever **the rows a gesture lives on stop existing**: the panel being
/// hidden mid-drag, or the tab switching out from under it. One door for both —
/// a gesture stranded by either is stranded the same way, and the failure mode is
/// identical and silent: an open bracket swallows the NEXT atomic edit, so the
/// animator's following action lands inside a Ctrl+Z step that is not its own.
pub(crate) fn drop_row_gestures(state: &mut TimelinePanelState) {
    state.box_drag = None;
    state.box_commit = None;
    // One pointer means at most one of these is armed, but take all three
    // before testing: `||` would short-circuit and strand the others.
    let key = state.key_drag.take().is_some();
    let handle = state.handle_drag.take().is_some();
    let anchor = state.anchor_drag.take().is_some();
    state.summary_press = None;
    if key || handle || anchor {
        push_intent(TimelineIntent::EndEdit);
    }
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

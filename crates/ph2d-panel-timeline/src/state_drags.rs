//! The in-progress DRAG-state structs (anchor / loop / strip / scale / stagger /
//! key drags + the rename / press helpers) — plain data, split from `state.rs`
//! under the panel LOC cap. A CHILD module (`use super::*`) sharing the parent's
//! imports; re-exported by `state` so callers keep using `crate::state::KeyDrag`.

use super::*;

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
    /// Which lane the strip sits on **right now** — it FOLLOWS a cross-lane drag, because the
    /// next frame's intent has to name the lane the strip has already moved to.
    pub lane: usize,
    /// The lane it started on, and the pointer `y` it started at — the pair the row-crossing
    /// is measured from. Absolute, never accumulated per frame, for the same reason
    /// `start_span` is (`arch_no_absolute_drag_pattern`).
    pub start_lane: usize,
    /// Pointer y (global px) when the drag began.
    pub start_y: f32,
    /// The strip's stable identity — NOT its index: the lane re-sorts as the strip
    /// crosses its neighbour, and an index-anchored drag would swap victims mid-air.
    pub id: ph2d_timeline::StripId,
    /// Which gizmo the drag grabbed: `0` = trim-start (bottom-left, red), `1` =
    /// trim-end (bottom-right, red), `2` = body (slide), `3` = fade-in, `4` = fade-out,
    /// `5` = stretch-start (top-left, green), `6` = stretch-end (top-right, green).
    /// Each strip operation is its own corner now (Enio, 2026-07-16) — trim and stretch
    /// no longer share an edge behind a Cmd modifier.
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
    /// The field is editing the marker's SIGNAL (ADR-0143), not its label — set by
    /// the marker menu's *Set Signal* row (`marker_menu`), the label mode by *Rename
    /// Marker*. One field, two modes: same widget id, same seeding rule and
    /// Enter/Esc/click-away contract, differing only in which value it seeds from and
    /// which intent it commits (`SetMarkerSignal` vs `RenameMarker`).
    pub editing_signal: bool,
}

/// **Which list the open rename is renaming.**
///
/// One field, two lists: a clip named from the transport chip, a container named from its
/// row's pencil in the Containers list. A SECOND rename field would be a second answer to
/// "how do you type a name in this panel" — same widget id, same seeding rule, same
/// Enter/Esc/click-away contract — differing only in where it floats and which intent it
/// commits, which is exactly what this enum carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameKind {
    /// A clip in `TimelineViewSnapshot::clips`, committed as `RenameClip`.
    Clip,
    /// A container in `TimelineViewSnapshot::containers`, committed as `RenameContainer`.
    Container,
    /// A LANE row (Arrange/Container), committed as `RenameLane`. `index` is the lane's
    /// index within the open host's stack — the same index the row paints and the mode/mute
    /// edits carry (Enio, 2026-07-23).
    Lane,
}

/// An open name edit — see [`TimelinePanelState::clip_rename`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipRename {
    /// Which list `index` points into.
    pub kind: RenameKind,
    /// Index of the clip (or container) being renamed.
    pub index: usize,
    /// The field has been registered + seeded with the current name + focused.
    pub opened: bool,
}

/// An open **property-expression** edit (ADR-0144) — the inline formula field
/// opened from a track menu's "Expression\u{2026}" row. It floats at the click
/// position (`x`/`y`) rather than over the row, so it needs no row geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExprEdit {
    /// The binding target whose expression is being authored.
    pub target: u64,
    /// Where the field floats — the menu's click position.
    pub x: f32,
    /// See [`ExprEdit::x`].
    pub y: f32,
    /// The field has been registered + seeded with the current formula + focused.
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

/// An in-progress time-scale drag of the key selection: the fixed pivot + the
/// moving edge's original time + the total factor streamed so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleDrag {
    /// The pivot (seconds) — the OPPOSITE edge of the selection, held fixed for the
    /// whole drag so streamed incremental factors compose about ONE point (the
    /// engine's `scale_keys` maps `t -> pivot + (t - pivot)*factor`).
    pub pivot_seconds: f64,
    /// The moving edge's time (seconds) at Begin. The target factor each frame is
    /// `(t_at_cursor - pivot) / (edge_seconds - pivot)`, from the FIXED drag
    /// geometry — never the live extent, which the scale itself is moving.
    pub edge_seconds: f64,
    /// Which edge is grabbed: `true` = right/end (pivot is the left edge).
    pub right: bool,
    /// Total scale factor already streamed as `ScaleSelectedKeys`. Each frame emits
    /// only `want / applied`, so the composed scale equals the drag's target factor.
    pub applied: f64,
}

/// An in-progress Quick-Offset stagger drag (§3): the pointer x at Begin + the
/// total per-rank step already streamed. The per-rank step is the frame-snapped
/// drag delta; each track then shifts by `rank · step`, so the drag distance IS
/// the cascade amount and successive increments compose (the rank is constant).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaggerDrag {
    /// Pointer x (global px) when the drag began.
    pub start_x: f32,
    /// Latest pointer x (global px).
    pub cur_x: f32,
    /// The pressed key was already selected and pressed without Shift: keep the
    /// whole selection so the cascade spans it, collapsing to this key only on a
    /// plain click. `None` when the press already set the selection.
    pub collapse_to: Option<ph2d_timeline::SelectedKey>,
    /// Total per-rank step already emitted as `StaggerSelectedKeys`. Each frame
    /// emits only the difference from here.
    pub applied_step_s: f64,
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

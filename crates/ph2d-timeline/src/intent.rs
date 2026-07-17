//! [`TimelineIntent`] — the panel→runtime command channel: **what the panel is
//! allowed to ask for**, and nothing about what happens next.
//!
//! Its interpreter lives in the sibling [`crate::intent_apply`], for the same
//! reason `stack.rs` (the data) and `stack_edit.rs` (the authoring) are two
//! files: the vocabulary is read by everyone who drives the panel, and the
//! router is read by whoever changes what an edit does. Keeping them in one file
//! meant every reader of either paid for both — and the file grew past the LOC
//! cap, which is the gate noticing exactly this.
//!
//! The panel never mutates the document directly: it emits intents, the bridge
//! drains them through [`crate::apply_intent`], and re-reads a snapshot. This
//! keeps the panel free of document/undo semantics (mirrors the vector/motion
//! bridges), and — because `apply_intent` is pure over `(state, playhead)` — every
//! gesture is headless-testable without any UI (DIRETIVA: the seam test is the
//! deliverable).
//!
//! Each **document-mutating** intent is one undo step (history `begin` →
//! `commit_if_changed`), so a no-op edit never pollutes the stack. Selection,
//! transport and flag intents are not undoable.

use ph2d_anim::{AnimTarget, AnimValue, Interp, KeyId, RationalTime};

use crate::prop::PropKind;
use crate::stack::{LaneMode, StripId, StripLoop};
use crate::state::SelectedKey;

/// A single command from the timeline panel (or a headless test).
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineIntent {
    // ── transport (drives the Playhead) ─────────────────────────────────────
    /// Start playback.
    Play,
    /// Pause playback (position holds).
    Pause,
    /// Toggle play/pause.
    TogglePlay,
    /// Scrub to an absolute time (seconds); frame-snapped if the flag is set.
    Scrub(f64),
    /// Seek to a whole frame index at the document fps.
    SeekFrame(i64),
    /// Set the playback-rate multiplier.
    SetRate(f64),
    /// Set (or clear) the ACTIVE CLIP's loop: its `[start, end)` range in seconds
    /// and whether it **ping-pongs** (plays back and forth) instead of wrapping.
    ///
    /// One intent carries both because they are ONE thing — a loop is a span PLUS
    /// what happens at its end. That also makes the two toggles that drive it
    /// (Loop / PingPong) mutually exclusive **by construction**: there is no value
    /// that is both, so no rule anyone can forget to enforce.
    SetLoop {
        /// `[start, end)` in seconds; `None` clears the loop.
        range: Option<(f64, f64)>,
        /// Play back and forth instead of jumping to the start.
        ping_pong: bool,
    },

    // ── authoring (active clip; each is one undo step) ──────────────────────
    /// Ensure a binding exists for `(entity, prop)` (creates the row).
    Bind {
        /// Live entity bits.
        entity: u64,
        /// Property to bind.
        prop: PropKind,
    },
    /// Remove a binding + its track.
    Unbind {
        /// Live entity bits.
        entity: u64,
        /// Property to unbind.
        prop: PropKind,
    },
    /// Insert a key on `(entity, prop)`; binds + creates the track if needed.
    /// The new key becomes the sole selection.
    AddKey {
        /// Live entity bits.
        entity: u64,
        /// Property.
        prop: PropKind,
        /// Key time.
        t: RationalTime,
        /// Key value.
        value: AnimValue,
        /// Interpolation from this key.
        interp: Interp,
    },
    /// Shift every selected key by `delta_seconds`.
    MoveSelectedKeys {
        /// Signed time delta.
        delta_seconds: f64,
    },
    /// Scale every selected key's time about `pivot_seconds` by `factor`.
    ScaleSelectedKeys {
        /// Fixed point of the scale.
        pivot_seconds: f64,
        /// Scale factor.
        factor: f64,
    },
    /// Duplicate every selected key, preserving the group's internal timing.
    ///
    /// Where the copies land is read off the playhead (see
    /// [`duplicate_delta`]): normally the FIRST copy lands on the playhead;
    /// with the playhead already sitting on the first selected key — where a
    /// copy would be invisible underneath it — the group is offset two display
    /// frames instead. Copies overwrite any key they land on. One undo step; the
    /// copies become the selection.
    DuplicateSelection,
    /// Delete every selected key.
    DeleteSelection,
    /// Copy the selected keys onto the clipboard (time-rebased to the earliest).
    /// Not undoable — the clipboard is panel state. A no-op with no selection,
    /// so an accidental copy never clobbers a good clipboard.
    CopySelection,
    /// Copy the selected keys, then delete them (the delete is one undo step).
    CutSelection,
    /// Paste the clipboard at the playhead, preserving the copied group's
    /// internal timing. The pasted keys become the selection; one undo step.
    Paste,
    /// Set one key's value.
    SetKeyValue {
        /// Track target.
        target: AnimTarget,
        /// Key id.
        key: KeyId,
        /// New value.
        value: AnimValue,
    },
    /// Set one key's outgoing interpolation.
    SetInterp {
        /// Track target.
        target: AnimTarget,
        /// Key id.
        key: KeyId,
        /// New interpolation.
        interp: Interp,
    },
    /// Give **every selected key** the same outgoing interpolation, across any
    /// number of tracks and times. One undo step. A no-op with nothing selected.
    SetSelectedInterp {
        /// The interpolation each selected key receives.
        interp: Interp,
    },
    /// Freeze every selected key's interpolation into the bézier its own handles
    /// already draw ([`Interp::to_bezier`]). Unlike
    /// [`TimelineIntent::SetSelectedInterp`] there is no single `Interp` to send:
    /// each key converts from the curve IT had, so a mixed selection stays mixed
    /// and nothing moves on screen — it only becomes draggable.
    ConvertSelectionToBezier,
    /// Mark / unmark one key as **roving** (AE "rove across time"): its time
    /// stops being authored and is derived so the value travels at constant
    /// speed between the pinned neighbours. Boundary keys never rove.
    SetRove {
        /// Track target.
        target: AnimTarget,
        /// Key id.
        key: KeyId,
        /// `true` to rove, `false` to pin at the currently derived time.
        on: bool,
    },
    /// Mark / unmark **every selected key** as roving, across any number of
    /// tracks. One undo step. A no-op with nothing selected.
    SetSelectedRove {
        /// `true` to rove, `false` to pin.
        on: bool,
    },

    // ── markers (W4.T3; each is one undo step) ──────────────────────────────
    /// Add a marker at `t_seconds` with `label` (user content; the shell
    /// auto-names it). Frame-snapped like a key.
    AddMarker {
        /// Marker time in seconds.
        t_seconds: f64,
        /// Author-visible label.
        label: String,
    },
    /// Move the marker at storage `index` to `t_seconds` (frame-snapped).
    MoveMarker {
        /// Storage index (stable across a drag).
        index: usize,
        /// New time in seconds.
        t_seconds: f64,
    },
    /// Remove the marker at storage `index`.
    RemoveMarker {
        /// Storage index.
        index: usize,
    },
    /// Relabel the marker at storage `index`.
    RenameMarker {
        /// Storage index.
        index: usize,
        /// New label.
        label: String,
    },

    // ── selection (not undoable) ────────────────────────────────────────────
    /// Replace the selection with a single key.
    SelectSingle(SelectedKey),
    /// Toggle a key's membership (shift-click).
    ToggleSelect(SelectedKey),
    /// Add a key to the selection (box-select).
    AddToSelection(SelectedKey),
    /// Clear the selection.
    ClearSelection,

    // ── flags (not undoable) ────────────────────────────────────────────────
    /// Arm/disarm auto-key.
    SetAutoKey(bool),
    /// Enable/disable frame snapping of edited/scrubbed times.
    SetFrameSnap(bool),
    /// Arm/disarm performing (record-during-play, W5).
    SetPerforming(bool),

    // ── history ─────────────────────────────────────────────────────────────
    /// Open an undo bracket around a multi-frame gesture (a graph-handle drag).
    /// Until the matching [`TimelineIntent::EndEdit`], every document edit joins
    /// this one step instead of pushing its own — a drag that emits one
    /// `SetInterp` per frame must undo in a single Ctrl+Z.
    BeginEdit,
    /// Close the bracket opened by [`TimelineIntent::BeginEdit`], pushing one
    /// undo step if the document actually changed. Unmatched = no-op.
    EndEdit,
    /// Undo one document step.
    Undo,
    /// Redo one document step.
    Redo,

    // ── clips (W5 / NLA step 1 — each is one undo step) ─────────────────────
    /// Switch which clip is edited. Out of range: no-op.
    ///
    /// Undoable like any other document edit: the active clip is what every
    /// authoring intent below writes into, so a Ctrl+Z that did not put it back
    /// would undo the keys into a clip the animator is no longer looking at.
    SetActiveClip {
        /// Index into [`crate::TimelineDoc::clips`].
        index: usize,
    },
    /// Append a new, empty clip and make it active. Refused past
    /// [`crate::MAX_CLIPS`].
    AddClip,
    /// Rename clip `index`.
    RenameClip {
        /// Index into [`crate::TimelineDoc::clips`].
        index: usize,
        /// The new author-visible name.
        name: String,
    },
    /// Delete clip `index`. The LAST clip is never deleted (a document always has
    /// one to edit).
    DeleteClip {
        /// Index into [`crate::TimelineDoc::clips`].
        index: usize,
    },
    /// **Copy clip `index`** — curves, loop and all — as a new clip, and make it
    /// active. Refused past [`crate::MAX_CLIPS`].
    ///
    /// Distinct from [`Self::AddClip`], which can only ever make an EMPTY clip:
    /// bindings are document-wide, so a variation of an existing animation means
    /// copying its curves, and hand-copying every key is not a workflow.
    DuplicateClip {
        /// Index into [`crate::TimelineDoc::clips`].
        index: usize,
    },
    /// **Play clip `index` backwards**: every track mirrored inside the clip's own
    /// extent, segment shapes and all ([`crate::TimelineDoc::reverse_clip`]).
    ReverseClip {
        /// Index into [`crate::TimelineDoc::clips`].
        index: usize,
    },

    // ── the clip stack (ADR-0115 — each is one undo step) ───────────────────
    /// Append an empty lane. Refused past [`crate::MAX_LANES`].
    AddLane,
    /// Delete a lane and every strip on it.
    RemoveLane {
        /// Index into [`crate::TimelineDoc::stack`].
        lane: usize,
    },
    /// Mute a lane — which REMOVES it from the blend, and is not the same as
    /// turning its weight to zero (a zero-weight lane still asserts its coverage
    /// and mixes toward its value; a muted one is simply not there).
    SetLaneMuted {
        /// Index into the stack.
        lane: usize,
        /// The new state.
        muted: bool,
    },
    /// How a lane enters the stack below it.
    SetLaneMode {
        /// Index into the stack.
        lane: usize,
        /// Override (mix toward) or Additive (apply the delta).
        mode: LaneMode,
    },
    /// A lane's influence, `[0, 1]`.
    SetLaneWeight {
        /// Index into the stack.
        lane: usize,
        /// Clamped by the evaluator.
        weight: f64,
    },
    /// Place a clip on a lane over `[t_start, t_end)`.
    AddStrip {
        /// Index into the stack.
        lane: usize,
        /// Index into [`crate::TimelineDoc::clips`].
        clip: usize,
        /// Where it starts, in seconds.
        t_start: f64,
        /// Where it ends, exclusive.
        t_end: f64,
    },
    /// Remove a strip.
    RemoveStrip {
        /// Index into the stack.
        lane: usize,
        /// The strip's stable identity — never its index (dragging one past its
        /// neighbour renumbers both).
        id: StripId,
    },
    /// Copy a strip and lay the copy down right after it (see
    /// [`crate::TimelineDoc::duplicate_strip`]).
    DuplicateStrip {
        /// Index into the stack.
        lane: usize,
        /// Stable identity of the original.
        id: StripId,
    },
    /// **Slide a strip**, rigidly: its span moves, its content comes along.
    MoveStrip {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// The new start; the end follows by the span.
        t_start: f64,
    },
    /// **Trim a strip** by one edge — which is NOT a move and NOT a stretch.
    ///
    /// Dragging an edge reveals or hides content: the span's edge and the source
    /// slice's edge travel together, so the frames that remain visible stay put on
    /// the timeline. Stretching (the same content over a longer span) is `speed`,
    /// and it is a different gesture on purpose — conflating the two is how a trim
    /// ends up silently retiming an animation.
    TrimStrip {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// `0` = the start edge, `1` = the end edge.
        edge: u8,
        /// Where that edge is being dragged to, in seconds.
        t: f64,
    },
    /// **Stretch a strip** by one edge — the retime that `TrimStrip` refuses to be.
    ///
    /// The source slice is held FIXED and the span is resized around it, so the
    /// rate falls out: `speed = slice / span`. Nothing is revealed or hidden; the
    /// same frames play, slower or faster. The edge you are NOT dragging stays put.
    ///
    /// This is the *only* authoring path to `speed` in the UI, and it exists
    /// because a rate is a thing you feel, not a number you type. (The NLEs make
    /// it a separate tool — Premiere's Rate Stretch, Resolve's Change Speed. We
    /// have no tool palette inside a panel, so it is the modifier on the gesture
    /// it modifies.)
    StretchStrip {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// `0` = the start edge, `1` = the end edge.
        edge: u8,
        /// Where that edge is being dragged to, in seconds.
        t: f64,
    },
    /// What a strip's source does once it runs past its slice.
    SetStripLoop {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// Once / Loop / PingPong.
        loop_mode: StripLoop,
    },
    /// A strip's playback rate, stated as a number rather than felt as a drag —
    /// the same edit [`Self::StretchStrip`] makes, from the other end.
    ///
    /// The slice is held and the span is re-derived (`span = slice / speed`), with
    /// `t_start` pinned. Setting the rate and stretching therefore mean the SAME
    /// thing about the strip; a setter that moved the rate without the span would
    /// leave the strip playing a slice that no longer fits it — the source would
    /// run out mid-span (or overrun), and the number in the menu would describe a
    /// strip nobody authored. `1.0` restores real time, which is the menu's
    /// "Reset Speed".
    SetStripSpeed {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// Clamped away from zero — a strip at speed 0 reads one frame forever,
        /// which is what `StripLoop::Once` past its end already does, honestly.
        speed: f64,
    },
    /// **The strip's own fade at one edge** (ADR-0115 B4) — what the corner handle
    /// authors.
    ///
    /// It is the SAME curve as the crossfade, which is the whole point: where a
    /// neighbour overlaps this edge, the overlap defines the window and this number is
    /// ignored (`ClipLane::blend_in`/`blend_out`), so the two can never disagree. The
    /// panel refuses the drag there rather than writing a number nobody will read
    /// (Unity greys the field out and relabels it "Blend").
    ///
    /// Without this, a strip ALONE on a lane could not fade at all: it entered and left
    /// hard. The fields existed and the evaluator already honoured them — nothing wrote
    /// them.
    SetStripEase {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// `0` = the fade-in (at `t_start`), `1` = the fade-out (at `t_end`) — the same
        /// edge vocabulary as [`Self::TrimStrip`].
        edge: u8,
        /// The fade's length in seconds. Clamped to `[0, span]` on apply: a fade longer
        /// than the strip is not a fade, and a negative one is not a thing.
        seconds: f64,
    },
}

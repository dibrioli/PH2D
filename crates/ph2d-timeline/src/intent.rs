//! [`TimelineIntent`] — the panel→runtime command channel, and
//! [`apply_intent`], which interprets one intent against the
//! [`TimelineState`] + engine [`Playhead`].
//!
//! The panel never mutates the document directly: it emits intents, the bridge
//! drains them through `apply_intent`, and re-reads a snapshot. This keeps the
//! panel free of document/undo semantics (mirrors the vector/motion bridges),
//! and — because `apply_intent` is pure over `(state, playhead)` — every gesture
//! is headless-testable without any UI (DIRETIVA: the seam test is the
//! deliverable).
//!
//! Each **document-mutating** intent is one undo step (history `begin` →
//! `commit_if_changed`), so a no-op edit never pollutes the stack. Selection,
//! transport and flag intents are not undoable.

use ph2d_anim::{AnimTarget, AnimValue, Interp, KeyId, RationalTime};
use ph2d_core::Playhead;

use crate::doc::TimelineDoc;
use crate::prop::PropKind;
use crate::stack::{LaneMode, StripId, StripLoop};
use crate::state::{SelectedKey, Selection, TimelineState};

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
    /// What a strip's source does once it runs past its slice.
    SetStripLoop {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// Once / Loop / PingPong.
        loop_mode: StripLoop,
    },
    /// A strip's playback rate (1.0 = real time).
    SetStripSpeed {
        /// Index into the stack.
        lane: usize,
        /// Stable identity.
        id: StripId,
        /// Clamped away from zero — a strip at speed 0 reads one frame forever,
        /// which is what `StripLoop::Once` past its end already does, honestly.
        speed: f64,
    },
}

/// Apply one intent to the timeline state + playhead. Document-mutating intents
/// are grouped as a single undo step.
pub fn apply_intent(state: &mut TimelineState, playhead: &mut Playhead, intent: TimelineIntent) {
    use TimelineIntent as I;
    let fps = state.doc.fps_display;
    match intent {
        // transport
        I::Play => playhead.play(),
        I::Pause => playhead.pause(),
        I::TogglePlay => {
            playhead.toggle_play();
        }
        I::Scrub(t) => playhead.seek(snap(t, fps, state.flags.frame_snap)),
        I::SeekFrame(f) => playhead.seek_frame(f, fps),
        I::SetRate(r) => playhead.set_rate(r),
        // The loop belongs to the CLIP, not the document: "walk" cycles over its
        // own two seconds and "run" over its own. The `Playhead` owns the LIVE loop
        // (it is what wraps the transport); the clip is where it is parked, so a
        // switch swaps it in. Written straight (no undo step) — a loop brace is
        // transport, not authoring.
        I::SetLoop { range, ping_pong } => {
            state.doc.set_active_loop(range);
            state.doc.set_active_ping_pong(ping_pong);
            sync_loop(&state.doc, playhead);
        }

        // authoring (undoable)
        I::Bind { entity, prop } => edit(state, |doc, _| {
            doc.bind(entity, prop);
        }),
        I::Unbind { entity, prop } => edit(state, |doc, _| {
            doc.unbind(entity, prop);
        }),
        I::AddKey {
            entity,
            prop,
            t,
            value,
            interp,
        } => {
            let snapped = snap_time(t, fps, state.flags.frame_snap);
            edit(state, |doc, sel| {
                // Upsert: capture-the-pose (K / auto-key) updates the key at this
                // time instead of stacking duplicates when fired repeatedly.
                let (target, key) = doc.upsert_key(entity, prop, snapped, value, interp);
                sel.set_single(SelectedKey { target, key });
            });
        }
        I::MoveSelectedKeys { delta_seconds } => {
            // Rationalize the offset ONCE, against the display rate: a whole
            // number of frames must stay a whole number of frames after the move.
            let delta = delta_time(delta_seconds, fps, state.flags.frame_snap);
            edit(state, |doc, sel| {
                for_selected_tracks(doc, sel, |track, ids| track.move_keys(ids, delta));
            });
        }
        I::ScaleSelectedKeys {
            pivot_seconds,
            factor,
        } => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| {
                track.scale_keys(ids, pivot_seconds, factor)
            });
        }),
        I::DuplicateSelection => {
            let Some(delta) = duplicate_delta(state, playhead.time()) else {
                return; // nothing selected
            };
            edit(state, |doc, sel| {
                // The copies become the selection (Blender): the next drag moves
                // the duplicates, not the originals they were made from.
                let mut copies: Vec<SelectedKey> = Vec::new();
                for target in distinct_targets(sel) {
                    let ids = sel.ids_for(target);
                    if let Some(track) = doc.active_clip_mut().track_mut(target) {
                        copies.extend(
                            track
                                .duplicate_keys(&ids, delta)
                                .into_iter()
                                .map(|key| SelectedKey { target, key }),
                        );
                    }
                }
                if !copies.is_empty() {
                    sel.clear();
                    for k in copies {
                        sel.add(k);
                    }
                }
            });
        }
        I::DeleteSelection => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| track.remove_keys(ids));
            sel.clear();
        }),
        I::CopySelection => copy_selection(state),
        I::CutSelection => {
            copy_selection(state);
            edit(state, |doc, sel| {
                for_selected_tracks(doc, sel, |track, ids| track.remove_keys(ids));
                sel.clear();
            });
        }
        I::Paste => {
            // Snapshot the clipboard out so the `edit` closure can borrow `state`.
            let items: Vec<_> = state.clipboard.keys().to_vec();
            let t0 = playhead.time();
            let snap_on = state.flags.frame_snap;
            edit(state, |doc, sel| {
                sel.clear();
                for ck in &items {
                    let t = snap_time(
                        RationalTime::from_seconds(t0 + ck.offset_seconds),
                        fps,
                        snap_on,
                    );
                    // A track the copy came from may have been unbound since —
                    // skip it rather than resurrecting a dead binding.
                    if let Some(track) = doc.active_clip_mut().track_mut(ck.target) {
                        let key = track.upsert_key(t, ck.value, ck.interp);
                        sel.add(SelectedKey {
                            target: ck.target,
                            key,
                        });
                    }
                }
            });
        }
        I::SetKeyValue { target, key, value } => edit(state, |doc, _| {
            if let Some(track) = doc.active_clip_mut().track_mut(target) {
                track.set_value(key, value);
            }
        }),
        I::SetInterp {
            target,
            key,
            interp,
        } => edit(state, |doc, _| {
            if let Some(track) = doc.active_clip_mut().track_mut(target) {
                track.set_interp(key, interp);
            }
        }),
        I::AddMarker { t_seconds, label } => {
            let snapped = snap_time(
                RationalTime::from_seconds(t_seconds),
                fps,
                state.flags.frame_snap,
            );
            edit(state, |doc, _| {
                doc.add_marker(snapped, label);
            });
        }
        I::MoveMarker { index, t_seconds } => {
            let snapped = snap_time(
                RationalTime::from_seconds(t_seconds),
                fps,
                state.flags.frame_snap,
            );
            edit(state, |doc, _| {
                doc.move_marker(index, snapped);
            });
        }
        I::RemoveMarker { index } => edit(state, |doc, _| {
            doc.remove_marker(index);
        }),
        I::RenameMarker { index, label } => edit(state, |doc, _| {
            doc.set_marker_label(index, label);
        }),
        I::SetSelectedInterp { interp } => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| {
                for &id in ids {
                    track.set_interp(id, interp);
                }
            });
        }),
        I::ConvertSelectionToBezier => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| {
                for &id in ids {
                    // Read each key's OWN curve: one shared `Interp` would flatten
                    // a mixed selection onto whichever one we happened to pick.
                    if let Some(k) = track.key(id) {
                        track.set_interp(id, k.interp.to_bezier());
                    }
                }
            });
        }),
        I::SetRove { target, key, on } => edit(state, |doc, _| {
            if let Some(track) = doc.active_clip_mut().track_mut(target) {
                track.set_roving(key, on);
            }
        }),
        I::SetSelectedRove { on } => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| {
                for &id in ids {
                    track.set_roving(id, on);
                }
            });
        }),

        // selection
        I::SelectSingle(k) => state.selection.set_single(k),
        I::ToggleSelect(k) => state.selection.toggle(k),
        I::AddToSelection(k) => state.selection.add(k),
        I::ClearSelection => state.selection.clear(),

        // flags
        I::SetAutoKey(on) => state.flags.auto_key = on,
        I::SetFrameSnap(on) => state.flags.frame_snap = on,
        I::SetPerforming(on) => state.flags.performing = on,

        // history
        I::BeginEdit => state.history.begin(&state.doc),
        I::EndEdit => state.history.commit_if_changed(&state.doc),
        I::Undo => {
            // A stray bracket (pointer capture lost, panel hidden mid-drag) would
            // otherwise commit a stale pre-state on the next EndEdit.
            state.history.cancel();
            state.undo();
            // The undo may have put another clip back in front — its loop comes with it.
            sync_loop(&state.doc, playhead);
        }
        I::Redo => {
            state.history.cancel();
            state.redo();
            sync_loop(&state.doc, playhead);
        }

        // ── clips ───────────────────────────────────────────────────────────
        // Every arm clears the SELECTION: a `KeyId` is only meaningful inside the
        // track that issued it, so ids held across a clip switch would point at
        // whatever key happens to sit at that index in the new clip — a stale
        // selection that deletes the wrong keys. Clearing is the only safe move,
        // and it is what switching a comp does everywhere else too.
        I::SetActiveClip { index } => {
            edit(state, |doc, sel| {
                doc.set_active(index);
                sel.clear();
            });
            sync_loop(&state.doc, playhead);
        }
        I::AddClip => {
            edit(state, |doc, sel| {
                let name = doc.fresh_clip_name();
                let i = doc.add_clip(name);
                doc.set_active(i);
                sel.clear();
            });
            sync_loop(&state.doc, playhead);
        }
        I::RenameClip { index, name } => edit(state, |doc, _| {
            doc.rename_clip(index, name);
        }),
        I::DeleteClip { index } => {
            edit(state, |doc, sel| {
                if doc.remove_clip(index) {
                    sel.clear();
                }
            });
            sync_loop(&state.doc, playhead);
        }

        // ── the clip stack ──────────────────────────────────────────────────
        // None of these touch the SELECTION: a strip is not a key, and the two
        // never share an identity space.
        I::AddLane => edit(state, |doc, _| {
            let name = doc.fresh_lane_name();
            doc.add_lane(name);
        }),
        I::RemoveLane { lane } => edit(state, |doc, _| {
            doc.remove_lane(lane);
        }),
        I::SetLaneMuted { lane, muted } => edit(state, |doc, _| {
            if let Some(l) = doc.stack_mut().get_mut(lane) {
                l.muted = muted;
            }
        }),
        I::SetLaneMode { lane, mode } => edit(state, |doc, _| {
            if let Some(l) = doc.stack_mut().get_mut(lane) {
                l.mode = mode;
            }
        }),
        I::SetLaneWeight { lane, weight } => edit(state, |doc, _| {
            if let Some(l) = doc.stack_mut().get_mut(lane) {
                l.weight = weight.clamp(0.0, 1.0); // CLAMP-OK: constant bounds
            }
        }),
        I::AddStrip {
            lane,
            clip,
            t_start,
            t_end,
        } => edit(state, |doc, _| {
            doc.add_strip(lane, clip, t_start.max(0.0), t_end.max(0.0));
        }),
        I::RemoveStrip { lane, id } => edit(state, |doc, _| {
            doc.remove_strip(lane, id);
        }),
        I::MoveStrip { lane, id, t_start } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                let span = s.span();
                s.t_start = t_start.max(0.0);
                s.t_end = s.t_start + span; // rigid: the span rides along
            }
        }),
        I::TrimStrip { lane, id, edge, t } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                trim_strip(s, edge, t);
            }
        }),
        I::SetStripLoop {
            lane,
            id,
            loop_mode,
        } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                s.loop_mode = loop_mode;
            }
        }),
        I::SetStripSpeed { lane, id, speed } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                s.speed = speed.clamp(MIN_STRIP_SPEED, MAX_STRIP_SPEED); // CLAMP-OK: const bounds
            }
        }),
    }
}

/// The slowest and fastest a strip may play. Zero would freeze the source on one
/// frame forever — which `StripLoop::Once` past its end already expresses, and
/// honestly; a negative speed would run it backwards, which is `PingPong`'s job.
const MIN_STRIP_SPEED: f64 = 0.01;
/// Mirror of [`MIN_STRIP_SPEED`].
const MAX_STRIP_SPEED: f64 = 100.0;

/// Move one edge of a strip, taking the source slice with it.
///
/// **A trim is not a stretch.** The frames that stay visible must stay WHERE they
/// were on the timeline, so the span's edge and the slice's edge travel together
/// (by `speed`, which is what converts timeline seconds into clip seconds).
/// Dragging the start edge one second to the right hides the clip's first second
/// — it does not squeeze the whole clip into a shorter box.
///
/// Neither edge may cross the other: a strip of negative span is a strip that
/// covers no time and paints inside-out.
fn trim_strip(s: &mut crate::ClipStrip, edge: u8, t: f64) {
    let min_span = 1.0 / 240.0; // LITERAL-OK: a quarter of a frame at 60 fps
    if edge == 0 {
        let t_start = t.max(0.0).min(s.t_end - min_span);
        s.src_in += (t_start - s.t_start) * s.speed;
        s.t_start = t_start;
    } else {
        let t_end = t.max(s.t_start + min_span);
        s.src_out += (t_end - s.t_end) * s.speed;
        s.t_end = t_end;
    }
}

/// Restore every invariant an edit may have broken. Idempotent, cheap, and the
/// SINGLE place either one is re-derived — which is the whole point: an invariant
/// that any authoring path can break needs exactly one place that fixes it, or it
/// is really N places and one of them will be forgotten.
///
/// 1. **Roving keys**: their time is a function of their neighbours' values, so it
///    is re-derived after every add/move/scale/paste/value/interp/rove.
/// 2. **Strip order**: a lane's strips are sorted by start time, and that is what
///    `ClipLane::weight_at` reads to find the neighbour a crossfade blends with.
///    Drag a strip past its neighbour and the order is stale — the drawn crossfade
///    and the evaluated one would disagree, which is exactly the bug class that
///    "one place" exists to prevent.
fn settle(doc: &mut TimelineDoc) {
    doc.active_clip_mut().resolve_roving();
    for lane in doc.stack_mut() {
        lane.resort();
    }
}

/// Copy the selected keys' real `(track, time, value, interp)` onto the
/// clipboard, time-rebased to the earliest. A selection that resolves to no live
/// key leaves the clipboard untouched (never clobber a good clipboard).
fn copy_selection(state: &mut TimelineState) {
    let picked: Vec<(AnimTarget, f64, AnimValue, Interp)> = {
        let clip = state.doc.active_clip();
        state
            .selection
            .keys()
            .iter()
            .filter_map(|sk| {
                let k = clip.track(sk.target)?.key(sk.key)?;
                Some((sk.target, k.t.to_seconds(), k.value, k.interp))
            })
            .collect()
    };
    if picked.is_empty() {
        return;
    }
    state.clipboard.set_from_absolute(&picked);
}

/// Run a doc edit as one undo step: snapshot, mutate `(doc, selection)`, commit
/// only if the document changed.
/// Mirror the ACTIVE clip's loop into the playhead — the one place a clip switch
/// (or an undo of one) becomes the live transport loop.
fn sync_loop(doc: &TimelineDoc, playhead: &mut Playhead) {
    match doc.active_loop() {
        Some((a, b)) => playhead.set_loop(a, b),
        None => playhead.clear_loop(),
    }
    playhead.set_loop_mode(if doc.active_ping_pong() {
        ph2d_core::LoopMode::PingPong
    } else {
        ph2d_core::LoopMode::Wrap
    });
}

fn edit(state: &mut TimelineState, f: impl FnOnce(&mut TimelineDoc, &mut Selection)) {
    // Inside a `BeginEdit`/`EndEdit` bracket the caller owns the undo step: just
    // mutate, and let the bracket's commit fold every frame of the gesture into
    // one. Outside it, the edit is atomic and brackets itself.
    //
    // Every branch re-derives the roving keys' times after the mutation — the
    // single choke point that keeps "time ∝ value travel" true through every
    // authoring path (add/move/scale/paste/value/interp/rove), idempotent and
    // cheap, and folded into the same undo step as the edit it follows.
    if state.history.is_open() {
        f(&mut state.doc, &mut state.selection);
        settle(&mut state.doc);
        return;
    }
    state.history.begin(&state.doc);
    f(&mut state.doc, &mut state.selection);
    settle(&mut state.doc);
    state.history.commit_if_changed(&state.doc);
}

/// Frames a duplicate is pushed by when it would otherwise land on top of its
/// own source (playhead on the first selected key). Two rather than one so the
/// copy is unmistakably separate at any zoom.
const DUPLICATE_IDLE_FRAMES: i64 = 2;

/// Turn a UI time offset into an exact [`RationalTime`]. With frame snapping on
/// the offset is a whole number of display frames, expressed over `fps` so it
/// stays frame-exact; otherwise it degrades to microseconds.
fn delta_time(delta_seconds: f64, fps: f64, frame_snap: bool) -> RationalTime {
    if frame_snap && fps.is_finite() && fps > 0.0 {
        RationalTime::from_frame(
            (delta_seconds * fps).round() as i64,
            fps.round().max(1.0) as u32,
        )
    } else {
        RationalTime::from_seconds(delta_seconds)
    }
}

/// Where [`TimelineIntent::DuplicateSelection`] puts the copies, as an offset
/// from the sources. `None` when nothing is selected.
///
/// The first selected key is the group's handle: it lands on the playhead, and
/// the rest of the group follows rigidly. When the playhead is already ON that
/// key the copy would hide underneath it (we have no modal grab to drag it out),
/// so the group is nudged [`DUPLICATE_IDLE_FRAMES`] frames right instead.
fn duplicate_delta(state: &TimelineState, playhead_seconds: f64) -> Option<RationalTime> {
    let fps = state.doc.fps_display;
    let first = earliest_selected_time(state)?;
    let target = snap_time(
        RationalTime::from_seconds(playhead_seconds),
        fps,
        state.flags.frame_snap,
    );
    let delta = target - first;
    Some(if delta == RationalTime::ZERO {
        RationalTime::from_frame(DUPLICATE_IDLE_FRAMES, fps.round().max(1.0) as u32)
    } else {
        delta
    })
}

/// The time of the earliest selected key across every track, or `None` when the
/// selection is empty (or names keys the document no longer has).
fn earliest_selected_time(state: &TimelineState) -> Option<RationalTime> {
    let clip = state.doc.active_clip();
    state
        .selection
        .keys()
        .iter()
        .filter_map(|sk| clip.track(sk.target)?.key(sk.key))
        .map(|k| k.t)
        .min()
}

/// The distinct tracks the selection touches, sorted. Collected up front
/// (immutable borrow) so the caller can then mutate each track in turn.
fn distinct_targets(sel: &Selection) -> Vec<AnimTarget> {
    let mut ts: Vec<AnimTarget> = sel.keys().iter().map(|k| k.target).collect();
    ts.sort_unstable();
    ts.dedup();
    ts
}

/// Apply `op` to every active-clip track that owns at least one selected key,
/// passing that track's selected key ids.
fn for_selected_tracks(
    doc: &mut TimelineDoc,
    sel: &Selection,
    mut op: impl FnMut(&mut ph2d_anim::Track, &[KeyId]),
) {
    for target in distinct_targets(sel) {
        let ids = sel.ids_for(target);
        if let Some(track) = doc.active_clip_mut().track_mut(target) {
            op(track, &ids);
        }
    }
}

/// Snap a second value to a whole display frame when snapping is on.
fn snap(t: f64, fps: f64, on: bool) -> f64 {
    if on && fps.is_finite() && fps > 0.0 {
        (t * fps).round() / fps
    } else {
        t
    }
}

/// Snap a rational time to a whole display frame (returns rational frame time).
/// Public so the shell's auto-key path snaps recorded keys identically to the
/// panel-driven `AddKey` intent.
pub fn snap_time(t: RationalTime, fps: f64, on: bool) -> RationalTime {
    if on && fps.is_finite() && fps > 0.0 {
        let frame = (t.to_seconds() * fps).round() as i64;
        RationalTime::from_frame(frame, fps.round() as u32)
    } else {
        t
    }
}

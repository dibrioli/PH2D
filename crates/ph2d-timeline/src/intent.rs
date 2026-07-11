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
    /// Set (or clear) the `[start, end)` loop range in seconds.
    SetLoop(Option<(f64, f64)>),

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
        I::SetLoop(range) => match range {
            Some((a, b)) => playhead.set_loop(a, b),
            None => playhead.clear_loop(),
        },

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

        // history
        I::BeginEdit => state.history.begin(&state.doc),
        I::EndEdit => state.history.commit_if_changed(&state.doc),
        I::Undo => {
            // A stray bracket (pointer capture lost, panel hidden mid-drag) would
            // otherwise commit a stale pre-state on the next EndEdit.
            state.history.cancel();
            state.undo();
        }
        I::Redo => {
            state.history.cancel();
            state.redo();
        }
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
        state.doc.active_clip_mut().resolve_roving();
        return;
    }
    state.history.begin(&state.doc);
    f(&mut state.doc, &mut state.selection);
    state.doc.active_clip_mut().resolve_roving();
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

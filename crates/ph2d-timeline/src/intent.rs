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
    /// Duplicate every selected key, offset by `delta_seconds`.
    DuplicateSelection {
        /// Offset applied to the copies.
        delta_seconds: f64,
    },
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
        I::MoveSelectedKeys { delta_seconds } => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| track.move_keys(ids, delta_seconds));
        }),
        I::ScaleSelectedKeys {
            pivot_seconds,
            factor,
        } => edit(state, |doc, sel| {
            for_selected_tracks(doc, sel, |track, ids| {
                track.scale_keys(ids, pivot_seconds, factor)
            });
        }),
        I::DuplicateSelection { delta_seconds } => edit(state, |doc, sel| {
            // The copies become the selection (Blender): the next drag moves the
            // duplicates, not the originals they were made from.
            let mut copies: Vec<SelectedKey> = Vec::new();
            for target in distinct_targets(sel) {
                let ids = sel.ids_for(target);
                if let Some(track) = doc.active_clip_mut().track_mut(target) {
                    copies.extend(
                        track
                            .duplicate_keys(&ids, delta_seconds)
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
        }),
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

        // selection
        I::SelectSingle(k) => state.selection.set_single(k),
        I::ToggleSelect(k) => state.selection.toggle(k),
        I::AddToSelection(k) => state.selection.add(k),
        I::ClearSelection => state.selection.clear(),

        // flags
        I::SetAutoKey(on) => state.flags.auto_key = on,
        I::SetFrameSnap(on) => state.flags.frame_snap = on,

        // history
        I::Undo => {
            state.undo();
        }
        I::Redo => {
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
    state.history.begin(&state.doc);
    f(&mut state.doc, &mut state.selection);
    state.history.commit_if_changed(&state.doc);
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

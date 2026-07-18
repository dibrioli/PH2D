//! [`apply_intent`] — the interpreter for [`crate::TimelineIntent`]: one command in,
//! one undo step out.
//!
//! The vocabulary it interprets lives in the sibling `intent.rs`. Everything here
//! is pure over `(state, playhead)`, which is why every gesture in the panel is
//! testable with no UI at all — the seam test drives these functions directly.

use ph2d_anim::{AnimTarget, AnimValue, Interp, KeyId, RationalTime};
use ph2d_core::Playhead;

use crate::doc::TimelineDoc;
use crate::intent::TimelineIntent;
use crate::state::{SelectedKey, Selection, TimelineState};

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
        // own two seconds and "run" over its own. And it belongs to the VIEW: the
        // Keys tab loops the clip's own clock, the Arrange tab the timeline, each
        // independently (`state.keys_mode` — set by the shell before the drain — says
        // which). The `Playhead` passed IS that view's clock (the shell picks
        // clip_playhead in Keys, playhead in Arrange), so parking the loop in the
        // matching pair and syncing that playhead cannot cross the two views.
        // Written straight (no undo step) — a loop brace is transport, not authoring.
        I::SetLoop { range, ping_pong } => {
            let keys = state.keys_mode;
            state.doc.set_active_loop_for(keys, range);
            state.doc.set_active_ping_pong_for(keys, ping_pong);
            sync_loop(&state.doc, playhead, keys);
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
            sync_loop(&state.doc, playhead, state.keys_mode);
        }
        I::Redo => {
            state.history.cancel();
            state.redo();
            sync_loop(&state.doc, playhead, state.keys_mode);
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
            sync_loop(&state.doc, playhead, state.keys_mode);
        }
        I::AddClip => {
            edit(state, |doc, sel| {
                let name = doc.fresh_clip_name();
                let i = doc.add_clip(name);
                doc.set_active(i);
                sel.clear();
            });
            sync_loop(&state.doc, playhead, state.keys_mode);
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
            sync_loop(&state.doc, playhead, state.keys_mode);
        }
        I::DuplicateClip { index } => {
            edit(state, |doc, sel| {
                // The copy is where you want to be — you asked for it to work on it.
                if let Some(i) = doc.duplicate_clip(index) {
                    doc.set_active(i);
                    sel.clear(); // the selection names keys of the clip we just left
                }
            });
            sync_loop(&state.doc, playhead, state.keys_mode);
        }
        // The selection SURVIVES: a reversed key is the same key at a new time
        // (`KeyId` names a key, not a position), so what was selected still is.
        I::ReverseClip { index } => edit(state, |doc, _| {
            doc.reverse_clip(index);
        }),

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
        I::DuplicateStrip { lane, id } => edit(state, |doc, _| {
            doc.duplicate_strip(lane, id);
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
        I::StretchStrip { lane, id, edge, t } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                stretch_strip(s, edge, t);
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
        // The fade the strip authors for itself (B4).
        //
        // **The clamp is on the SUM, not on each edge.** `weight_at` is `mixIn(t) * mixOut(t)`
        // (Unity's shape), so two fades that OVERLAP multiply: give a 2 s strip a 2 s fade-in
        // and a 2 s fade-out — each perfectly legal on its own — and its weight peaks at
        // **0.25**, never reaching 1. On an `Override` lane that is a sprite permanently
        // half-blended toward the pose below, with nothing on screen to explain why. Unity
        // clamps the sum for exactly this reason, and so do we: a fade can take the whole
        // strip, but the two of them cannot take it twice.
        //
        // It also keeps the two grips from crossing on screen — the tips can meet, never pass.
        I::SetStripEase {
            lane,
            id,
            edge,
            seconds,
        } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                let other = if edge == 0 { s.ease_out } else { s.ease_in };
                let room = (s.span() - other.max(0.0)).max(0.0);
                let v = seconds.clamp(0.0, room); // CLAMP-OK: 0..what the other fade left
                if edge == 0 {
                    s.ease_in = v;
                } else {
                    s.ease_out = v;
                }
            }
        }),
        I::SetStripSpeed { lane, id, speed } => edit(state, |doc, _| {
            if let Some(s) = doc.strip_mut(lane, id) {
                // The span follows the rate, `t_start` pinned — the same edit
                // `stretch_strip` makes, stated as a number instead of felt as a
                // drag (see the variant's docs).
                s.speed = speed.clamp(MIN_STRIP_SPEED, MAX_STRIP_SPEED); // CLAMP-OK: const bounds
                let slice = s.slice();
                if slice > 0.0 {
                    s.t_end = s.t_start + slice / s.speed;
                }
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

/// Move one edge of a strip WITHOUT touching its source slice — the retime.
///
/// The mirror image of [`trim_strip`], and the reason the two are separate
/// functions rather than one with a flag: a trim holds the *rate* and changes
/// *which frames play*; a stretch holds *which frames play* and changes the
/// *rate*. Every strip edit is one or the other, and an editor that blurs them is
/// an editor where an animator lengthens a strip and silently slows the animation
/// down (or shortens one and loses its tail) — the single most reported confusion
/// in every NLE that ever conflated them.
///
/// `speed = slice / span`, so the span is what actually gets clamped: the rate's
/// bounds are span bounds in disguise, and clamping the span (rather than the
/// speed, after the fact) is what keeps `speed` and the drawn box in agreement at
/// the limit. A zero-length slice is a pose, not a clip — it has no rate to change.
fn stretch_strip(s: &mut crate::ClipStrip, edge: u8, t: f64) {
    let slice = s.slice();
    if slice <= 0.0 {
        return;
    }
    // span = slice / speed, so the speed bounds ARE these span bounds.
    let (min_span, max_span) = (slice / MAX_STRIP_SPEED, slice / MIN_STRIP_SPEED);
    if edge == 0 {
        let span = (s.t_end - t.max(0.0)).clamp(min_span, max_span); // CLAMP-OK: derived bounds
        s.t_start = (s.t_end - span).max(0.0);
    } else {
        let span = (t - s.t_start).clamp(min_span, max_span); // CLAMP-OK: derived bounds
        s.t_end = s.t_start + span;
    }
    // Read back the span that actually landed, so the rate describes the box that
    // was drawn even where a clamp bit. Deriving it from the *requested* span is
    // how a strip ends up with a speed its own edges contradict.
    s.speed = slice / s.span();
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
/// Publish the ACTIVE CLIP's loop for one VIEW onto its transport — the doc is the
/// truth, the `Playhead` is the copy. `keys` picks the pair: the Keys-view clip-clock
/// loop (`clip_playhead`) or the Arrange timeline loop (`playhead`).
///
/// Every intent that can change which clip is active (or its range) calls this. **So must
/// anything that swaps the document under the transport** — loading a project, above all:
/// the loop lives in the clip (`NamedClip.loop_range` / `keys_loop_range`, DOC v3/v5), so
/// without this a saved loop never comes back, and — worse — the *previous* project's loop
/// stays armed on the `Playhead` and quietly loops the new project over a range that belongs
/// to a file the artist already closed. The shell also calls it on a TAB switch (which is not
/// an intent) so the now-active clock adopts its own view's loop.
pub fn sync_transport_loop(doc: &TimelineDoc, playhead: &mut Playhead, keys: bool) {
    sync_loop(doc, playhead, keys);
}

fn sync_loop(doc: &TimelineDoc, playhead: &mut Playhead, keys: bool) {
    match doc.active_loop_for(keys) {
        Some((a, b)) => playhead.set_loop(a, b),
        None => playhead.clear_loop(),
    }
    playhead.set_loop_mode(if doc.active_ping_pong_for(keys) {
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

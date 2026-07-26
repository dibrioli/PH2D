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
use crate::strip_edge_edit::{
    MAX_STRIP_SPEED, MIN_STRIP_SPEED, mark_edge, stretch_strip, trim_strip,
};

/// The Buffer-Curves bodies (`store`/`swap`) — sibling module under the LOC cap.
#[path = "intent_apply_buffer.rs"]
mod intent_apply_buffer;
/// Bulk time-transform bodies (scale markers, reverse selection) — child module
/// under the LOC cap, reaching the parent's `edit`/`for_selected_tracks`.
#[path = "intent_apply_time.rs"]
mod intent_apply_time;
/// The loop-sync family (`sync_transport_loop`/`sync_container_loop`/`sync_loop`) — sibling
/// module under the LOC cap.
#[path = "intent_loop_sync.rs"]
mod intent_loop_sync;
/// The post-edit invariant restoration (`settle`) — sibling module under the LOC cap.
#[path = "intent_settle.rs"]
mod intent_settle;
use crate::intent_apply_path;
use intent_loop_sync::{LengthTarget, apply_length, sync_loop};
pub use intent_loop_sync::{sync_container_loop, sync_transport_loop};
use intent_settle::settle;

/// Apply one intent to the timeline state + playhead. Document-mutating intents
/// are grouped as a single undo step.
pub fn apply_intent(state: &mut TimelineState, playhead: &mut Playhead, intent: TimelineIntent) {
    use TimelineIntent as I;
    let fps = state.doc.fps_display;
    // **Where a stack edit lands** — the document's own stack, or the interior of the
    // container the animator has entered. Read ONCE, here, so every arm below routes through
    // the same answer (ADR-0133 §5).
    let host = state.edit_host();
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
            // A brace armed or dragged past the authored duration is pulled back
            // inside it (Enio, 2026-07-23) — the same law every length edit runs.
            state.doc.clamp_loops_to_lengths();
            sync_loop(&state.doc, playhead, keys);
        }
        // Transport-only: the clock is armed, the DOCUMENT is not touched — see the
        // intent's doc for why (inside a container the cycled thing is the walked-into
        // instance, a navigation fact, and the scene's authored loop must survive it).
        I::SetTransportLoop { range, ping_pong } => {
            match range {
                Some((a, b)) => playhead.set_loop(a, b),
                None => playhead.clear_loop(),
            }
            playhead.set_loop_mode(if ping_pong {
                ph2d_core::LoopMode::PingPong
            } else {
                ph2d_core::LoopMode::Wrap
            });
        }
        // The CONTAINER's own loop (Enio, 2026-07-22). Writes the container and syncs
        // the playhead it was handed — the container clock in the interior view. The
        // scene's loop and the clips' are not touched: each mode owns its own.
        I::SetContainerLoop {
            container,
            range,
            ping_pong,
        } => {
            state.doc.set_container_loop(container, range, ping_pong);
            state.doc.clamp_loops_to_lengths();
            sync_container_loop(&state.doc, container, playhead);
        }

        // The explicit durations (Enio, 2026-07-23) — one family, one function
        // (`apply_length`, beside the loop syncs it drives): authoring + the
        // loop clamp + the live resync.
        I::SetSceneLength { len } => apply_length(state, playhead, LengthTarget::Scene, len),
        I::SetClipLength { len } => {
            apply_length(state, playhead, LengthTarget::ActiveClip, len);
        }
        I::SetContainerLength { container, len } => {
            apply_length(state, playhead, LengthTarget::Container(container), len);
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
        I::AddPathKey { entity, t, at } => intent_apply_path::add_key(state, entity, t, at, fps),
        I::ToggleAutoOrient { entity } => intent_apply_path::toggle_orient(state, entity),
        I::ConvertPositionMode { entity, to_path } => {
            intent_apply_path::convert_mode(state, entity, to_path);
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
        // The time-scale box carries the markers in its span (body in the sibling
        // `intent_apply_time` under the LOC cap).
        I::ScaleMarkers {
            indices,
            pivot_seconds,
            factor,
        } => intent_apply_time::scale_markers(state, &indices, pivot_seconds, factor),
        I::ReverseSelectedKeys => intent_apply_time::reverse_selected(state),
        I::DistributeSelectedKeys => intent_apply_time::distribute_selected(state),
        I::StaggerSelectedKeys { step_seconds } => {
            intent_apply_time::stagger_selected(state, step_seconds)
        }
        I::StoreTrackBuffer { target } => intent_apply_buffer::store(state, target),
        I::SwapTrackBuffer { target } => intent_apply_buffer::swap(state, target),
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
        I::SetMarkerSignal { index, signal } => edit(state, |doc, _| {
            doc.set_marker_signal(index, signal);
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
        I::SetSimulatePhysics(on) => state.flags.simulate_physics = on,
        I::SetOnion(onion) => state.onion = onion,

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
                // A new clip opens as a 4 s composition, not a derived-0 one (Enio,
                // 2026-07-23) — the same product default the first clip gets. The
                // AUTHORING layer stamps it; the data `add_clip` stays derived.
                doc.set_clip_length_override(i, Some(crate::DEFAULT_DURATION_SECONDS));
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
        //
        // **Every one of them edits `host`, not the document's stack** (ADR-0133 §5): when the
        // animator has ENTERED a container, the lanes on screen are its interior, and an edit
        // that landed on the document's stack instead would be an edit to something they are
        // not looking at. `StackHost::Document` is the default, so a document that never uses
        // containers takes exactly the path it always did.
        I::AddLane => edit_at(state, host, |doc, host, _| {
            // Name against the HOST's stack, not the document's — a container lane named
            // against the document came out "Lane 1" every time (Enio, 2026-07-23).
            let name = doc.fresh_lane_name_in(host);
            doc.add_lane_in(host, name);
        }),
        // Rename a lane row (Arrange/Container). `edit_at`, so it lands in whichever stack the
        // animator has OPEN — the same routing the lane's own add/mode/mute edits take.
        I::RenameLane { lane, name } => edit_at(state, host, |doc, host, _| {
            doc.rename_lane_in(host, lane, name);
        }),
        // The ASSET, and nothing else — see the intent's docs for why placing is a separate
        // act. `edit`, not `edit_at`: a container belongs to the DOCUMENT's list, so the
        // stack that happens to be open must not decide whose list it joins.
        I::AddContainer => edit(state, |doc, _| {
            let name = doc.fresh_container_name();
            let c = doc.add_container(name);
            // A new container opens as a 4 s composition too (Enio, 2026-07-23) — an
            // empty one's derived 2 s bar is not the default the artist expects, and
            // without an authored duration its veil never shows. Same product default,
            // stamped at authoring.
            doc.set_container_length_override(c, Some(crate::DEFAULT_DURATION_SECONDS));
        }),
        // Renaming an ASSET, so `edit` like its clip sibling — never `edit_at`: which stack
        // is open has nothing to do with whose name this is.
        I::RenameContainer { index, name } => edit(state, |doc, _| {
            doc.rename_container(index, name);
        }),
        // Deleting an ASSET — `edit` for the same reason, and the doc's cascade rides inside
        // the same capture, so the instances and the asset undo as one.
        I::RemoveContainer { index } => edit(state, |doc, _| {
            doc.remove_container(index);
        }),
        I::RemoveLane { lane } => edit_at(state, host, |doc, host, _| {
            doc.remove_lane_in(host, lane);
        }),
        I::SetLaneMuted { lane, muted } => edit_at(state, host, |doc, host, _| {
            if let Some(l) = doc.host_stack_mut(host).and_then(|s| s.get_mut(lane)) {
                l.muted = muted;
            }
        }),
        I::SetLaneMode { lane, mode } => edit_at(state, host, |doc, host, _| {
            if let Some(l) = doc.host_stack_mut(host).and_then(|s| s.get_mut(lane)) {
                l.mode = mode;
            }
        }),
        I::SetLaneWeight { lane, weight } => edit_at(state, host, |doc, host, _| {
            if let Some(l) = doc.host_stack_mut(host).and_then(|s| s.get_mut(lane)) {
                l.weight = weight.clamp(0.0, 1.0); // CLAMP-OK: constant bounds
            }
        }),
        I::AddStrip {
            lane,
            source,
            t_start,
            t_end,
        } => edit_at(state, host, |doc, host, _| {
            let _ = doc.add_strip_to(host, lane, source, t_start.max(0.0), t_end.max(0.0));
        }),
        I::RemoveStrip { lane, id } => edit_at(state, host, |doc, host, _| {
            doc.remove_strip_in(host, lane, id);
        }),
        I::DuplicateStrip { lane, id } => edit_at(state, host, |doc, host, _| {
            doc.duplicate_strip_in(host, lane, id);
        }),
        I::MoveStrip {
            lane,
            to_lane,
            id,
            t_start,
        } => edit_at(state, host, |doc, host, _| {
            doc.move_strip_in(host, lane, to_lane, id, t_start.max(0.0));
        }),
        I::TrimStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                trim_strip(s, edge, t);
                mark_edge(s, false, edge, from);
            }
        }),
        I::StretchStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                stretch_strip(s, edge, t);
                mark_edge(s, true, edge, from);
            }
        }),
        I::SetStripLoop {
            lane,
            id,
            loop_mode,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                s.loop_mode = loop_mode;
            }
        }),
        // Authoring a strip's fades — both live in `intent_apply_fade` (LOC cap). The inward
        // ease and the outward lead, at either edge; each runs inside this `edit_at` bracket.
        I::SetStripEase {
            lane,
            id,
            edge,
            seconds,
        } => edit_at(state, host, |doc, host, _| {
            crate::intent_apply_fade::set_ease(doc, host, lane, id, edge, seconds);
        }),
        I::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => edit_at(state, host, |doc, host, _| {
            crate::intent_apply_fade::set_lead(doc, host, lane, id, edge, seconds);
        }),
        I::SetStripSpeed { lane, id, speed } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                // The span follows the rate, `t_start` pinned — the same edit
                // `stretch_strip` makes, stated as a number instead of felt as a
                // drag (see the variant's docs).
                s.speed = speed.clamp(MIN_STRIP_SPEED, MAX_STRIP_SPEED); // CLAMP-OK: const bounds
                let slice = s.slice();
                if slice > 0.0 {
                    // Same edit, same change bar: typing a rate moves the END edge,
                    // and a mark that only appeared for the DRAG would make the two
                    // paths to one number look like two different edits.
                    let before = s.t_end;
                    s.t_end = s.t_start + slice / s.speed;
                    mark_edge(s, true, 1, before);
                }
            }
        }),
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

/// [`edit`], for a mutation that must land in whichever stack the animator has OPEN.
///
/// The host is passed through rather than read from `state` inside the closure because the
/// closure already borrows `state` mutably — and threading it makes the routing visible at
/// every call site, which is the point: a stack edit that forgot the host would silently
/// edit the document while the animator watched a container.
fn edit_at(
    state: &mut TimelineState,
    host: crate::StackHost,
    f: impl FnOnce(&mut TimelineDoc, crate::StackHost, &mut Selection),
) {
    edit(state, |doc, sel| f(doc, host, sel));
}

/// Run a doc edit as one undo step: snapshot, mutate `(doc, selection)`, commit
/// only if the document changed.
pub(crate) fn edit(state: &mut TimelineState, f: impl FnOnce(&mut TimelineDoc, &mut Selection)) {
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

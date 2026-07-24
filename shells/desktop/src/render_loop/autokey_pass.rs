//! Auto-key pass (W4.T1/T2) — the shell half of "the user moved something, key
//! it at the playhead".
//!
//! Runs ONCE per frame, AFTER every UI-driven Transform/opacity write for the
//! frame (the gizmo drag early, the Inspector + Hierarchy commits late) and after
//! the timeline apply. That placement is what makes it a genuine single choke
//! point despite the scattered write sites: it reads the *resulting* pose of each
//! selected sprite and asks [`ph2d_timeline::autokey_props`] which properties left
//! their curve. Because the apply pass has already written the document's value to
//! the world, an undo / paste / dope-sheet move leaves world == curve and keys
//! nothing — no feedback loop.
//!
//! Undo grouping mirrors the pre-existing gizmo bracket: a gizmo drag is one step
//! across the whole gesture; a discrete edit (Inspector field, Reset) is one step
//! for the frame it lands on.
//!
//! While the transport is **playing**, ordinary auto-key stays inert: the pose
//! is the animation driving the object, not a user edit, so there is nothing to
//! key. The ONE exception is **performing / record** (W5): with `Record` armed,
//! a live gizmo drag records the dragged pose along the playhead (mocap by
//! hand). The guard is strict on purpose — `performing && drag_now` — so the
//! passive pose the animation is driving can NEVER mint a key on its own; only
//! an active manipulation gesture records. (That, plus the diff comparing at
//! the apply's raw clock below, is why a plain Play — even with AutoKey armed —
//! records nothing.)
//!
//! **Disarmed, the pass pins instead of keying.** Without the pin, posing a
//! bound object for a manual K was impossible: the apply pass rewrote the pose
//! from the curve the frame the gesture ended, snapping the object back before
//! K could record it. So a bound pose the user displaced while paused joins
//! [`AutokeyState::displaced`], and the apply skips those entities — the pose
//! holds, Blender-style, until the playhead moves (scrub/play reclaims it for
//! the animation) or it returns to its curve (a K keyed it, or an undo).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_anim::{AnimValue, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::World;
use ph2d_editor::HeroScreen;
use ph2d_timeline::{PoseSample, PropKind, TimelineState, autokey_props};

use super::record_fit::{REC_SIMPLIFY_REL, REC_SMOOTH_PASSES, RecSpan, simplify_recorded};
use super::timeline_bridge::{default_interp, sample_prop_value};

/// The shell-owned state of the auto-key / pose machinery (one per `App`).
#[derive(Default)]
pub(crate) struct AutokeyState {
    /// Last frame's pose per selected entity — the reference for unbound
    /// first-touch auto-create.
    pub baseline: BTreeMap<u64, PoseSample>,
    /// A gizmo-drag undo bracket is open.
    pub drag_active: bool,
    /// Entities whose bound pose the user displaced while PAUSED and disarmed.
    /// The apply pass skips them so the pose holds for a manual K. Cleared by
    /// the bridge when the playhead moves; an entity heals out here when its
    /// pose returns to its curve.
    pub displaced: BTreeSet<u64>,
    /// The playhead time `displaced` was collected at (the bridge clears the
    /// set when the time changes).
    pub displaced_t: f64,
    /// The refusal the animator has already been told about. A drag against an
    /// overriding lane refuses on EVERY frame — sixty identical toasts a second is
    /// not information, it is noise. The toast fires on the rising edge, again if
    /// the REASON changes, and re-arms once the refusals stop.
    pub refusal: Option<ph2d_timeline::KeyRefusal>,
    /// Per `(entity, prop)` recorded span of the in-flight performing session —
    /// what to simplify (and over what tolerance) when the record drag ends.
    /// Empty outside a performing drag.
    pub(crate) record: BTreeMap<(u64, PropKind), RecSpan>,
}

/// Sample a sprite's six animatable values into a [`PoseSample`], in
/// `PropKind::ALL` order. `None` for a property whose backing component is absent
/// (e.g. opacity on an entity with no `Sprite`).
fn sample_pose(world: &World, entity: u64) -> PoseSample {
    let mut pose: PoseSample = [None; 6];
    for (i, &prop) in PropKind::ALL.iter().enumerate() {
        pose[i] = sample_prop_value(world, entity, prop).and_then(|v| match v {
            AnimValue::Float(f) => Some(f),
            _ => None,
        });
    }
    pose
}

/// Auto-key the current selection. `armed` is `panel_open && auto_key`, resolved
/// by the caller; the pass additionally goes inert while the transport is playing
/// (`playhead.is_playing()`). Rebuilds the baseline from the live selection every
/// frame (so it tracks selection changes), brackets the undo step, and manages
/// the displaced-pose pin (see the module docs).
pub(crate) fn run(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    ak: &mut AutokeyState,
    toasts: &mut ph2d_editor::ToastQueue,
    hero: &HeroScreen,
    world: &World,
) {
    let panel_open = hero.is_panel_visible("timeline");
    let armed = panel_open && timeline.flags.auto_key;
    // Performing needs the panel open too (same gate as auto-key): record is a
    // timeline authoring mode, meaningless when its UI is hidden.
    let performing = panel_open && timeline.flags.performing;
    let drag_now = hero.gizmo.drag.is_some();
    // Sample every selected sprite's live pose, in selection order.
    let samples: Vec<(u64, PoseSample)> = hero
        .gizmo
        .iter_selected()
        .map(|e| (e, sample_pose(world, e)))
        .collect();
    apply_samples(
        timeline, playhead, &samples, drag_now, armed, performing, ak, toasts,
    );
}

/// The pure core of the pass: given each selected sprite's sampled pose, key what
/// left its curve and bracket the undo step. Separated from [`run`] (which owns
/// the `HeroScreen`/`World` sampling) so the frame logic — the diff, the
/// auto-create, the bracket — is testable headless.
#[allow(clippy::too_many_arguments)] // the frame's inputs; bundling them hides them
pub(crate) fn apply_samples(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    performing: bool,
    ak: &mut AutokeyState,
    toasts: &mut ph2d_editor::ToastQueue,
) {
    // **Which scene the diff believes in** — the same split the manual K has
    // (`key_authoring_solo` vs `key_value_for`): in the KEYS view the apply solos
    // the active clip at the CLIP playhead, so the pass must author on that clock
    // and against that curve. Before this split, ONE strip in a lane flipped every
    // question here to the stack's blend at the TIMELINE clock — a clock the solo
    // apply was not driving — and auto-key on the Keys tab died in a wall of
    // "does not play here" toasts (Enio, 2026-07-22). The caller passes the
    // matching playhead (clip in keys mode, timeline otherwise).
    let solo = timeline.keys_mode;
    // Inside a container the diff believes in the CONTAINER's blend, at the container
    // clock (Enio, 2026-07-22) — the third view. The scratch is primed ROOTED there
    // (`container_open`), so the root-aware `shown_value`/`key_home`/`key_value_in_active_clip`
    // read the container's lanes, not the scene's. `None` primes the scene (root None),
    // exactly as before.
    let root = timeline.container_open;
    // The stack's scratch must describe THIS instant before anything asks it where
    // a key lands or whether a pose is reachable. In production the apply built it
    // a moment ago at this very playhead, so this costs a compare — but the pass no
    // longer DEPENDS on that having happened, which is the difference between right
    // and accidentally right. Solo never consults the scratch (there is no blend),
    // and priming it with the CLIP clock would poison it for everyone else.
    if !solo {
        timeline.doc.prime_rooted(root, playhead.time());
    }

    // Whether this frame CAPTURES the pose into keys.
    //  - Paused: ordinary auto-key (`armed`) — a UI edit off the curve keys.
    //  - Playing: ONLY performing, and ONLY with a live gizmo drag. The pose
    //    changes every played frame because the animation drives it, not the
    //    user — so capturing the passive pose would mint a key per frame (the
    //    "autoplay creates keyframes" bug). Requiring an active drag makes that
    //    impossible: a plain Play, even with AutoKey armed, never records.
    // The baseline still advances below regardless, so pausing mid-play never
    // misreads the settled pose as a jump.
    let playing = playhead.is_playing();
    let capturing = if playing {
        performing && drag_now
    } else {
        armed
    };
    let fps = timeline.doc.fps_display;
    // **The author's clock is the apply's clock, CUT included** (seed == sample).
    // The apply cuts every clock at the view's authored duration before anything
    // else (`apply_active_clip` / the empty-stack lane of `apply_from_doc_except`
    // cut by the clip; `stack_frames` cuts frame 0 by the scene/container and each
    // strip by `cut_source`). Past the cut the pose is FROZEN at `curve(cut)`, so
    // a diff that reads `curve(raw)` sees a phantom delta and mints a key per
    // scrubbed frame, at a time the apply never samples (the 2026-07-23 superbug).
    // THIS cut is the SOLE correctness boundary — and stays so now that the playhead
    // clamp upstream is GONE (removed 2026-07-25: the playhead is free so the transport
    // can drive physics past the authored end). The playhead routinely runs past the cut;
    // the evaluator freezes the pose at `curve(cut)`, so the diff never sees a phantom
    // delta. The autokey correctness never depended on the clamp — only on this cut.
    // Which cut mirrors which apply:
    //  - Keys/solo, and Arrange with an EMPTY stack: the active clip's own cut.
    //  - A real stack / container: the view's frame-0 cut. The diff there reads
    //    the primed scratch (already cut per strip), and `key_home`'s stacked
    //    branch debug-asserts the clock it is handed matches the scratch's own —
    //    so hand it the same one.
    // All cuts are `t.min(len)`, so within the authored range (and with nothing
    // authored) `t_cut == playhead.time()` and this is byte-identical to before.
    let t_raw = playhead.time();
    let t_cut = if solo || (root.is_none() && timeline.doc.stack().is_empty()) {
        timeline.doc.clip_cut(timeline.doc.active_index(), t_raw)
    } else if let Some(c) = root {
        timeline.doc.container_cut(c, t_raw)
    } else {
        timeline.doc.cut_scene(t_raw)
    };
    let t = ph2d_timeline::snap_time(
        RationalTime::from_seconds(t_cut),
        fps,
        timeline.flags.frame_snap,
    );

    // The fresh-object default: Position is a motion path (the After Effects
    // default). An object already in one mode keeps it — `position_key_mode`
    // resolves that; this is only the fallback for one with no position animation
    // yet, and the per-object toggle marks it otherwise (ADR-0141).
    let default_path = true;

    // Diff each sprite against its curve (bound) or last frame (unbound), and
    // rebuild the baseline in one pass. The diff reads the document BEFORE any
    // upsert, so the whole selection is judged against one consistent state.
    let mut to_key: Vec<(u64, PropKind, f32, RationalTime)> = Vec::new();
    // Motion-path anchors to author (Path mode): `(entity, [x, y], key time)`.
    // Separate from `to_key` because an anchor is 2D geometry, keyed through
    // `key_the_path`, not a scalar upsert.
    let mut to_path: Vec<(u64, [f32; 2], RationalTime)> = Vec::new();
    let mut next_baseline: BTreeMap<u64, PoseSample> = BTreeMap::new();
    // The first refusal this frame, if any (they share a cause far more often than
    // not — one stack, one playhead).
    let mut refused_now: Option<ph2d_timeline::KeyRefusal> = None;
    for &(entity, pose) in samples {
        // The entity's own clock: under a Time Remap its scene tracks are
        // authored in SOURCE time — the diff must compare (and a key must
        // land) at the exact time the apply pass sampled, or world == curve
        // breaks: every pose edit would key at an invisible time and snap
        // back. Identity (the common case, remapped == playhead) keeps the
        // frame-snapped playhead time byte-identical to before. The remap
        // composes ON TOP of the cut (`t_cut`), the apply's own order
        // (cut first, then the entity's clock).
        let t_src = ph2d_timeline::remapped_time(&timeline.doc, entity, t_cut);
        // Where a key lands — the active clip's OWN time. Without a stack that is
        // the entity's clock (above); with one, the strip's map composes on top,
        // and it can have **no answer**: a clip playing twice right now offers two
        // homes for "here", and a clip playing zero times offers none. `key_time`
        // says so rather than guessing (ADR-0115 R9), and a key with no home is
        // simply not written. Moving the object silently is the one outcome that
        // is never acceptable.
        //
        // Performing records in REAL (sub-frame) time: a mocap gesture is a
        // continuous trajectory, and the simplify afterward places clean keys
        // wherever the curve needs them — frame-snapping here would pin each key
        // to a frame while its VALUE is the pose at the un-snapped instant, a
        // mismatch of up to half a frame that the fit would chase (many spurious
        // keys). Paused authoring still snaps (keys on whole frames); a Time Remap
        // always keys at its source time.
        //
        // `key_home`, not `key_time`: the two differ only in that this one carries
        // the REASON, and a refusal the animator cannot see is indistinguishable
        // from a bug (they drag, the object snaps back, nothing says why).
        // Soloed there is no strip map to compose and nothing to refuse — the key
        // lands on the entity's own clip clock, which is `t_src` by construction.
        let home = if solo {
            Ok(t_src)
        } else {
            ph2d_timeline::key_home(&timeline.doc, entity, t_cut)
        };
        let t_e = home.ok().map(|ts| {
            if playing || ts != t_cut {
                RationalTime::from_seconds(ts)
            } else {
                t
            }
        });
        // The diff's reference clock is the RAW `t_src` — the exact `f64` the
        // apply sampled the curve at to write this pose — NOT the frame-snapped
        // `t_e` a new key lands at, and NOT a `RationalTime` round-trip of it.
        //
        // Both mistakes have now been made. Snapping it (2026-07-11): pausing
        // mid-play rests the playhead off-frame (sim dt 1/60 vs display frames
        // 1/24) and `curve(t_raw) != curve(t_snap)`, so an untouched pose read as
        // "dragged". Quantising it (2026-07-12): `RationalTime` rounds to the
        // MICROSECOND, so scrubbing an animated object minted a key every frame —
        // same failure, one thousandth the size, invisible to every test because a
        // test scrubs to 0.5 and 0.5 round-trips exactly.
        //
        // Compare where you read. [[feedback_derived_coordinate_seed_must_match_sample]]
        let t_diff = t_src;
        let base = ak.baseline.get(&entity).copied().unwrap_or([None; 6]);
        if capturing {
            // Performing (playing) records only what the DRAG pushed off the
            // curve; under a plain Play the drag is the sole source of an
            // off-curve pose, so this naturally captures just the dragged
            // entity's trajectory, key per display frame.
            let plan = if solo {
                ph2d_timeline::autokey_props_solo(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    true,
                    default_path,
                )
            } else {
                autokey_props(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    true,
                    default_path,
                )
            };
            // **A refusal is a result, not an absence.** The pose was moved and the
            // clip being edited cannot express it (ADR-0115 R9) — so no key is
            // written, the apply snaps the object back next frame, and the animator
            // is owed a reason. `plan.refused` is non-empty exactly when something
            // was wanted and could not be stored; `home` names WHY when the cause is
            // the strip's map (the clip does not play here, or plays twice), and an
            // Override lane above is what remains.
            if !plan.refused.is_empty() {
                refused_now = Some(home.err().unwrap_or(ph2d_timeline::KeyRefusal::Overridden));
            }
            if let Some(t_e) = t_e {
                // The motion-path anchor (Path mode), authored at the same home time
                // as the scalar keys. During a paused drag `t_e` is fixed, so
                // `key_the_path` moves the single anchor at that instant; while
                // performing, `t_e` advances and lays a trail (a motion sketch).
                if let Some(at) = plan.path_key {
                    to_path.push((entity, at, t_e));
                }
                for (prop, v) in plan.keys {
                    to_key.push((entity, prop, v, t_e));
                    // Track the recorded span so the drag's end can simplify
                    // exactly what it recorded (playing = a performing session,
                    // not a paused one-off edit).
                    if playing {
                        let (ts, vf) = (t_e.to_seconds(), f64::from(v));
                        ak.record
                            .entry((entity, prop))
                            .and_modify(|s| s.extend(ts, vf))
                            .or_insert_with(|| RecSpan::seed(ts, vf));
                    }
                }
            }
        }
        // Displaced-pose pin (paused only — while playing the apply drives the
        // pose, and the bridge clears the set on the time change anyway).
        // Disarmed with a bound prop off its curve = the user posed the object
        // for a manual K: pin it so the apply stops snapping it back. Armed (the
        // diff is keyed above) or back on-curve (K landed / undo) → heal out.
        if !playing {
            // `allow_create = false`: only BOUND props matter here — an unbound
            // one is never overwritten by the apply, so it needs no pin. Same
            // solo split as the capture above: the pin must judge "off its
            // curve" against the scene the apply is actually driving.
            let off_curve = !(if solo {
                ph2d_timeline::autokey_props_solo(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    false,
                    default_path,
                )
            } else {
                autokey_props(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    false,
                    default_path,
                )
            })
            .is_empty();
            if !capturing && off_curve {
                ak.displaced.insert(entity);
            } else {
                ak.displaced.remove(&entity);
            }
        }
        next_baseline.insert(entity, pose);
    }
    ak.baseline = next_baseline;

    // Say it once. On the rising edge, again if the REASON changes, and re-armed
    // when the refusals stop — a drag against an overriding lane refuses on every
    // frame, and sixty identical toasts a second is not information.
    if refused_now != ak.refusal {
        if let Some(r) = refused_now {
            toasts.push(ph2d_editor::Toast::warning(r.message()));
        }
        ak.refusal = refused_now;
    }

    if capturing {
        // A gizmo drag opens one step on its first frame; a discrete edit brackets
        // just this frame. Guard on `is_open` so we never nest inside a panel edit
        // bracket already in flight. A performing session is a drag, so it takes
        // the drag branch — the whole record (across every played frame) commits
        // as ONE undo step when the drag ends, below.
        if drag_now && !ak.drag_active && !timeline.history.is_open() {
            timeline.history.begin(&timeline.doc);
        }
        // A path anchor (`to_path`) is a discrete edit too — in Path mode there are
        // no scalar keys, so bracketing on `to_key` alone would author the anchor
        // outside any undo step.
        let discrete =
            !drag_now && (!to_key.is_empty() || !to_path.is_empty()) && !timeline.history.is_open();
        if discrete {
            timeline.history.begin(&timeline.doc);
        }
        let interp = default_interp();
        for (entity, prop, v, t_e) in &to_key {
            timeline
                .doc
                .upsert_key(*entity, *prop, *t_e, AnimValue::Float(*v), interp);
        }
        // The motion-path anchors (Path mode): `key_the_path` binds Position on
        // first touch, adds/moves the anchor, and rewrites the distances the keys
        // hold — the same door the manual `K` uses.
        for (entity, at, t_e) in &to_path {
            timeline.doc.key_the_path(*entity, *t_e, *at);
        }
        if discrete {
            let doc = timeline.doc.clone();
            timeline.history.commit_if_changed(&doc);
        }
    }

    // Close the gizmo drag's step when the drag ends: one undo step if it changed
    // the document.
    if ak.drag_active && !drag_now {
        // A performing session just ended: simplify the dense per-frame keys it
        // recorded into clean minimal Bézier curves — WITHIN this same undo step,
        // so one Ctrl+Z reverts the whole record + cleanup.
        simplify_recorded(timeline, &ak.record, REC_SMOOTH_PASSES, REC_SIMPLIFY_REL);
        ak.record.clear();
        let doc = timeline.doc.clone();
        timeline.history.commit_if_changed(&doc);
    }
    ak.drag_active = drag_now;
}

#[cfg(test)]
#[path = "autokey_cut_clock_tests.rs"]
mod cut_clock_tests;
#[cfg(test)]
#[path = "autokey_performing_tests.rs"]
mod performing_tests;
#[cfg(test)]
#[path = "autokey_refusal_tests.rs"]
mod refusal_tests;
#[cfg(test)]
#[path = "autokey_test_helpers.rs"]
mod test_helpers;
#[cfg(test)]
#[path = "autokey_pass_tests.rs"]
mod tests;

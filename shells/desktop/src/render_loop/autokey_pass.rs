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

use super::timeline_bridge::{default_interp, sample_prop_value};

/// Value tolerance the record-cleanup fit targets, as a fraction of each track's
/// recorded value range — 1% is visually lossless while cutting the dense
/// per-frame keys to a handful ([`ph2d_anim::fit_fcurve`]). Per-channel: the fit
/// normalises by the range, so this reads the same on a pixel track and a radian
/// track. (Paired with the low-pass below: together they take a noisy 120-sample
/// gesture to ~5-6 clean keys; measured in `curve_fit` calibration.)
const REC_SIMPLIFY_REL: f64 = 0.01;
/// Absolute value-tolerance floor, so a near-constant track (its range ~0) does
/// not get an impossibly tight tolerance that keeps every noise sample.
const REC_SIMPLIFY_FLOOR: f64 = 1e-4;
/// Low-pass passes applied to the recorded values before the fit — strips the
/// hand/mouse tremor that otherwise makes the fit over-subdivide (the "reduziu
/// um pouco" symptom). A binomial `[1,2,1]` kernel ×8 ≈ a ~9-sample window,
/// which at 60 fps is ~150 ms — removes jitter, keeps the gesture's shape.
const REC_SMOOTH_PASSES: usize = 8;
/// Two key times closer than this collapse into ONE dope-sheet column when the
/// session's tracks are aligned — so a channel that turns a frame or two after
/// another still shares its column instead of sitting beside it. ~2 frames at
/// 24 fps: tight enough that genuinely separate beats stay separate.
const COLUMN_MERGE_S: f64 = 0.08;

/// The recorded time+value span of one `(entity, prop)` track during a performing
/// session — enough to simplify exactly the recorded range at a proportional
/// tolerance when the drag ends.
#[derive(Clone, Copy)]
pub(crate) struct RecSpan {
    t_min: f64,
    t_max: f64,
    v_min: f64,
    v_max: f64,
}

impl RecSpan {
    fn seed(t: f64, v: f64) -> Self {
        Self {
            t_min: t,
            t_max: t,
            v_min: v,
            v_max: v,
        }
    }
    fn extend(&mut self, t: f64, v: f64) {
        self.t_min = self.t_min.min(t);
        self.t_max = self.t_max.max(t);
        self.v_min = self.v_min.min(v);
        self.v_max = self.v_max.max(v);
    }
}

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
        timeline, playhead, &samples, drag_now, armed, performing, ak,
    );
}

/// The pure core of the pass: given each selected sprite's sampled pose, key what
/// left its curve and bracket the undo step. Separated from [`run`] (which owns
/// the `HeroScreen`/`World` sampling) so the frame logic — the diff, the
/// auto-create, the bracket — is testable headless.
pub(crate) fn apply_samples(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    performing: bool,
    ak: &mut AutokeyState,
) {
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
    let t = ph2d_timeline::snap_time(
        RationalTime::from_seconds(playhead.time()),
        fps,
        timeline.flags.frame_snap,
    );

    // Diff each sprite against its curve (bound) or last frame (unbound), and
    // rebuild the baseline in one pass. The diff reads the document BEFORE any
    // upsert, so the whole selection is judged against one consistent state.
    let mut to_key: Vec<(u64, PropKind, f32, RationalTime)> = Vec::new();
    let mut next_baseline: BTreeMap<u64, PoseSample> = BTreeMap::new();
    for &(entity, pose) in samples {
        // The entity's own clock: under a Time Remap its scene tracks are
        // authored in SOURCE time — the diff must compare (and a key must
        // land) at the exact time the apply pass sampled, or world == curve
        // breaks: every pose edit would key at an invisible time and snap
        // back. Identity (the common case, remapped == playhead) keeps the
        // frame-snapped playhead time byte-identical to before.
        let t_src = ph2d_timeline::remapped_time(&timeline.doc, entity, playhead.time());
        // Where a key lands. Performing records in REAL (sub-frame) time: a mocap
        // gesture is a continuous trajectory, and the simplify afterward places
        // clean keys wherever the curve needs them — frame-snapping here would
        // pin each key to a frame while its VALUE is the pose at the un-snapped
        // instant, a mismatch of up to half a frame that the fit would chase
        // (many spurious keys). Paused authoring still snaps (keys on whole
        // frames); a Time Remap always keys at its source time.
        let t_e = if playing || t_src != playhead.time() {
            RationalTime::from_seconds(t_src)
        } else {
            t
        };
        // The diff's reference clock is the RAW `t_src` — the exact time the
        // apply sampled — NOT the frame-snapped `t_e` a new key lands at.
        // Pausing mid-play rests the playhead off-frame (sim dt 1/60 vs
        // display frames 1/24), and `curve(t_raw) != curve(t_snap)` there: an
        // untouched pose would read as "dragged", so armed minted a key out
        // of thin air and disarmed pinned the entity (freezing it against
        // every timeline edit, undo included) — Enio, 2026-07-11. Same lesson
        // as the Time-remap K seed: compare where you read.
        let t_diff = RationalTime::from_seconds(t_src);
        let base = ak.baseline.get(&entity).copied().unwrap_or([None; 6]);
        if capturing {
            // Performing (playing) records only what the DRAG pushed off the
            // curve; under a plain Play the drag is the sole source of an
            // off-curve pose, so this naturally captures just the dragged
            // entity's trajectory, key per display frame.
            for (prop, v) in autokey_props(&timeline.doc, entity, t_diff, &pose, &base, true) {
                to_key.push((entity, prop, v, t_e));
                // Track the recorded span so the drag's end can simplify exactly
                // what it recorded (playing = a performing session, not a paused
                // one-off edit).
                if playing {
                    let (ts, vf) = (t_e.to_seconds(), f64::from(v));
                    ak.record
                        .entry((entity, prop))
                        .and_modify(|s| s.extend(ts, vf))
                        .or_insert_with(|| RecSpan::seed(ts, vf));
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
            // one is never overwritten by the apply, so it needs no pin.
            let off_curve =
                !autokey_props(&timeline.doc, entity, t_diff, &pose, &base, false).is_empty();
            if !capturing && off_curve {
                ak.displaced.insert(entity);
            } else {
                ak.displaced.remove(&entity);
            }
        }
        next_baseline.insert(entity, pose);
    }
    ak.baseline = next_baseline;

    if capturing {
        // A gizmo drag opens one step on its first frame; a discrete edit brackets
        // just this frame. Guard on `is_open` so we never nest inside a panel edit
        // bracket already in flight. A performing session is a drag, so it takes
        // the drag branch — the whole record (across every played frame) commits
        // as ONE undo step when the drag ends, below.
        if drag_now && !ak.drag_active && !timeline.history.is_open() {
            timeline.history.begin(&timeline.doc);
        }
        let discrete = !drag_now && !to_key.is_empty() && !timeline.history.is_open();
        if discrete {
            timeline.history.begin(&timeline.doc);
        }
        let interp = default_interp();
        for (entity, prop, v, t_e) in &to_key {
            timeline
                .doc
                .upsert_key(*entity, *prop, *t_e, AnimValue::Float(*v), interp);
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
        simplify_recorded(timeline, &ak.record);
        ak.record.clear();
        let doc = timeline.doc.clone();
        timeline.history.commit_if_changed(&doc);
    }
    ak.drag_active = drag_now;
}

/// Clean up everything one performing session recorded, **per entity**: fit each
/// of its tracks, then re-fit them all at ONE shared set of key times so the dope
/// sheet reads as aligned columns (Enio: "keys for x and y of translate and scale
/// created at the same point in time"). Column-aligned keys are what hand-keyed
/// animation looks like — the animator grabs a column and re-times every channel
/// of the object together.
///
/// The shared times are the union of each track's own extrema, with near-coincident
/// times merged ([`COLUMN_MERGE_S`]) so two channels that turn at *almost* the same
/// instant land on one column instead of two a frame apart. `simplify_range_at`
/// then pins every track's keys to those times (no splitting — a split would land
/// off the column grid).
fn simplify_recorded(timeline: &mut TimelineState, record: &BTreeMap<(u64, PropKind), RecSpan>) {
    // Group the session's tracks by entity — alignment is per OBJECT (two objects
    // recorded at once keep independent timing).
    let mut by_entity: BTreeMap<u64, Vec<(PropKind, RecSpan)>> = BTreeMap::new();
    for (&(entity, prop), &span) in record {
        by_entity.entry(entity).or_default().push((prop, span));
    }
    for (entity, props) in by_entity {
        // Pass 1 — each track's own fit proposes the times it wants (its extrema).
        let mut times: Vec<f64> = Vec::new();
        for &(prop, span) in &props {
            let Some(target) = timeline.doc.binding_for(entity, prop).map(|b| b.target) else {
                continue;
            };
            let Some(track) = timeline.doc.active_clip().track(target) else {
                continue;
            };
            let channel = prop.fit_channel();
            let Some(rs) = track.range_samples(span.t_min, span.t_max, channel, REC_SMOOTH_PASSES)
            else {
                continue;
            };
            // Tolerance off the PREPARED samples, not the raw `RecSpan`: an angle
            // channel's raw extent is one wrapped turn (~2π) however many times it
            // actually spun, so a raw-derived tolerance would be far too tight for
            // the unwrapped curve the fit now sees.
            let tol = value_tol(&rs.samples);
            times.extend(
                ph2d_anim::fit_fcurve(&rs.samples, tol, channel.bounds)
                    .iter()
                    .map(|k| k.t),
            );
        }
        if times.is_empty() {
            continue;
        }
        // Pass 2 — merge near-coincident times into single columns.
        times.sort_by(f64::total_cmp);
        let mut columns: Vec<f64> = Vec::with_capacity(times.len());
        for t in times {
            if columns.last().is_none_or(|&c| t - c > COLUMN_MERGE_S) {
                columns.push(t);
            }
        }
        // Pass 3 — re-fit every track of this entity AT those columns.
        for &(prop, span) in &props {
            let Some(target) = timeline.doc.binding_for(entity, prop).map(|b| b.target) else {
                continue;
            };
            if let Some(track) = timeline.doc.active_clip_mut().track_mut(target) {
                track.simplify_range_at(
                    span.t_min,
                    span.t_max,
                    &columns,
                    prop.fit_channel(),
                    REC_SMOOTH_PASSES,
                );
            }
        }
    }
}

/// The fit tolerance for one recorded track: a fraction of ITS value range, with
/// an absolute floor so a near-constant channel is not held to an impossible bar.
/// Measured on the PREPARED samples (see the caller) — the range the fit will
/// actually see, which for an unwrapped spin is every turn, not one.
fn value_tol(samples: &[(f64, f64)]) -> f64 {
    let (mut v_min, mut v_max) = (f64::MAX, f64::MIN);
    for &(_, v) in samples {
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    (REC_SIMPLIFY_REL * (v_max - v_min)).max(REC_SIMPLIFY_FLOOR)
}

#[cfg(test)]
#[path = "autokey_performing_tests.rs"]
mod performing_tests;
#[cfg(test)]
#[path = "autokey_test_helpers.rs"]
mod test_helpers;
#[cfg(test)]
#[path = "autokey_pass_tests.rs"]
mod tests;

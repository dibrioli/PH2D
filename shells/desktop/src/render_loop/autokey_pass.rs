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
//! While the transport is **playing**, the pass is inert: the pose is the
//! animation driving the object, not a user edit, so there is nothing to key
//! (record-during-play "performing" is W5, not v1).
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
    let armed = hero.is_panel_visible("timeline") && timeline.flags.auto_key;
    let drag_now = hero.gizmo.drag.is_some();
    // Sample every selected sprite's live pose, in selection order.
    let samples: Vec<(u64, PoseSample)> = hero
        .gizmo
        .iter_selected()
        .map(|e| (e, sample_pose(world, e)))
        .collect();
    apply_samples(timeline, playhead, &samples, drag_now, armed, ak);
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
    ak: &mut AutokeyState,
) {
    // While the transport is PLAYING the pose changes every frame because the
    // animation is driving the object, not the user — so auto-key must stay
    // silent (v1 has no record-during-play "performing"; that is W5). Without
    // this, the apply pass writes world = curve(raw playhead t) each frame while
    // the off-curve diff below compares against curve(snapped t); the two
    // disagree under frame-snap / float drift, so every played frame would mint a
    // spurious key. The baseline still advances below (exactly as when disarmed),
    // so pausing mid-play never misreads the settled pose as a jump.
    let playing = playhead.is_playing();
    let armed = armed && !playing;
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
        let t_e = if t_src == playhead.time() {
            t
        } else {
            RationalTime::from_seconds(t_src)
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
        if armed {
            for (prop, v) in autokey_props(&timeline.doc, entity, t_diff, &pose, &base, true) {
                to_key.push((entity, prop, v, t_e));
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
            if !armed && off_curve {
                ak.displaced.insert(entity);
            } else {
                ak.displaced.remove(&entity);
            }
        }
        next_baseline.insert(entity, pose);
    }
    ak.baseline = next_baseline;

    if armed {
        // A gizmo drag opens one step on its first frame; a discrete edit brackets
        // just this frame. Guard on `is_open` so we never nest inside a panel edit
        // bracket already in flight.
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
        let doc = timeline.doc.clone();
        timeline.history.commit_if_changed(&doc);
    }
    ak.drag_active = drag_now;
}

#[cfg(test)]
#[path = "autokey_pass_tests.rs"]
mod tests;

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
    let mut to_key: Vec<(u64, PropKind, f32)> = Vec::new();
    let mut next_baseline: BTreeMap<u64, PoseSample> = BTreeMap::new();
    for &(entity, pose) in samples {
        let base = ak.baseline.get(&entity).copied().unwrap_or([None; 6]);
        if armed {
            for (prop, v) in autokey_props(&timeline.doc, entity, t, &pose, &base, true) {
                to_key.push((entity, prop, v));
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
                !autokey_props(&timeline.doc, entity, t, &pose, &base, false).is_empty();
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
        for (entity, prop, v) in &to_key {
            timeline
                .doc
                .upsert_key(*entity, *prop, t, AnimValue::Float(*v), interp);
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
mod tests {
    use super::*;
    use ph2d_timeline::{PropKind, TimelineIntent as I, apply_intent};

    const E: u64 = 1;
    const TX: usize = 0;
    const ROT: usize = 2;

    fn state_with_tx_track() -> (TimelineState, Playhead) {
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        for (t, v) in [(0.0, 0.0), (1.0, 10.0)] {
            apply_intent(
                &mut st,
                &mut ph,
                I::AddKey {
                    entity: E,
                    prop: PropKind::TranslationX,
                    t: RationalTime::from_seconds(t),
                    value: AnimValue::Float(v),
                    interp: ph2d_anim::Interp::Linear,
                },
            );
        }
        ph.seek(0.5); // curve says x = 5 here
        ph.pause(); // the tests edit while PAUSED; play-suppression is its own test
        (st, ph)
    }

    fn pose(vals: &[(usize, f32)]) -> PoseSample {
        let mut p: PoseSample = [None; 6];
        for &(i, v) in vals {
            p[i] = Some(v);
        }
        p
    }

    fn tx_at(st: &TimelineState, t: f64) -> Option<f32> {
        use ph2d_anim::AttributeEvaluator;
        let target = st.doc.binding_for(E, PropKind::TranslationX)?.target;
        match st.doc.active_clip().track(target)?.sample(t) {
            AnimValue::Float(v) => Some(v),
            _ => None,
        }
    }

    /// One frame of the pass with the given selection samples.
    fn frame(
        st: &mut TimelineState,
        ph: &Playhead,
        samples: &[(u64, PoseSample)],
        drag_now: bool,
        armed: bool,
        ak: &mut AutokeyState,
    ) {
        apply_samples(st, ph, samples, drag_now, armed, ak);
    }

    #[test]
    fn a_bound_prop_dragged_off_its_curve_gets_keyed_in_one_undo_step() {
        let (mut st, ph) = state_with_tx_track();
        let mut ak = AutokeyState::default();
        let before = st.doc.clone();
        // A discrete edit (no gizmo drag): world x = 7 at t = 0.5, curve says 5.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut ak,
        );
        assert_eq!(
            tx_at(&st, 0.5),
            Some(7.0),
            "the pose was keyed at the playhead"
        );
        // Exactly one undo step, and it reverts the whole thing.
        apply_intent(&mut st, &mut Playhead::new(1.0 / 60.0), I::Undo);
        assert_eq!(st.doc, before, "the discrete auto-key is one undo step");
    }

    #[test]
    fn a_pose_on_its_curve_keys_nothing() {
        // THE anti-feedback case in the shell glue: world == curve → no key, so an
        // undo/paste/scrub (which the apply writes back to the world) is not undone.
        let (mut st, ph) = state_with_tx_track();
        let before = st.doc.clone();
        let mut ak = AutokeyState::default();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 5.0)]))],
            false,
            true,
            &mut ak,
        );
        assert_eq!(st.doc, before, "on-curve pose left the document untouched");
    }

    #[test]
    fn a_gizmo_drag_is_a_single_undo_step_across_many_frames() {
        let (mut st, ph) = state_with_tx_track();
        let before = st.doc.clone();
        let mut ak = AutokeyState::default();
        // Three frames of a drag (drag_now = true), the pose creeping 6 → 7 → 8.
        for x in [6.0, 7.0, 8.0] {
            frame(&mut st, &ph, &[(E, pose(&[(TX, x)]))], true, true, &mut ak);
        }
        assert_eq!(tx_at(&st, 0.5), Some(8.0), "the last dragged pose stuck");
        // Drag ends (drag_now = false) → the step commits.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 8.0)]))],
            false,
            true,
            &mut ak,
        );
        // ONE undo reverts the whole drag, not one frame of it.
        apply_intent(&mut st, &mut Playhead::new(1.0 / 60.0), I::Undo);
        assert_eq!(st.doc, before, "the whole drag undoes in one step");
    }

    #[test]
    fn an_unbound_prop_that_moves_auto_creates_a_track() {
        let (mut st, ph) = state_with_tx_track();
        let mut ak = AutokeyState::default();
        assert!(st.doc.binding_for(E, PropKind::Rotation).is_none());
        // Frame 1 establishes the baseline (rotation = 0), keys nothing new.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(ROT, 0.0)]))],
            false,
            true,
            &mut ak,
        );
        assert!(
            st.doc.binding_for(E, PropKind::Rotation).is_none(),
            "no track from mere baseline"
        );
        // Frame 2 the rotation moved → auto-create the track + key.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(ROT, 1.5)]))],
            false,
            true,
            &mut ak,
        );
        assert!(
            st.doc.binding_for(E, PropKind::Rotation).is_some(),
            "track auto-created"
        );
    }

    #[test]
    fn disarmed_keys_nothing_but_still_tracks_the_baseline() {
        // With auto-key off, a move records no key — but the baseline must still
        // advance, or arming mid-drag would treat a stale pose as a jump.
        let (mut st, ph) = state_with_tx_track();
        let before = st.doc.clone();
        let mut ak = AutokeyState::default();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            false,
            &mut ak,
        );
        assert_eq!(st.doc, before, "disarmed: no key");
        assert_eq!(
            ak.baseline.get(&E).and_then(|p| p[TX]),
            Some(7.0),
            "baseline advanced anyway"
        );
    }

    #[test]
    fn deselecting_an_entity_drops_it_from_the_baseline() {
        let (mut st, ph) = state_with_tx_track();
        let mut ak = AutokeyState::default();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut ak,
        );
        assert!(ak.baseline.contains_key(&E));
        // Next frame nothing is selected → the baseline empties.
        frame(&mut st, &ph, &[], false, true, &mut ak);
        assert!(
            ak.baseline.is_empty(),
            "an unselected entity leaves the baseline"
        );
    }

    #[test]
    fn a_disarmed_displaced_pose_pins_the_entity_for_a_manual_k() {
        // THE pose-to-pose bug: without auto-key, the apply pass snapped a
        // posed object back to its curve before K could record it. Disarmed,
        // an off-curve bound pose must join the displaced set (the apply skips
        // it); once the pose returns to the curve (K keyed it / an undo), it
        // heals out.
        let (mut st, ph) = state_with_tx_track(); // paused; curve says x = 5
        let mut ak = AutokeyState::default();
        // Disarmed, pose off-curve (7 vs 5) → pinned, and nothing keyed.
        let before = st.doc.clone();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            false,
            &mut ak,
        );
        assert!(ak.displaced.contains(&E), "off-curve disarmed pose pins");
        assert_eq!(st.doc, before, "disarmed: still no key");
        // The pose returns to the curve (a K keyed it, or an undo) → heals out.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 5.0)]))],
            false,
            false,
            &mut ak,
        );
        assert!(ak.displaced.is_empty(), "on-curve pose heals the pin out");
        // Armed, the same off-curve pose is KEYED, never pinned.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut ak,
        );
        assert!(ak.displaced.is_empty(), "armed keys instead of pinning");
        assert_eq!(tx_at(&st, 0.5), Some(7.0), "armed: the pose was keyed");
    }

    #[test]
    fn playing_does_not_auto_key_even_when_the_pose_looks_off_its_curve() {
        // The Play + AutoKey regression: during playback the apply pass rewrites
        // the pose every frame, and the frame-snap/raw-time mismatch made the
        // off-curve diff fire, minting a key per frame. While playing, the pass
        // must be inert regardless of the pose.
        let (mut st, mut ph) = state_with_tx_track(); // paused; curve x = 5 at 0.5
        let before = st.doc.clone();
        let mut ak = AutokeyState::default();
        // Armed (panel open + auto_key on) and the pose looks off-curve (7 vs 5),
        // but the transport is PLAYING → nothing is keyed.
        ph.play();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut ak,
        );
        assert_eq!(st.doc, before, "playing suppresses auto-key");
        assert_eq!(
            ak.baseline.get(&E).and_then(|p| p[TX]),
            Some(7.0),
            "the baseline still advances while playing (so pausing is not a jump)"
        );
        // Control: the SAME off-curve pose DOES key once PAUSED, proving the play
        // gate — not something else — is what suppressed it.
        ph.pause();
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut ak,
        );
        assert_eq!(
            tx_at(&st, 0.5),
            Some(7.0),
            "paused: the off-curve pose keys as before"
        );
    }
}

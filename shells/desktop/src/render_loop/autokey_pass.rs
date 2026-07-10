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

use std::collections::BTreeMap;

use ph2d_anim::{AnimValue, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::World;
use ph2d_editor::HeroScreen;
use ph2d_timeline::{PoseSample, PropKind, TimelineState, autokey_props};

use super::timeline_bridge::{default_interp, sample_prop_value};

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
/// by the caller. Rebuilds `baseline` from the live selection every frame (so it
/// tracks selection changes) and brackets the undo step via `drag_active`.
pub(crate) fn run(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    baseline: &mut BTreeMap<u64, PoseSample>,
    drag_active: &mut bool,
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
    apply_samples(
        timeline,
        playhead,
        &samples,
        drag_now,
        armed,
        baseline,
        drag_active,
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
    baseline: &mut BTreeMap<u64, PoseSample>,
    drag_active: &mut bool,
) {
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
        if armed {
            let base = baseline.get(&entity).copied().unwrap_or([None; 6]);
            for (prop, v) in autokey_props(&timeline.doc, entity, t, &pose, &base, true) {
                to_key.push((entity, prop, v));
            }
        }
        next_baseline.insert(entity, pose);
    }
    *baseline = next_baseline;

    if armed {
        // A gizmo drag opens one step on its first frame; a discrete edit brackets
        // just this frame. Guard on `is_open` so we never nest inside a panel edit
        // bracket already in flight.
        if drag_now && !*drag_active && !timeline.history.is_open() {
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
    if *drag_active && !drag_now {
        let doc = timeline.doc.clone();
        timeline.history.commit_if_changed(&doc);
    }
    *drag_active = drag_now;
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
        baseline: &mut BTreeMap<u64, PoseSample>,
        drag_active: &mut bool,
    ) {
        apply_samples(st, ph, samples, drag_now, armed, baseline, drag_active);
    }

    #[test]
    fn a_bound_prop_dragged_off_its_curve_gets_keyed_in_one_undo_step() {
        let (mut st, ph) = state_with_tx_track();
        let mut base = BTreeMap::new();
        let mut drag = false;
        let before = st.doc.clone();
        // A discrete edit (no gizmo drag): world x = 7 at t = 0.5, curve says 5.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut base,
            &mut drag,
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
        let mut base = BTreeMap::new();
        let mut drag = false;
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 5.0)]))],
            false,
            true,
            &mut base,
            &mut drag,
        );
        assert_eq!(st.doc, before, "on-curve pose left the document untouched");
    }

    #[test]
    fn a_gizmo_drag_is_a_single_undo_step_across_many_frames() {
        let (mut st, ph) = state_with_tx_track();
        let before = st.doc.clone();
        let mut base = BTreeMap::new();
        let mut drag = false;
        // Three frames of a drag (drag_now = true), the pose creeping 6 → 7 → 8.
        for x in [6.0, 7.0, 8.0] {
            frame(
                &mut st,
                &ph,
                &[(E, pose(&[(TX, x)]))],
                true,
                true,
                &mut base,
                &mut drag,
            );
        }
        assert_eq!(tx_at(&st, 0.5), Some(8.0), "the last dragged pose stuck");
        // Drag ends (drag_now = false) → the step commits.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 8.0)]))],
            false,
            true,
            &mut base,
            &mut drag,
        );
        // ONE undo reverts the whole drag, not one frame of it.
        apply_intent(&mut st, &mut Playhead::new(1.0 / 60.0), I::Undo);
        assert_eq!(st.doc, before, "the whole drag undoes in one step");
    }

    #[test]
    fn an_unbound_prop_that_moves_auto_creates_a_track() {
        let (mut st, ph) = state_with_tx_track();
        let mut base = BTreeMap::new();
        let mut drag = false;
        assert!(st.doc.binding_for(E, PropKind::Rotation).is_none());
        // Frame 1 establishes the baseline (rotation = 0), keys nothing new.
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(ROT, 0.0)]))],
            false,
            true,
            &mut base,
            &mut drag,
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
            &mut base,
            &mut drag,
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
        let mut base = BTreeMap::new();
        let mut drag = false;
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            false,
            &mut base,
            &mut drag,
        );
        assert_eq!(st.doc, before, "disarmed: no key");
        assert_eq!(
            base.get(&E).and_then(|p| p[TX]),
            Some(7.0),
            "baseline advanced anyway"
        );
    }

    #[test]
    fn deselecting_an_entity_drops_it_from_the_baseline() {
        let (mut st, ph) = state_with_tx_track();
        let mut base = BTreeMap::new();
        let mut drag = false;
        frame(
            &mut st,
            &ph,
            &[(E, pose(&[(TX, 7.0)]))],
            false,
            true,
            &mut base,
            &mut drag,
        );
        assert!(base.contains_key(&E));
        // Next frame nothing is selected → the baseline empties.
        frame(&mut st, &ph, &[], false, true, &mut base, &mut drag);
        assert!(base.is_empty(), "an unselected entity leaves the baseline");
    }
}

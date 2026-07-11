//! Timeline bridge — the per-frame glue between the app-general
//! [`TimelineState`] and the scene.
//!
//! Once per frame it drains the pending [`TimelineIntent`]s (from the panel /
//! auto-key) through `apply_intent` — which mutates the document + transport +
//! selection, one undo step per gesture — and then applies the document to the
//! world at the Playhead via `apply_from_doc`. Both halves are pure, tested
//! functions in `ph2d-timeline`; this module only composes them (mirrors how
//! `motion_bridge` / `vector_bridge` compose their crate logic).
//!
//! A no-op when the document is empty (no bindings) and no intents are pending,
//! so any programmatic `SpriteAnimation` bind is left untouched.

use ph2d_core::Playhead;
use ph2d_ecs::World;
use ph2d_editor::tool::PanelEvent;
use ph2d_timeline::{PropKind, TimelineIntent, TimelineState, apply_from_doc_except, apply_intent};

/// Drain pending intents into `timeline`, then apply its document to `world` at
/// the current `playhead` time. The apply leaves untouched: `live_entity` (the
/// entity whose gizmo is being dragged this frame, if any) and every entity in
/// `ak.displaced` (a pose the user displaced while paused, waiting for a manual
/// K — see `autokey_pass`), so the document does not fight the manipulation.
/// Call each frame in the apply pass, after `apply_sprite_animations`.
pub(crate) fn run(
    world: &mut World,
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    intents: &mut Vec<TimelineIntent>,
    live_entity: Option<u64>,
    ak: &mut super::autokey_pass::AutokeyState,
) {
    for intent in intents.drain(..) {
        apply_intent(timeline, playhead, intent);
    }
    // A playhead move (scrub, play, frame step) reclaims every displaced pose
    // for the animation — Blender semantics: the un-keyed pose is discarded.
    if playhead.time() != ak.displaced_t {
        ak.displaced.clear();
        ak.displaced_t = playhead.time();
    }
    let displaced = &ak.displaced;
    apply_from_doc_except(world, &mut timeline.doc, playhead.time(), |bits| {
        live_entity == Some(bits) || displaced.contains(&bits)
    });
    // Identity upkeep: a deleted object's rows leave the view (the apply above
    // just flagged them missing), and an object that comes back under the same
    // name — the global undo respawns entities with FRESH bits — reconnects.
    crate::timeline_persist::upkeep(timeline, world);
}

/// Translate a transport [`PanelEvent`] (by widget id) into a [`TimelineIntent`].
/// The timeline semantics live here so editor-core stays timeline-agnostic;
/// frame-relative and duration-relative commands read the current
/// `playhead`/`timeline`. Returns `None` for ids this panel does not own.
pub(crate) fn intent_for_transport(
    ev: &PanelEvent,
    timeline: &TimelineState,
    playhead: &Playhead,
) -> Option<TimelineIntent> {
    use TimelineIntent as I;
    use ph2d_editor::ids;
    let fps = timeline.doc.fps_display;
    // "The end" is the last keyframe when it runs past the authored duration —
    // a fresh clip's duration is 0, which would pin go-to-end (and the loop
    // range) at t = 0 for every hand-keyed animation.
    let duration = || timeline.doc.end_seconds();
    match *ev {
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY => Some(I::TogglePlay),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_START => Some(I::Scrub(0.0)),
        PanelEvent::Click(id) if id == ids::TIMELINE_ADD_MARKER => Some(I::AddMarker {
            t_seconds: playhead.time(),
            label: format!("M{}", timeline.doc.markers().len() + 1),
        }),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_END => Some(I::Scrub(duration())),
        PanelEvent::Click(id) if id == ids::TIMELINE_PREV_FRAME => {
            Some(I::SeekFrame(playhead.frame(fps) - 1))
        }
        PanelEvent::Click(id) if id == ids::TIMELINE_NEXT_FRAME => {
            Some(I::SeekFrame(playhead.frame(fps) + 1))
        }
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_TIME_NUM => Some(I::Scrub(v)),
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_RULER => Some(I::Scrub(v)),
        PanelEvent::SetValue(id, v) if id == ids::TIMELINE_FRAME_NUM => {
            Some(I::SeekFrame(v as i64))
        }
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_LOOP => Some(if on {
            I::SetLoop(Some((0.0, duration().max(1.0 / fps.max(1.0)))))
        } else {
            I::SetLoop(None)
        }),
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_AUTOKEY => Some(I::SetAutoKey(on)),
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_SNAP => Some(I::SetFrameSnap(on)),
        _ => None,
    }
}

/// Whether this transport event **jumps** the playhead to a time the current
/// view may not show — go-to-start/end, a frame step off the edge, a typed time
/// or frame. The shell asks the panel to pan the playhead back into view after
/// applying one (the panel page-follows only while playing).
///
/// Deliberately excludes the ruler scrub (its value is a fraction of the visible
/// span, so it can never land off-screen) and the flag toggles.
pub(crate) fn jumps_the_playhead(ev: &PanelEvent) -> bool {
    use ph2d_editor::ids;
    match *ev {
        PanelEvent::Click(id) => {
            id == ids::TIMELINE_GO_START
                || id == ids::TIMELINE_GO_END
                || id == ids::TIMELINE_PREV_FRAME
                || id == ids::TIMELINE_NEXT_FRAME
        }
        PanelEvent::SetValue(id, _) => {
            id == ids::TIMELINE_TIME_NUM || id == ids::TIMELINE_FRAME_NUM
        }
        _ => false,
    }
}

/// Sample a bound property's CURRENT value from the scene, for a K-insert
/// keyframe (capture-the-pose). Transform properties read the entity's
/// `Transform`; opacity reads `Sprite.tint[3]`.
pub(crate) fn sample_prop_value(
    world: &World,
    entity_bits: u64,
    prop: PropKind,
) -> Option<ph2d_anim::AnimValue> {
    use ph2d_anim::AnimValue::Float;
    use ph2d_ecs::{Entity, Transform};
    let e = Entity::from_bits(entity_bits);
    let xf = || world.get::<Transform>(e);
    Some(match prop {
        PropKind::TranslationX => Float(xf()?.translation.x),
        PropKind::TranslationY => Float(xf()?.translation.y),
        PropKind::Rotation => Float(xf()?.rotation),
        PropKind::ScaleX => Float(xf()?.scale.x),
        PropKind::ScaleY => Float(xf()?.scale.y),
        PropKind::Opacity => Float(world.get::<ph2d_render::Sprite>(e)?.tint[3]),
        // The timeline's own clock has no scene value to sample — the K flow
        // seeds it through `key_value_for` instead.
        PropKind::TimeRemap => return None,
    })
}

/// The value a K-inserted key carries for `prop` at playhead `t_secs`: scene
/// properties sample the live world ([`sample_prop_value`]); **Time Remap** has
/// no scene value — a new key lands ON the clock the entity ALREADY plays
/// ([`ph2d_timeline::remapped_time`]: identity on an empty track, on-curve
/// between keys, slope-1 extrapolation past them, Hold's freeze respected), so
/// K never bends the remap. Seeding through any OTHER transform than the one
/// the apply samples with re-creates the freeze: `tr.sample` flat-clamps past
/// the last key, so K@0 then K@2 laid down a FLAT map = every track of the
/// entity frozen at source 0 (the 2026-07-11 "Time nullifies the animation").
pub(crate) fn key_value_for(
    world: &World,
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    t_secs: f64,
) -> Option<ph2d_anim::AnimValue> {
    if prop == PropKind::TimeRemap {
        let source = ph2d_timeline::remapped_time(&timeline.doc, entity, t_secs);
        return Some(ph2d_anim::AnimValue::Float(source as f32));
    }
    sample_prop_value(world, entity, prop)
}

/// The track time a K-inserted key lands at, for playhead `t_secs`: scene
/// props key at the entity's own clock ([`ph2d_timeline::remapped_time`]) —
/// tracks are authored in SOURCE time, the same time the apply samples them
/// at, so under a Time Remap the captured pose stays visible at the playhead
/// instead of landing at an invisible time and snapping back. The Time track
/// itself keys at the playhead (the map lives in playhead time). Identity /
/// no remap: `t_secs` unchanged.
pub(crate) fn key_insert_time(
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    t_secs: f64,
) -> ph2d_anim::RationalTime {
    let t = if prop == PropKind::TimeRemap {
        t_secs
    } else {
        ph2d_timeline::remapped_time(&timeline.doc, entity, t_secs)
    };
    ph2d_anim::RationalTime::from_seconds(t)
}

/// The default interpolation for a freshly inserted key (a gentle ease).
pub(crate) fn default_interp() -> ph2d_anim::Interp {
    ph2d_anim::Interp::Eased(ph2d_anim::Easing::new(
        ph2d_anim::EasingFamily::Cubic,
        ph2d_anim::EasingMode::InOut,
    ))
}

/// Map a "+Track" property-button id to its [`PropKind`] (the shell binds the
/// selected sprite's matching property). `None` for non-"+Track" ids.
pub(crate) fn prop_for_addprop_id(id: ph2d_editor::NodeId) -> Option<PropKind> {
    use ph2d_editor::ids as c;
    Some(match id {
        _ if id == c::TIMELINE_ADDPROP_TX => PropKind::TranslationX,
        _ if id == c::TIMELINE_ADDPROP_TY => PropKind::TranslationY,
        _ if id == c::TIMELINE_ADDPROP_ROT => PropKind::Rotation,
        _ if id == c::TIMELINE_ADDPROP_SX => PropKind::ScaleX,
        _ if id == c::TIMELINE_ADDPROP_SY => PropKind::ScaleY,
        _ if id == c::TIMELINE_ADDPROP_OPACITY => PropKind::Opacity,
        _ if id == c::TIMELINE_ADDPROP_TIME => PropKind::TimeRemap,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor::ids;

    #[test]
    fn transport_ids_map_to_intents() {
        let st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        ph.seek_frame(10, st.doc.fps_display);

        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_PLAY), &st, &ph),
            Some(TimelineIntent::TogglePlay)
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_NEXT_FRAME), &st, &ph),
            Some(TimelineIntent::SeekFrame(11))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_PREV_FRAME), &st, &ph),
            Some(TimelineIntent::SeekFrame(9))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 1.5), &st, &ph),
            Some(TimelineIntent::Scrub(1.5))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_AUTOKEY, true), &st, &ph),
            Some(TimelineIntent::SetAutoKey(true))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_SNAP, false), &st, &ph),
            Some(TimelineIntent::SetFrameSnap(false))
        );
        // A non-transport id (Close is handled in the panel, not translated).
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_CLOSE), &st, &ph),
            None
        );
    }

    #[test]
    fn go_start_and_go_end_scrub_to_the_clip_bounds() {
        use ph2d_timeline::PropKind;
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);

        // Empty doc: both ends are t = 0.
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_START), &st, &ph),
            Some(TimelineIntent::Scrub(0.0))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_END), &st, &ph),
            Some(TimelineIntent::Scrub(0.0))
        );

        // Key at 2.5 s: go-to-end must follow the LAST KEY, not the clip's
        // authored duration (0 on a fresh clip — a dead button otherwise).
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(2.5),
                value: ph2d_anim::AnimValue::Float(3.0),
                interp: default_interp(),
            },
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_END), &st, &ph),
            Some(TimelineIntent::Scrub(2.5))
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Click(ids::TIMELINE_GO_START), &st, &ph),
            Some(TimelineIntent::Scrub(0.0))
        );
    }

    #[test]
    fn the_default_loop_range_spans_to_the_last_key() {
        use ph2d_timeline::PropKind;
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(4.0),
                value: ph2d_anim::AnimValue::Float(1.0),
                interp: default_interp(),
            },
        );
        assert_eq!(
            intent_for_transport(&PanelEvent::Toggle(ids::TIMELINE_LOOP, true), &st, &ph),
            Some(TimelineIntent::SetLoop(Some((0.0, 4.0)))),
            "looping a hand-keyed clip must not collapse to a zero-length range"
        );
    }

    #[test]
    fn only_absolute_jumps_ask_the_panel_to_pan() {
        for ev in [
            PanelEvent::Click(ids::TIMELINE_GO_START),
            PanelEvent::Click(ids::TIMELINE_GO_END),
            PanelEvent::Click(ids::TIMELINE_PREV_FRAME),
            PanelEvent::Click(ids::TIMELINE_NEXT_FRAME),
            PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 9.0),
            PanelEvent::SetValue(ids::TIMELINE_FRAME_NUM, 200.0),
        ] {
            assert!(jumps_the_playhead(&ev), "{ev:?} lands at an absolute time");
        }
        // The ruler scrub maps a fraction of the VISIBLE span, so it can never
        // land off-screen — panning after it would fight the drag.
        assert!(!jumps_the_playhead(&PanelEvent::SetValue(
            ids::TIMELINE_RULER,
            0.5
        )));
        assert!(!jumps_the_playhead(&PanelEvent::Click(ids::TIMELINE_PLAY)));
        assert!(!jumps_the_playhead(&PanelEvent::Toggle(
            ids::TIMELINE_LOOP,
            true
        )));
    }

    #[test]
    fn sample_reads_transform_and_opacity() {
        use ph2d_anim::AnimValue;
        use ph2d_core::Vec2;
        use ph2d_ecs::{Transform, World};
        let mut w = World::new();
        let e = w
            .spawn(Transform::from_translation(Vec2::new(7.0, -3.0)))
            .id();
        assert_eq!(
            sample_prop_value(&w, e.to_bits(), PropKind::TranslationX),
            Some(AnimValue::Float(7.0))
        );
        assert_eq!(
            sample_prop_value(&w, e.to_bits(), PropKind::TranslationY),
            Some(AnimValue::Float(-3.0))
        );
        // No Sprite component → opacity sample is None (skipped, not a panic).
        assert_eq!(sample_prop_value(&w, e.to_bits(), PropKind::Opacity), None);
    }

    #[test]
    fn addprop_ids_map_to_prop_kinds() {
        use ph2d_timeline::PropKind;
        assert_eq!(
            prop_for_addprop_id(ids::TIMELINE_ADDPROP_TX),
            Some(PropKind::TranslationX)
        );
        assert_eq!(
            prop_for_addprop_id(ids::TIMELINE_ADDPROP_OPACITY),
            Some(PropKind::Opacity)
        );
        assert_eq!(
            prop_for_addprop_id(ids::TIMELINE_ADDPROP_TIME),
            Some(PropKind::TimeRemap)
        );
        assert_eq!(prop_for_addprop_id(ids::TIMELINE_PLAY), None);
    }

    #[test]
    fn k_keys_scene_props_at_the_entitys_remapped_clock() {
        use ph2d_anim::AnimValue::Float;
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        // No remap: identity (a scene key lands at the playhead).
        let at = |st: &TimelineState, prop, t: f64| key_insert_time(st, 1, prop, t).to_seconds();
        assert_eq!(at(&st, PropKind::TranslationX, 1.0), 1.0);
        // 2x remap (0 → 0, 2 → 4): a TX key at playhead 1 lands at SOURCE 2 —
        // where the apply samples it — while the Time track itself keys at the
        // playhead (the map lives in playhead time).
        for (t, v) in [(0.0, 0.0f32), (2.0, 4.0)] {
            ph2d_timeline::apply_intent(
                &mut st,
                &mut ph,
                TimelineIntent::AddKey {
                    entity: 1,
                    prop: PropKind::TimeRemap,
                    t: ph2d_anim::RationalTime::from_seconds(t),
                    value: Float(v),
                    interp: ph2d_anim::Interp::Linear,
                },
            );
        }
        assert_eq!(at(&st, PropKind::TranslationX, 1.0), 2.0, "source time");
        assert_eq!(at(&st, PropKind::TimeRemap, 1.0), 1.0, "playhead time");
    }

    #[test]
    fn k_seeds_a_time_remap_key_on_its_curve_or_at_the_identity() {
        use ph2d_anim::AnimValue::Float;
        use ph2d_ecs::World;
        let w = World::new();
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        // Empty Time track: K at t = 1.5 seeds the IDENTITY (source = playhead),
        // so the remap changes nothing until the author edits it.
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::Bind {
                entity: 1,
                prop: PropKind::TimeRemap,
            },
        );
        assert_eq!(
            key_value_for(&w, &st, 1, PropKind::TimeRemap, 1.5),
            Some(Float(1.5)),
            "empty Time track seeds the identity"
        );
        // With keys (0 → 0, 2 → 4), a K at t = 1 lands ON the curve (source 2)
        // — inserting it must not jump the retime the entity already plays.
        for (t, v) in [(0.0, 0.0f32), (2.0, 4.0)] {
            ph2d_timeline::apply_intent(
                &mut st,
                &mut ph,
                TimelineIntent::AddKey {
                    entity: 1,
                    prop: PropKind::TimeRemap,
                    t: ph2d_anim::RationalTime::from_seconds(t),
                    value: Float(v),
                    interp: ph2d_anim::Interp::Linear,
                },
            );
        }
        assert_eq!(
            key_value_for(&w, &st, 1, PropKind::TimeRemap, 1.0),
            Some(Float(2.0)),
            "a keyed Time track seeds ON its own curve"
        );
        // A scene prop still samples the world (a dead entity has none).
        assert_eq!(key_value_for(&w, &st, 1, PropKind::TranslationX, 1.0), None);
    }

    // The natural K flow — K at t=0, scrub, K again — must lay down an
    // identity-shaped remap, not a FLAT one that freezes every track of the
    // entity at source 0 ("Time nullifies the animation", 2026-07-11). Drives
    // the exact functions the shell's K handler uses.
    #[test]
    fn time_remap_double_k_must_not_freeze_position() {
        use ph2d_anim::AnimValue::Float;
        use ph2d_core::Vec2;
        use ph2d_ecs::{Transform, World};
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let eb = e.to_bits();
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        for (t, v) in [(0.0, 0.0f32), (4.0, 10.0)] {
            apply_intent(
                &mut st,
                &mut ph,
                TimelineIntent::AddKey {
                    entity: eb,
                    prop: PropKind::TranslationX,
                    t: ph2d_anim::RationalTime::from_seconds(t),
                    value: Float(v),
                    interp: ph2d_anim::Interp::Linear,
                },
            );
        }
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::Bind {
                entity: eb,
                prop: PropKind::TimeRemap,
            },
        );
        // Two K presses through the SAME functions the shell's K handler uses.
        for playhead_t in [0.0f64, 2.0] {
            let v = key_value_for(&w, &st, eb, PropKind::TimeRemap, playhead_t).unwrap();
            let t = key_insert_time(&st, eb, PropKind::TimeRemap, playhead_t);
            apply_intent(
                &mut st,
                &mut ph,
                TimelineIntent::AddKey {
                    entity: eb,
                    prop: PropKind::TimeRemap,
                    t,
                    value: v,
                    interp: default_interp(),
                },
            );
        }
        ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, 1.0);
        let x1 = w.get::<Transform>(e).unwrap().translation.x;
        assert!(
            (x1 - 2.5).abs() < 1e-4,
            "posição congelada: x@1 = {x1}, esperado 2.5"
        );
        // And past the seeded range the identity must keep playing too.
        ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, 3.0);
        let x3 = w.get::<Transform>(e).unwrap().translation.x;
        assert!(
            (x3 - 7.5).abs() < 1e-4,
            "posição congelada: x@3 = {x3}, esperado 7.5"
        );
    }

    // Hold's freeze-frame is DELIBERATE: past a Hold last key the entity plays
    // frozen, so a K there seeds the frozen clock (what the entity plays), not
    // a slope-1 continuation — seed and sampling stay the same transform.
    #[test]
    fn k_past_a_hold_freeze_seeds_the_frozen_clock() {
        use ph2d_anim::AnimValue::Float;
        use ph2d_ecs::World;
        let w = World::new();
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::Bind {
                entity: 1,
                prop: PropKind::TimeRemap,
            },
        );
        for (t, v, interp) in [
            (0.0, 0.0f32, ph2d_anim::Interp::Linear),
            (2.0, 4.0, ph2d_anim::Interp::Hold),
        ] {
            ph2d_timeline::apply_intent(
                &mut st,
                &mut ph,
                TimelineIntent::AddKey {
                    entity: 1,
                    prop: PropKind::TimeRemap,
                    t: ph2d_anim::RationalTime::from_seconds(t),
                    value: Float(v),
                    interp,
                },
            );
        }
        assert_eq!(
            key_value_for(&w, &st, 1, PropKind::TimeRemap, 3.0),
            Some(Float(4.0)),
            "K past a Hold freeze seeds the frozen source, matching the apply"
        );
        // Scene props key at the same frozen clock (where the apply samples).
        assert_eq!(
            key_insert_time(&st, 1, PropKind::TranslationX, 3.0).to_seconds(),
            4.0,
            "scene keys land at the frozen source time"
        );
    }
}

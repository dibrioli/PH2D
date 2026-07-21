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
    solo: bool,
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
    let skip = |bits: u64| live_entity == Some(bits) || displaced.contains(&bits);
    // **Keys mode solos the active clip** (`apply_active_clip`): you see and pose
    // exactly the curves you are editing, at the clip's own clock, with the stack
    // out of the way. Arrange blends the stack at the timeline clock. Both honour
    // `skip` (the gizmo-dragged entity, the displaced pin).
    if solo {
        ph2d_timeline::apply_active_clip(world, &mut timeline.doc, playhead.time(), skip);
    } else {
        apply_from_doc_except(world, &mut timeline.doc, playhead.time(), skip);
    }
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
    // **"The end" is the end of what THIS VIEW shows** (`TimelineDoc::view_end_seconds`):
    // the active clip on Keys, the last strip on Arrange. Both go-to-end and a freshly
    // armed loop read it, which is the point — they are the same question, and asking
    // the clip while looking at the stack bracketed one strip out of the set.
    //
    // Within a clip it is the last keyframe when that runs past the authored duration:
    // a fresh clip's duration is 0, which would pin both at t = 0 for every hand-keyed
    // animation.
    let duration = || timeline.doc.view_end_seconds(timeline.keys_mode);
    // **Inside a container, "the whole thing" is the walked-into INSTANCE** — its scene
    // window, leads included (`entry_reach`, the same door entering arms the loop
    // through). Go-to-start/end land on its edges, and the Loop/PingPong toggles bracket
    // it on the TRANSPORT only ([`TimelineIntent::SetTransportLoop`]): the document's
    // loop is the SCENE's, and a toggle flipped while visiting an instance must not
    // rewrite it (Enio, 2026-07-20). `None` at the root, or on a stale walk — then every
    // arm below falls back to the scene behaviour it always had.
    let reach = (!timeline.keys_mode && !timeline.edit_path.is_empty())
        .then(|| ph2d_timeline::entry_reach(&timeline.doc, &timeline.edit_path))
        .flatten();
    match *ev {
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY => Some(I::TogglePlay),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_START => {
            Some(I::Scrub(reach.map_or(0.0, |(lo, _)| lo)))
        }
        PanelEvent::Click(id) if id == ids::TIMELINE_ADD_MARKER => Some(I::AddMarker {
            t_seconds: playhead.time(),
            label: format!("M{}", timeline.doc.markers().len() + 1),
        }),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_END => {
            Some(I::Scrub(reach.map_or_else(duration, |(_, hi)| hi)))
        }
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
        // Loop and PingPong are ONE loop seen two ways — a range plus what happens
        // at its end. Each toggle sends the whole value, so arming one necessarily
        // disarms the other: there is no state where both are on, and no rule
        // anyone has to remember to enforce.
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_LOOP || id == ids::TIMELINE_PINGPONG => {
            let ping_pong = id == ids::TIMELINE_PINGPONG;
            // Inside a container the toggle brackets the INSTANCE, transport-only (see
            // `reach` above): on = its reach, off = no loop; leaving re-installs the
            // scene's own loop (`on_nav_change`).
            if let Some((lo, hi)) = reach {
                return Some(I::SetTransportLoop {
                    range: on.then_some((lo, hi)),
                    ping_pong: on && ping_pong,
                });
            }
            Some(if on {
                I::SetLoop {
                    range: Some((0.0, duration().max(1.0 / fps.max(1.0)))),
                    ping_pong,
                }
            } else {
                I::SetLoop {
                    range: None,
                    ping_pong: false,
                }
            })
        }
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_PHYSICS => {
            Some(I::SetSimulatePhysics(on))
        }
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_AUTOKEY => Some(I::SetAutoKey(on)),
        PanelEvent::Toggle(id, on) if id == ids::TIMELINE_RECORD => Some(I::SetPerforming(on)),
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
        // The morph `t`: unlike the clock below, this IS a scene value, so K captures it the same
        // way it captures a pose — the artist parks the slider where the shape looks right and
        // presses K. (No `VecMorph` on the entity ⇒ nothing to capture, and the `?` refuses.)
        PropKind::Morph => Float(world.get::<ph2d_ecs::VecMorph>(e)?.t),
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
    // The live pose is what the animator SEES. Under a clip stack it is a blend,
    // so the number the track must hold to produce it is the blend's inverse —
    // never the pose itself, which would land the object somewhere else the moment
    // the stack re-evaluated. `None` = the active clip cannot express this pose,
    // and K refuses (ADR-0115 R9).
    let ph2d_anim::AnimValue::Float(pose) = sample_prop_value(world, entity, prop)? else {
        return None;
    };
    let stored = ph2d_timeline::key_value_in_active_clip(&timeline.doc, entity, prop, pose)?;
    Some(ph2d_anim::AnimValue::Float(stored))
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
) -> Option<ph2d_anim::RationalTime> {
    let t = if prop == PropKind::TimeRemap {
        t_secs
    } else {
        // `key_time`, not `remapped_time`: under a clip stack the strip's map
        // composes on top of the entity's clock, and it can have NO answer (the
        // clip is playing twice at this instant, or not at all). Refuse rather
        // than drop the key at a time the animator never looked at.
        ph2d_timeline::key_time(&timeline.doc, entity, t_secs)?
    };
    Some(ph2d_anim::RationalTime::from_seconds(t))
}

/// **K in the Keys/solo view** — the `(value, time)` a key captures when the scene
/// is showing the active clip soloed at clip time `clip_t`.
///
/// It never REFUSES (returns `Some` whenever the pose can be read): in solo there is
/// no stack to override the clip or to play it twice, so "key it here" always names
/// one place. That is the whole point of editing a clip in isolation.
///
/// The VALUE is the live pose stored DIRECTLY — with the clip soloed, the pose you
/// see IS the clip's value (no blend to invert). The TIME is the entity's own clip
/// clock (`remapped_time`, the active clip's Time Remap), so a Time-Remapped object
/// keys where its retime puts it. Both come through the SAME door the solo apply and
/// the Arrange-side K read, so a soloed pose and the key that captures it agree.
///
/// A Time Remap track keys ON its own curve (the retime value at `clip_t`), exactly
/// as the Arrange-side K does — that half is clock-agnostic.
pub(crate) fn key_authoring_solo(
    world: &World,
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    clip_t: f64,
) -> Option<(ph2d_anim::AnimValue, ph2d_anim::RationalTime)> {
    let doc = &timeline.doc;
    if prop == PropKind::TimeRemap {
        let source = ph2d_timeline::remapped_time(doc, entity, clip_t);
        return Some((
            ph2d_anim::AnimValue::Float(source as f32),
            ph2d_anim::RationalTime::from_seconds(clip_t),
        ));
    }
    let value = sample_prop_value(world, entity, prop)?; // the raw pose = what you see
    let t = ph2d_timeline::remapped_time(doc, entity, clip_t);
    Some((value, ph2d_anim::RationalTime::from_seconds(t)))
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

/// **What entering (or leaving) a container does to the transport loop** — called on the
/// navigation EDGE only, from the shell's per-frame `edit_path` stamp (Enio, 2026-07-20:
/// *"dentro do container o loop não se ajustou automaticamente ao tempo das lanes"*).
///
/// Inside a container the thing being authored is the INSTANCE, so play cycles over the
/// timeline window that instance occupies — leads included ([`ph2d_timeline::entry_reach`]:
/// bracketing only the moving window would cut the very fades the artist is tuning, the
/// `stack_end_seconds` bug one level down). A playhead standing outside that window is
/// seeked to its start: the alternative is a marker-less ruler that reads as broken.
///
/// Leaving (path empty) hands the transport back to the DOCUMENT's Arrange loop via the
/// same [`ph2d_timeline::sync_transport_loop`] a tab switch uses — navigation is not an
/// edit, so nothing here writes the document; the authored loop is merely re-installed.
///
/// Edge-triggered, not stamped every frame, so the artist can still re-arm or clear the
/// loop while inside — their gesture wins over the convenience.
pub(crate) fn on_nav_change(
    doc: &ph2d_timeline::TimelineDoc,
    path: &[ph2d_timeline::EnterStep],
    playhead: &mut ph2d_core::Playhead,
) {
    if path.is_empty() {
        ph2d_timeline::sync_transport_loop(doc, playhead, false);
        return;
    }
    let Some((lo, hi)) = ph2d_timeline::entry_reach(doc, path) else {
        return; // a stale walk has no window; leave the transport as it stands
    };
    playhead.set_loop(lo, hi);
    playhead.set_loop_mode(ph2d_core::LoopMode::Wrap);
    if playhead.time() < lo || playhead.time() > hi {
        playhead.seek(lo);
    }
}

#[cfg(test)]
#[path = "timeline_bridge_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "timeline_bridge_k_tests.rs"]
mod k_tests;

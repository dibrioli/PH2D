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
///
/// Returns `true` when the identity upkeep RESET the document (the last animated
/// object was deleted — `timeline_persist::upkeep`); the caller rewinds the
/// clocks and drops the panel's container trail, which this function cannot do
/// because it only holds ONE of the two playheads.
// The frame's inputs, each load-bearing (the world, the two-way state, the active
// clock, the drag/displaced skips, and the two view discriminants `solo`/`container`).
// Bundling them into a struct would hide which the caller must supply per frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    world: &mut World,
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    intents: &mut Vec<TimelineIntent>,
    live_entity: Option<u64>,
    ak: &mut super::autokey_pass::AutokeyState,
    solo: bool,
    container: Option<usize>,
) -> bool {
    // **The Containers list has no playback mode** (Enio, 2026-07-22). The two
    // refusal layers upstream (the play button paints dead + unhittable; the
    // TogglePlay intent maps to None) stop the GESTURES — this is the backstop
    // for the clock itself: switching to the list while playing, or any path
    // that reaches the playhead without asking the panel (spacebar), lands here
    // and the clock pauses. In the list `keys_mode` is false, so `playhead` IS
    // the scene clock the artist would otherwise watch run with dead controls.
    if timeline.containers_list && playhead.is_playing() {
        playhead.pause();
    }
    for intent in intents.drain(..) {
        apply_intent(timeline, playhead, intent);
    }
    // **An AUTHORED duration is a hard end** (Enio, 2026-07-23): the playhead never
    // leaves `[0, end]` on a view whose duration is explicit. Every road in — scrub,
    // typed time, frame step, play — passes through here once per frame, so this is
    // the one door; play that reaches the end parks there and PAUSES (the AE comp
    // end). A view with no authored Dur keeps today's open-ended run — which is what
    // keeps a physics scene's Play (an empty timeline driving a sim) alive.
    //
    // Through `view_authored_end` — the SAME door the veil asks (`snapshot`), so the
    // darkened dead zone and the wall the playhead hits are the same fact. `solo` is
    // `keys_mode`, and without a stack that is FALSE even on the Keys tab; the door
    // handles it (a clip's `length_override` closes the no-stack view). Reading
    // `scene_length` here directly missed exactly that — a clip Dur that darkened
    // nothing and pinned nothing (Enio, 2026-07-23).
    let authored_end = timeline.doc.view_authored_end(container, solo);
    if let Some(end) = authored_end
        && playhead.time() > end
    {
        if playhead.is_playing() {
            playhead.pause();
        }
        playhead.seek(end);
    }
    // A playhead move (scrub, play, frame step) reclaims every displaced pose
    // for the animation — Blender semantics: the un-keyed pose is discarded.
    if playhead.time() != ak.displaced_t {
        ak.displaced.clear();
        ak.displaced_t = playhead.time();
    }
    let displaced = &ak.displaced;
    let skip = |bits: u64| live_entity == Some(bits) || displaced.contains(&bits);
    // **Three clocks, three views** (Enio, 2026-07-16 / 2026-07-22):
    // - Keys solos the active CLIP at its own clock (`apply_active_clip`) — pose the
    //   exact curves you edit, stack out of the way.
    // - Inside a container, solo the CONTAINER's interior at ITS clock
    //   (`apply_container`) — the playback the animator is editing IS the container's.
    // - Arrange blends the scene stack at the timeline clock.
    // All honour `skip` (the gizmo-dragged entity, the displaced pin).
    if let Some(c) = container {
        ph2d_timeline::apply_container(world, &mut timeline.doc, c, playhead.time(), skip);
    } else if solo {
        ph2d_timeline::apply_active_clip(world, &mut timeline.doc, playhead.time(), skip);
    } else {
        apply_from_doc_except(world, &mut timeline.doc, playhead.time(), skip);
    }
    // Identity upkeep: heal (a project load's detached bindings recolam pelo
    // nome), then purge — a deleted object's tracks leave the document with it,
    // and deleting the LAST animated object resets the timeline whole
    // (`timeline_persist::upkeep`).
    crate::timeline_persist::upkeep(timeline, world)
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
    // **Inside a container the transport is the CONTAINER's own clock** (Enio, 2026-07-22:
    // *"o playback deve ser relativo ao container aberto em edição"*), so every command
    // here speaks the container's LOCAL time: go-to-start is 0, go-to-end is its length,
    // and Loop/PingPong write the CONTAINER's own loop ([`TimelineIntent::SetContainerLoop`],
    // persisted on the `NamedContainer`) — independent of the scene's and of every clip's.
    // `None` on Keys / at the scene root, where the arms below keep their old behaviour.
    let container: Option<usize> = (!timeline.keys_mode)
        .then(|| timeline.edit_path.last().map(|s| s.container))
        .flatten();
    let container_len = |c: usize| timeline.doc.container_length_seconds(c);
    match *ev {
        // No playback mode on the Containers LIST (Enio, 2026-07-22): the panel
        // already paints the button dead and unhittable — this layer keeps a
        // synthetic/stale click from starting a clock the view says cannot run
        // ([[feedback_layered_defenses_need_per_layer_gates]]).
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY && timeline.containers_list => None,
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY => Some(I::TogglePlay),
        // Go-to-start is 0 in every mode — the local start of whatever clock the shell
        // hands us (scene, clip, or container). (In container mode the clock is the
        // container's own, so 0 is its interior start, not the scene's.)
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_START => Some(I::Scrub(0.0)),
        PanelEvent::Click(id) if id == ids::TIMELINE_ADD_MARKER => Some(I::AddMarker {
            t_seconds: playhead.time(),
            label: format!("M{}", timeline.doc.markers().len() + 1),
        }),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_END => {
            Some(I::Scrub(container.map_or_else(duration, container_len)))
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
            // Inside a container the toggle writes the CONTAINER's OWN loop, over its
            // local `[0, length)` — persisted on the `NamedContainer`, independent of
            // the scene's and every clip's (Enio, 2026-07-22). The leak this replaces
            // was a container toggle reaching the SCENE's loop.
            if let Some(c) = container {
                let len = container_len(c).max(1.0 / fps.max(1.0));
                return Some(I::SetContainerLoop {
                    container: c,
                    range: on.then_some((0.0, len)),
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

/// **Selecionar um objeto NOVO leva a timeline à aba Keys** (Enio, 2026-07-22) — a
/// decisão de BORDA: dispara quando a seleção primária muda para um objeto (qualquer
/// um), e só então. Deselecionar não puxa aba nenhuma, e re-observar a MESMA seleção
/// frame após frame também não — senão o animador nunca conseguiria ficar em
/// Containers/Arrange com um objeto selecionado.
pub(crate) fn selection_jumps_to_keys(prev: Option<u64>, now: Option<u64>) -> bool {
    now.is_some() && now != prev
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
        // Position is not a scalar this function can sample, and that is the
        // channel's shape rather than a gap: capturing one means ADDING AN ANCHOR at
        // the object's current place, which moves the geometry and therefore rewrites
        // the distance every later key holds (ADR-0141 §2). That is an edit to a
        // trajectory, not a read of a number, so it goes through the path's own door
        // — which is what the authoring slice builds. Refusing here keeps this
        // function honest about what it is.
        PropKind::Position => return None,
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
    // The clip's authored duration cuts the solo clock BEFORE the remap — the
    // same composition `apply_active_clip` runs. Past the cut the pose you see
    // is frozen at `curve(cut)`, so K captures it AT the boundary (the visible
    // frame); keying at the raw time would land at an instant the apply never
    // samples, and the pose would snap back (seed == sample, 2026-07-23).
    let clip_t = doc.clip_cut(doc.active_index(), clip_t);
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
/// moved to its start: the alternative is a marker-less ruler that reads as broken.
///
/// Leaving (path empty) hands the transport back to the DOCUMENT's Arrange loop via the
/// same [`ph2d_timeline::sync_transport_loop`] a tab switch uses — navigation is not an
/// edit, so nothing here writes the document; the authored loop is merely re-installed.
///
/// Edge-triggered, not stamped every frame, so the artist can still re-arm or clear the
/// loop while inside — their gesture wins over the convenience.
/// **Entering / switching a container arms the CONTAINER clock with the container's
/// OWN loop** (Enio, 2026-07-22) — `container` is the innermost the animator has walked
/// into (`None` when they left back to the scene). It touches ONLY `container_ph`.
///
/// This replaces the old scene-side bracketing: entering a container no longer reaches
/// the scene playhead at all, so the Arrange loop the artist authored survives every
/// visit untouched. Leaving (`None`) does nothing — there is nothing on the scene clock
/// to restore, because nothing on it was ever moved.
pub(crate) fn on_container_nav_change(
    doc: &ph2d_timeline::TimelineDoc,
    container: Option<usize>,
    container_ph: &mut ph2d_core::Playhead,
) {
    let Some(c) = container else {
        return; // left to the scene: the scene clock was never ours to touch
    };
    ph2d_timeline::sync_container_loop(doc, c, container_ph);
    // Land inside the loop if one is set and the clock is parked outside it — the same
    // courtesy the scene-side bracketing used to do, now on the container's own clock.
    if let (Some((lo, hi)), _) = doc.container_loop(c)
        && (container_ph.time() < lo || container_ph.time() > hi)
    {
        container_ph.seek(lo);
    }
}

#[cfg(test)]
#[path = "timeline_bridge_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "timeline_bridge_k_tests.rs"]
mod k_tests;

#[cfg(test)]
#[path = "timeline_bridge_container_tests.rs"]
mod container_tests;

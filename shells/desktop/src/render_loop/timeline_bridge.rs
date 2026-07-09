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
//! so the KeyB `SpriteAnimation` demo path is unaffected.

use ph2d_core::Playhead;
use ph2d_ecs::World;
use ph2d_editor::tool::PanelEvent;
use ph2d_timeline::{PropKind, TimelineIntent, TimelineState, apply_from_doc_except, apply_intent};

/// Drain pending intents into `timeline`, then apply its document to `world` at
/// the current `playhead` time. `live_entity` (the entity whose gizmo is being
/// dragged this frame, if any) is left untouched by the apply so the document
/// does not fight the live manipulation. Call each frame in the apply pass,
/// after `apply_sprite_animations`.
pub(crate) fn run(
    world: &mut World,
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    intents: &mut Vec<TimelineIntent>,
    live_entity: Option<u64>,
) {
    for intent in intents.drain(..) {
        apply_intent(timeline, playhead, intent);
    }
    apply_from_doc_except(world, &mut timeline.doc, playhead.time(), live_entity);
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
    let duration = || timeline.doc.active_clip().duration().to_seconds();
    match *ev {
        PanelEvent::Click(id) if id == ids::TIMELINE_PLAY => Some(I::TogglePlay),
        PanelEvent::Click(id) if id == ids::TIMELINE_GO_START => Some(I::Scrub(0.0)),
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
    })
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
        assert_eq!(prop_for_addprop_id(ids::TIMELINE_PLAY), None);
    }
}

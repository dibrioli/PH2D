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
use ph2d_timeline::{TimelineIntent, TimelineState, apply_from_doc, apply_intent};

/// Drain pending intents into `timeline`, then apply its document to `world` at
/// the current `playhead` time. Call each frame in the apply pass, after
/// `apply_sprite_animations`.
pub(crate) fn run(
    world: &mut World,
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    intents: &mut Vec<TimelineIntent>,
) {
    for intent in intents.drain(..) {
        apply_intent(timeline, playhead, intent);
    }
    apply_from_doc(world, &mut timeline.doc, playhead.time());
}

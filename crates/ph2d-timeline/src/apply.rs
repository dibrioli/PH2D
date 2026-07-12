//! [`apply_from_doc`] — the document-driven apply pass: sample every binding's
//! track at the playhead and write the resolved property into its entity.
//!
//! This is the general counterpart to [`crate::apply_sprite_animations`] (the
//! per-component path, kept for programmatic use). It resolves each
//! [`TargetBinding`] through the sprite convention: the five `Transform`
//! properties natively (via `ph2d-ecs`), and `Opacity` into `Sprite.tint[3]`
//! behind the optional `render` feature (so the base crate stays free of the
//! GPU dependency). A binding whose entity is dead is flagged `missing` and
//! skipped — never a silent no-op (P6).

use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_ecs::{Entity, Transform, World};

use crate::doc::TimelineDoc;
use crate::prop::PropKind;
use crate::sprite::SpriteProp;
use crate::stack_eval;

/// Sample every binding in `doc`'s active clip at time `t` (seconds) and write
/// the resolved value into each bound entity. Updates each binding's `missing`
/// flag by liveness. Call once per frame after advancing the Playhead.
pub fn apply_from_doc(world: &mut World, doc: &mut TimelineDoc, t: f64) {
    apply_from_doc_except(world, doc, t, |_| false);
}

/// Like [`apply_from_doc`], but leaves every entity whose bits `skip` claims
/// untouched — the caller owns those transforms this frame: the one under a
/// live gizmo drag, and any pose the user displaced while paused that is
/// waiting for a manual K (the displaced-pose pin). Their `missing` flags are
/// still refreshed; only the write is skipped. With auto-key armed the drag
/// records keys, so once the gesture ends `apply_from_doc` resumes on the newly
/// recorded pose and the entity holds.
pub fn apply_from_doc_except(
    world: &mut World,
    doc: &mut TimelineDoc,
    t: f64,
    skip: impl Fn(u64) -> bool,
) {
    // Pass 1 — liveness (P6), and the one chance to capture `rest`.
    //
    // It precedes the clock pass on purpose: a dead entity's Time Remap track
    // must not drive anything, and resolving clocks against last frame's flags
    // would let it. And `rest` is read HERE, before any write this frame — the
    // world still holds the pose the animator left the object in.
    let n = doc.bindings().len();
    for i in 0..n {
        let (entity_bits, prop) = {
            let b = &doc.bindings()[i];
            (b.entity, b.prop)
        };
        let entity = Entity::from_bits(entity_bits);
        let alive = world.get_entity(entity).is_ok();
        doc.bindings_mut()[i].missing = !alive;
        if alive && doc.bindings()[i].rest.is_none() && prop != PropKind::TimeRemap {
            doc.bindings_mut()[i].rest = read_prop(world, entity, prop);
        }
    }

    // Pass 2 — the live strips and, inside each, one clock per remapped ENTITY.
    // Never one per binding: that was the quadratic (`clock.rs`).
    let mut scratch = doc.take_scratch();
    scratch.rebuild(doc, t);
    let stacked = !doc.stack().is_empty();

    // Pass 3 — write. Reads the document, writes the world: no `&mut doc` here,
    // so the binding list is a plain slice.
    for b in doc.bindings() {
        if b.missing {
            continue;
        }
        // The user owns this entity's transform this frame (gizmo drag /
        // displaced-pose pin) — don't clobber it from the document.
        if skip(b.entity) {
            continue;
        }
        // Time Remap is the timeline's own meta-property, never a scene write:
        // it was CONSUMED above as the entity's sampling clock.
        if b.prop == PropKind::TimeRemap {
            continue;
        }
        // THE branch (ADR-0115 §6): an empty stack is the single-clip path, on
        // the same code it always ran. A stack blends its lanes instead.
        let sampled = if stacked {
            stack_eval::sample_stack(
                doc,
                &scratch,
                stack_eval::Query {
                    entity: b.entity,
                    target: b.target,
                    prop: b.prop,
                    rest: b.rest.unwrap_or(0.0),
                },
            )
            .map(AnimValue::Float)
        } else {
            let t_entity = stack_eval::solo_source_time(&scratch, b.entity, t);
            // Skip empty tracks so a just-created binding never forces the
            // property to a default value.
            doc.active_clip().track(b.target).and_then(|tr| {
                if tr.is_empty() {
                    None
                } else {
                    Some(tr.sample(t_entity))
                }
            })
        };
        if let Some(v) = sampled {
            write_prop(world, Entity::from_bits(b.entity), b.prop, v);
        }
    }

    doc.put_scratch(scratch); // capacity retained — zero-alloc next frame (HR-3)
}

/// Read one property back out of an entity — the exact inverse of
/// [`write_prop`], and the reason `rest` can be captured without the shell's
/// help. Also what the inverse-blend key authoring will need (ADR-0115 R9).
fn read_prop(world: &World, entity: Entity, prop: PropKind) -> Option<f32> {
    if let Some(sp) = prop.as_sprite_transform() {
        let xf = world.get::<Transform>(entity)?;
        return Some(match sp {
            SpriteProp::TranslationX => xf.translation.x,
            SpriteProp::TranslationY => xf.translation.y,
            SpriteProp::Rotation => xf.rotation,
            SpriteProp::ScaleX => xf.scale.x,
            SpriteProp::ScaleY => xf.scale.y,
        });
    }
    #[cfg(feature = "render")]
    if prop == PropKind::Opacity {
        return world
            .get::<ph2d_render::Sprite>(entity)
            .map(|sprite| sprite.tint[3]);
    }
    let _ = (world, entity);
    None
}

/// The time `entity`'s tracks sample at: its Time Remap track's value at the
/// playhead — the AE model (slope < 1 slow motion, flat freeze, negative
/// reverse) — or the playhead itself when it has none / an empty track (the
/// identity: binding "Time" changes nothing until keys are authored).
///
/// The **single-entity** path, for key authoring outside the frame loop
/// (auto-key's diff, the displaced-pose pin, `K`). The apply resolves the whole
/// document at once through [`crate::clock::ClockIndex`] — every caller here
/// asks about one entity, so a scan is the right shape.
///
/// Seeding and sampling MUST come through this same function: a derived
/// coordinate whose author uses a different transform than its reader is the bug
/// that cost this module three rounds.
///
/// Zero-alloc (HR-3, the paused bridge path is gated).
pub fn remapped_time(doc: &TimelineDoc, entity: u64, t: f64) -> f64 {
    crate::clock::remapped_time_in(doc.active_clip(), doc.bindings(), entity, t)
}

/// **Where a key authored right now lands**, in the active clip's own time — or
/// `None` when there is no single answer and the key must be refused.
///
/// A track is authored in the clip's time, and the playhead runs in the
/// timeline's. Without a stack the only gap between them is the entity's Time
/// Remap, and [`remapped_time`] closes it. With a stack there is a second map on
/// top (the strip's), and it can fail to be a function: if the clip you are
/// editing is playing **twice** at this instant, "here" names two places in it,
/// and choosing one silently would drop the key somewhere the animator never
/// looked. If it is playing **zero** times, "here" names none.
///
/// Seeding and sampling go through the same composition — the strip's map, then
/// the clip's own clock — because a derived coordinate written by one transform
/// and read by another is the bug that has broken this module three times over.
#[must_use]
pub fn key_time(doc: &TimelineDoc, entity: u64, t: f64) -> Option<f64> {
    key_home(doc, entity, t).ok()
}

/// [`key_time`], with the **reason** when there is no answer.
///
/// The refusal is the whole point (ADR-0115 R9), and a refusal nobody can see is
/// indistinguishable from a bug: the animator drags, the object snaps back, and
/// nothing says why. The evaluator is the only place that knows which of the three
/// things went wrong, so it is the place that names it — the shell only speaks it.
///
/// `key_time` is this, thrown away: every caller that does not surface the reason
/// says so by calling that one instead.
pub fn key_home(doc: &TimelineDoc, entity: u64, t: f64) -> Result<f64, crate::KeyRefusal> {
    if doc.stack().is_empty() {
        return Ok(remapped_time(doc, entity, t));
    }
    let scratch = doc.scratch();
    let strip = stack_eval::sole_strip_of(scratch, doc.active_index())?;
    // The strip is live (`sole_strip_of` said so), so its clock has an entry —
    // `NotPlaying` here would mean the scratch disagreed with itself.
    stack_eval::strip_source_time(scratch, &strip, entity).ok_or(crate::KeyRefusal::NotPlaying)
}

/// Write one resolved property value into an entity, via the sprite resolver.
fn write_prop(world: &mut World, entity: Entity, prop: PropKind, v: AnimValue) {
    if let Some(sp) = prop.as_sprite_transform() {
        let AnimValue::Float(f) = v else { return };
        if let Some(mut xf) = world.get_mut::<Transform>(entity) {
            match sp {
                SpriteProp::TranslationX => xf.translation.x = f,
                SpriteProp::TranslationY => xf.translation.y = f,
                SpriteProp::Rotation => xf.rotation = f,
                SpriteProp::ScaleX => xf.scale.x = f,
                SpriteProp::ScaleY => xf.scale.y = f,
            }
        }
        return;
    }
    // Non-Transform properties. `Opacity` needs the render crate (Sprite lives
    // there); gated so the base timeline runtime stays GPU-free.
    #[cfg(feature = "render")]
    if prop == PropKind::Opacity
        && let AnimValue::Float(f) = v
        && let Some(mut sprite) = world.get_mut::<ph2d_render::Sprite>(entity)
    {
        sprite.tint[3] = f.clamp(0.0, 1.0);
    }
    #[cfg(not(feature = "render"))]
    let _ = (world, entity, prop);
}

#[cfg(test)]
mod time_remap_tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_core::Vec2;

    /// A world with one sprite entity whose TranslationX is keyed 0 → 10 over
    /// `0..4 s` (linear: `x(t_source) = 2.5·t_source`).
    fn rig() -> (World, Entity, TimelineDoc) {
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        for (t, v) in [(0.0, 0.0), (4.0, 10.0)] {
            doc.insert_key(
                e.to_bits(),
                PropKind::TranslationX,
                s(t),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
        (w, e, doc)
    }

    fn x_at(w: &mut World, e: Entity, doc: &mut TimelineDoc, t: f64) -> f32 {
        apply_from_doc(w, doc, t);
        w.get::<Transform>(e).unwrap().translation.x
    }

    fn remap_key(doc: &mut TimelineDoc, e: Entity, t: f64, source: f32, interp: Interp) {
        doc.insert_key(
            e.to_bits(),
            PropKind::TimeRemap,
            RationalTime::from_seconds(t),
            AnimValue::Float(source),
            interp,
        );
    }

    #[test]
    fn a_remap_curve_retimes_every_other_track_of_its_entity() {
        // Remap (0 → 0, 2 → 4): the playhead covers the 4 s animation in 2 s —
        // double speed. At playhead 1 the source time is 2 → x = 5.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 2.0, 4.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 5.0, "2x speed");
        assert_eq!(
            x_at(&mut w, e, &mut doc, 2.0),
            10.0,
            "done in half the time"
        );
    }

    #[test]
    fn a_flat_remap_freezes_and_a_falling_one_reverses() {
        let (mut w, e, mut doc) = rig();
        // Freeze: a single key holds source time at 2 s → x pinned at 5.
        remap_key(&mut doc, e, 0.0, 2.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 5.0);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 5.0, "frozen while t moves");
        // Reverse: remap (0 → 4, 4 → 0) plays the animation backwards.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 4.0, Interp::Linear);
        remap_key(&mut doc, e, 4.0, 0.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 10.0, "starts at the end");
        assert_eq!(x_at(&mut w, e, &mut doc, 4.0), 0.0, "ends at the start");
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 7.5, "backwards through 3 s");
    }

    #[test]
    fn a_single_seeded_time_key_keeps_the_identity_clock() {
        // THE "Time bugado" case: K seeds one identity key (a non-Hold interp).
        // The track's flat-hold would freeze the clock at that one value for
        // every playhead time — the sprite snapped back and no further pose
        // could be authored. Outside the keyed range the clock must extrapolate
        // at slope 1: one seeded key behaves exactly like zero keys.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 1.0, 1.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 2.5, "on the key: identity");
        assert_eq!(
            x_at(&mut w, e, &mut doc, 3.0),
            7.5,
            "after the key the clock keeps advancing (x(3) = 7.5, not frozen at 2.5)"
        );
        assert_eq!(
            x_at(&mut w, e, &mut doc, 0.0),
            0.0,
            "before the key too (x(0) = 0, not held at the key's value)"
        );
    }

    #[test]
    fn extrapolation_continues_a_ramp_at_normal_speed_but_a_hold_last_key_freezes() {
        // 2x ramp (0 → 0, 1 → 2): past the last key the clock resumes at
        // slope 1 from where the ramp left it — source 2 + (t − 1).
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 1.0, 2.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 10.0, "source 4 at t = 3");
        // The same ramp with a HOLD last key freezes from there on (the
        // deliberate AE freeze-frame survives the extrapolation rule).
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 1.0, 2.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 5.0, "held at source 2");
        // Extrapolating backwards below zero clamps like every playhead time.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 2.0, 0.5, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 0.0, "clamped at source 0");
    }

    #[test]
    fn an_empty_or_missing_remap_track_is_the_identity() {
        // Bind "Time" with no keys: nothing changes until the user authors.
        let (mut w, e, mut doc) = rig();
        doc.bind(e.to_bits(), PropKind::TimeRemap);
        assert_eq!(x_at(&mut w, e, &mut doc, 2.0), 5.0, "identity: x(2) = 5");
        // And another entity's remap never leaks onto this one.
        let stranger = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        remap_key(&mut doc, stranger, 0.0, 99.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 2.0), 5.0, "per-entity clock");
    }

    #[test]
    fn the_remap_track_itself_never_writes_a_scene_property() {
        // A Time key whose VALUE is huge must not bleed into any Transform
        // field — the remap binding is consumed as a clock, never written.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        let before = *w.get::<Transform>(e).unwrap();
        apply_from_doc(&mut w, &mut doc, 0.0);
        let after = *w.get::<Transform>(e).unwrap();
        assert_eq!(before.translation.y, after.translation.y);
        assert_eq!(before.rotation, after.rotation);
        assert_eq!(before.scale, after.scale);
        assert_eq!(
            after.translation.x, 0.0,
            "x follows its own track at source 0"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_core::Vec2;

    /// The live-dragged entity keeps its manipulated transform; every other
    /// bound entity is still driven from the document.
    #[test]
    fn live_entity_is_not_clobbered_by_the_document() {
        let mut w = World::new();
        let dragged = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let other = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        // Both entities have a TranslationX key pinning them to 5.0 at t=0.
        for e in [dragged, other] {
            doc.insert_key(
                e.to_bits(),
                PropKind::TranslationX,
                s(0.0),
                AnimValue::Float(5.0),
                Interp::Hold,
            );
        }
        // Simulate a gizmo drag having just written the dragged entity to 42.
        w.get_mut::<Transform>(dragged).unwrap().translation.x = 42.0;

        let dragged_bits = dragged.to_bits();
        apply_from_doc_except(&mut w, &mut doc, 0.0, |bits| bits == dragged_bits);

        assert_eq!(
            w.get::<Transform>(dragged).unwrap().translation.x,
            42.0,
            "the live-dragged entity must keep its manipulated pose (document must not fight it)"
        );
        assert_eq!(
            w.get::<Transform>(other).unwrap().translation.x,
            5.0,
            "every other bound entity is still driven from the document"
        );

        // With nothing skipped, the document reclaims the dragged one too.
        apply_from_doc_except(&mut w, &mut doc, 0.0, |_| false);
        assert_eq!(w.get::<Transform>(dragged).unwrap().translation.x, 5.0);
    }
}

#[cfg(all(test, feature = "render"))]
mod render_tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_render::Sprite;

    #[test]
    fn opacity_drives_sprite_tint_alpha() {
        let mut w = World::new();
        let e = w
            .spawn(Sprite::atlas(0, [10.0, 10.0], [1.0, 1.0, 1.0, 1.0]))
            .id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        doc.insert_key(
            e.to_bits(),
            PropKind::Opacity,
            s(0.0),
            AnimValue::Float(1.0),
            Interp::Linear,
        );
        doc.insert_key(
            e.to_bits(),
            PropKind::Opacity,
            s(2.0),
            AnimValue::Float(0.0),
            Interp::Hold,
        );

        apply_from_doc(&mut w, &mut doc, 1.0); // midpoint → alpha 0.5
        let a = w.get::<Sprite>(e).unwrap().tint[3];
        assert!(
            (a - 0.5).abs() < 1e-5,
            "opacity animated tint alpha to 0.5, got {a}"
        );
    }
}

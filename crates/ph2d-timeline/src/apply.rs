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
    let n = doc.bindings().len();
    for i in 0..n {
        let b = &doc.bindings()[i];
        let (target, prop, entity_bits) = (b.target, b.prop, b.entity);
        let entity = Entity::from_bits(entity_bits);
        let alive = world.get_entity(entity).is_ok();
        doc.bindings_mut()[i].missing = !alive;
        if !alive {
            continue;
        }
        // The user owns this entity's transform this frame (gizmo drag /
        // displaced-pose pin) — don't clobber it from the document.
        if skip(entity_bits) {
            continue;
        }
        // Time Remap is the timeline's own meta-property, never a scene write:
        // it is CONSUMED below as the entity's sampling clock.
        if prop == PropKind::TimeRemap {
            continue;
        }
        let t_entity = remapped_time(doc, entity_bits, t);
        // Sample the bound track (skip empty tracks so a just-created binding
        // never forces the property to a default value).
        let sampled = doc.active_clip().track(target).and_then(|tr| {
            if tr.is_empty() {
                None
            } else {
                Some(tr.sample(t_entity))
            }
        });
        if let Some(v) = sampled {
            write_prop(world, entity, prop, v);
        }
    }
}

/// The time `entity`'s tracks sample at: its Time Remap track's value at the
/// playhead — the AE model (slope < 1 slow motion, flat freeze, negative
/// reverse) — or the playhead itself when it has none / an empty track (the
/// identity: binding "Time" changes nothing until keys are authored).
///
/// **Outside the keyed range the clock extrapolates at slope 1** (identity
/// offset through the boundary key), instead of the track's flat-hold: a
/// track's hold would freeze the entity's whole timeline the moment the FIRST
/// key lands — K seeds the identity, so one seeded key must change nothing
/// anywhere, exactly like zero keys. The one exception is a **`Hold` last
/// key**, which freezes from there on (the deliberate AE freeze-frame).
///
/// Clamped non-negative like every playhead time. A linear scan over the few
/// bindings: zero-alloc (HR-3, the paused bridge path is gated).
pub fn remapped_time(doc: &TimelineDoc, entity: u64, t: f64) -> f64 {
    let as_f64 = |v: AnimValue| match v {
        AnimValue::Float(f) => Some(f64::from(f)),
        _ => None,
    };
    for b in doc.bindings() {
        if b.entity != entity || b.prop != PropKind::TimeRemap || b.missing {
            continue;
        }
        let Some(tr) = doc.active_clip().track(b.target) else {
            continue;
        };
        let (Some(first), Some(last)) = (tr.keys().first(), tr.keys().last()) else {
            continue; // empty track = identity
        };
        let (t0, t1) = (first.t.to_seconds(), last.t.to_seconds());
        let source = if t < t0 {
            as_f64(first.value).map(|v| v + (t - t0))
        } else if t > t1 {
            as_f64(last.value).map(|v| match last.interp {
                ph2d_anim::Interp::Hold => v,
                _ => v + (t - t1),
            })
        } else {
            as_f64(tr.sample(t))
        };
        if let Some(v) = source {
            return v.max(0.0);
        }
    }
    t
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

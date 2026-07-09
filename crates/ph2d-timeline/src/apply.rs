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
    apply_from_doc_except(world, doc, t, None);
}

/// Like [`apply_from_doc`], but leaves the entity whose bits are `live` (if any)
/// untouched — the caller is authoring its transform live this frame (a gizmo
/// drag), so the document must not fight the manipulation. Its `missing` flag is
/// still refreshed; only the write is skipped. With auto-key armed the drag
/// records keys, so once the gesture ends `apply_from_doc` resumes on the newly
/// recorded pose and the entity holds.
pub fn apply_from_doc_except(world: &mut World, doc: &mut TimelineDoc, t: f64, live: Option<u64>) {
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
        // The user is dragging this entity's gizmo — it owns its transform for
        // the duration of the gesture; don't clobber it from the document.
        if live == Some(entity_bits) {
            continue;
        }
        // Sample the bound track (skip empty tracks so a just-created binding
        // never forces the property to a default value).
        let sampled = doc.active_clip().track(target).and_then(|tr| {
            if tr.is_empty() {
                None
            } else {
                Some(tr.sample(t))
            }
        });
        if let Some(v) = sampled {
            write_prop(world, entity, prop, v);
        }
    }
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

        apply_from_doc_except(&mut w, &mut doc, 0.0, Some(dragged.to_bits()));

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

        // With no live entity, the document reclaims the dragged one too.
        apply_from_doc_except(&mut w, &mut doc, 0.0, None);
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

//! **Uma propriedade, para dentro e para fora de uma entidade** — o par inverso.
//!
//! Estas duas funções são uma a inversa da outra, e moram no mesmo arquivo por isso:
//! enquanto estavam separadas no meio do [`crate::apply`] a simetria era uma AFIRMAÇÃO
//! num doc-comment, e a afirmação ficou falsa em dois canais sem ninguém notar — a
//! leitora tomava só o `PropKind` e não conseguia responder por
//! [`crate::PropKind::Position`] (que precisa da trajetória do binding) nem por
//! `Morph` (uma omissão pura). Lado a lado, um braço sem par se vê (ADR-0146 C4).
//!
//! A metade de Position mora no irmão [`crate::apply_path`], que é exatamente o mesmo
//! trabalho para o único canal que é uma curva em vez de um campo.

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Transform, World};

use crate::prop::PropKind;
use crate::sprite::SpriteProp;

/// Write one resolved property value into an entity, via the sprite resolver.
///
/// Takes the whole BINDING, not just the kind: [`PropKind::Position`]'s value is a
/// distance, and turning it back into a place needs that binding's trajectory.
pub(crate) fn write_prop(
    world: &mut World,
    entity: Entity,
    b: &crate::TargetBinding,
    v: AnimValue,
    orient: bool,
) {
    let prop = b.prop;
    // Position (uma distância → ponto, ADR-0141) e os cinco de sprite-transform são POSE:
    // vão pela porta única `pose::set_transform_field`, a MESMA que o `pose_at` do onion
    // usa, para que não haja uma 2ª derivação da pose a divergir (ADR-0142).
    if prop == PropKind::Position || prop.as_sprite_transform().is_some() {
        if let Some(mut xf) = world.get_mut::<Transform>(entity) {
            crate::pose::set_transform_field(&mut xf, b, v, orient);
        }
        return;
    }
    // The Morph `t` (the Vector line's animatable channel). `VecMorph` lives in `ph2d-ecs`, which
    // this crate already depends on for `Transform`/`World` — no feature gate, no new dep. The
    // clamp is the motor's (`Plan::at`), so an ease with overshoot (back/elastic) touches the end
    // and stops instead of breaking the shape.
    if prop == PropKind::Morph
        && let AnimValue::Float(f) = v
        && let Some(mut m) = world.get_mut::<ph2d_ecs::VecMorph>(entity)
    {
        m.t = f;
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

/// Read one property back out of an entity — the exact inverse of
/// [`write_prop`], and the reason `rest` can be captured without the shell's
/// help. Also what the inverse-blend key authoring will need (ADR-0115 R9).
///
/// **Takes the whole BINDING, exactly like its twin, and for the same reason.**
/// [`PropKind::Position`]'s value is a *distance along a trajectory*, so turning
/// the object's pose back into that number needs this binding's path — the
/// argument `write_prop` has always taken. While this one took only the kind it
/// could not answer for Position at all, and the caller that needed the answer
/// (`refresh_liveness_and_rest`) carried a special case the OTHER two callers did
/// not have. That is the shape of "one question, two doors", and the two doors
/// disagreed: a `Nome.position` prop-link resolved 0.0 while the very same
/// binding's `rest` resolved correctly (ADR-0146 C4).
///
/// The arms below mirror [`write_prop`]'s, in the same order, on purpose: this
/// pair is an inverse, and the gate
/// `reading_a_property_back_is_the_inverse_of_writing_it` asserts it over EVERY
/// kind rather than over a list someone maintains.
pub(crate) fn read_prop(world: &World, entity: Entity, b: &crate::TargetBinding) -> Option<f32> {
    let prop = b.prop;
    // Position — read THROUGH the trajectory (the inverse of the pose door's
    // Position arm). `None` while the binding has no path yet, which is what an
    // unresolvable scalar does too.
    if prop == PropKind::Position {
        return crate::apply_path::read_distance(world, entity, b);
    }
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
    // The Morph `t`. No feature gate, for the same reason `write_prop` has none:
    // `VecMorph` lives in `ph2d-ecs`, which this crate already depends on.
    if prop == PropKind::Morph {
        return world.get::<ph2d_ecs::VecMorph>(entity).map(|m| m.t);
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

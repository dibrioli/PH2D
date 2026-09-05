//! **Uma propriedade, para dentro e para fora de uma entidade** — o par inverso.
//!
//! Estas duas funções são uma a inversa da outra, e moram no mesmo arquivo por isso:
//! enquanto estavam separadas no meio do [`crate::apply`] a simetria era uma AFIRMAÇÃO
//! num doc-comment, e a afirmação ficou falsa em dois canais sem ninguém notar — a
//! leitora tomava só o `PropKind` e não conseguia responder por
//! [`crate::PropKind::Position`] (que precisa da trajetória do binding) nem por
//! `Morph` (uma omissão pura). Lado a lado, um braço sem par se vê (ADR-0152 C4).
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
    path: Option<&crate::MotionPath>,
    v: AnimValue,
    orient: bool,
) {
    let prop = b.prop;
    // Position (uma distância → ponto, ADR-0141) e os cinco de sprite-transform são POSE:
    // vão pela porta única `pose::set_transform_field`, a MESMA que o `pose_at` do onion
    // usa, para que não haja uma 2ª derivação da pose a divergir (ADR-0142).
    if prop == PropKind::Position || prop.as_sprite_transform().is_some() {
        if let Some(mut xf) = world.get_mut::<Transform>(entity) {
            crate::pose::set_transform_field(&mut xf, b, path, v, orient);
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
    // **Os parâmetros de um JOINT** — o alvo do servo, a taxa do motor, os dois
    // comprimentos. `PhysicsJoint` mora na `ph2d-physics-ecs`, que o runtime base
    // não quer conhecer, então a dep é opcional atrás da feature `physics` —
    // exatamente a forma que o `Opacity` logo abaixo usa para o `ph2d-render`.
    //
    // ⚠️ **Uma escrita por si só não move o solver** — quem a leva ao rapier é o
    // `drive_joint_params` da ponte, por TICK. Sem ele o número chegaria uma vez
    // por quadro e, num replay, nunca.
    #[cfg(feature = "physics")]
    if let AnimValue::Float(f) = v
        && let Some(field) = joint_field(prop)
        && let Some(mut j) = world.get_mut::<ph2d_physics_ecs::PhysicsJoint>(entity)
    {
        *field.of_mut(&mut j) = f;
        return;
    }
    // ⭐⭐⭐ **A OPACIDADE DE UM CAMINHO VETORIAL** — o mesmo canal, o outro substrato.
    //
    // ⛔⛔ Até 2026-09-04 este canal era **MUDO** num vetor: o braço abaixo exige um
    // `ph2d_render::Sprite`, e uma entidade de caminho vetorial nasce com
    // `(Transform, Name, VecPathRef, RootOrder)` e nunca com `Sprite`. A row aparecia, aceitava
    // chaves e desenhava curva — e não movia um pixel. *Um controlo pintado e inerte.*
    //
    // ⚠️ **Por que um componente e não a tinta:** a tinta vive no `VecPath`, dentro da `VecScene`,
    // que é um campo da SHELL e não um recurso do ECS — não há assinatura de `write_prop` que a
    // alcance. O componente é a ponte, e a shell funde-o na projecção do quadro
    // (`ph2d_vec_scene::BoundStyle`). É o padrão do `VecStrokeProfile` (ADR-0148).
    //
    // ⚠️ **E ele é DESREGISTADO de propósito** — logo não entra no snapshot, não vai ao save e não
    // empilha undo. Sem isso uma reprodução de 3 s a 60 fps seria **180 passos de undo**, que é o
    // defeito que o `preview_drive` da shell existe para curar. O detalhe está no doc do
    // [`ph2d_ecs::VecDrivenStyle`].
    if prop == PropKind::Opacity
        && let AnimValue::Float(f) = v
        && world.get::<ph2d_ecs::VecPathRef>(entity).is_some()
    {
        let alpha = f.clamp(0.0, 1.0);
        if let Some(mut d) = world.get_mut::<ph2d_ecs::VecDrivenStyle>(entity) {
            d.alpha = Some(alpha);
        } else {
            world.entity_mut(entity).insert(ph2d_ecs::VecDrivenStyle {
                alpha: Some(alpha),
            });
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
/// binding's `rest` resolved correctly (ADR-0152 C4).
///
/// The arms below mirror [`write_prop`]'s, in the same order, on purpose: this
/// pair is an inverse, and the gate
/// `reading_a_property_back_is_the_inverse_of_writing_it` asserts it over EVERY
/// kind rather than over a list someone maintains.
pub(crate) fn read_prop(
    world: &World,
    entity: Entity,
    b: &crate::TargetBinding,
    path: Option<&crate::MotionPath>,
) -> Option<f32> {
    // Position — read THROUGH the trajectory (the inverse of the pose door's
    // Position arm). `None` while the binding has no path yet, which is what an
    // unresolvable scalar does too. It is the ONE kind that needs the binding, which
    // is why it is answered here and everything else delegates.
    if b.prop == PropKind::Position {
        return crate::apply_path::read_distance(world, entity, path);
    }
    read_prop_kind(world, entity, b.prop)
}

/// The same read, for a kind that needs **no binding** — every property whose value is
/// a plain component read.
///
/// ⚠️ Extracted rather than copied: [`read_prop`] delegates to it, so the pair
/// `read`/`write` stays ONE inverse and the gate over every kind still covers both
/// callers. Its second caller is `frame_solve::seed_unbound_links`, which needs the
/// value of an object the timeline does not animate — and therefore has no binding to
/// hand over.
///
/// `Position` is not answerable here and returns `None`: a distance along a trajectory
/// is a fact about a binding, not about an entity.
pub(crate) fn read_prop_kind(world: &World, entity: Entity, prop: PropKind) -> Option<f32> {
    if prop == PropKind::Position {
        return None;
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
    // Os parâmetros de joint — o braço espelho do `write_prop`, na mesma ordem.
    #[cfg(feature = "physics")]
    if let Some(field) = joint_field(prop) {
        return world
            .get::<ph2d_physics_ecs::PhysicsJoint>(entity)
            .map(|j| *field.of(j));
    }
    // ⭐ A opacidade de um caminho vetorial — o braço espelho do `write_prop`, e ele vem PRIMEIRO
    // pela mesma razão que lá: a entidade é um vetor ou é uma sprite, nunca as duas, e perguntar
    // ao substrato certo primeiro poupa a leitura que sabe que vai falhar.
    //
    // ⚠️ **A ausência do componente NÃO é opacidade zero, é *"ninguém a conduz"*.** O `rest` que
    // esta leitura semeia é o valor de ANTES de a linha do tempo assumir, e num caminho ainda não
    // conduzido esse valor é **opaco** — devolver `0.0` faria toda track nova nascer com um fade
    // a partir do invisível.
    if prop == PropKind::Opacity && world.get::<ph2d_ecs::VecPathRef>(entity).is_some() {
        return Some(
            world
                .get::<ph2d_ecs::VecDrivenStyle>(entity)
                .and_then(|d| d.alpha)
                .unwrap_or(1.0),
        );
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

/// **Qual campo do `PhysicsJoint` um [`PropKind`] nomeia** — e `None` para todo
/// kind que não é de joint.
///
/// Existe para que o par [`write_prop`]/[`read_prop_kind`] tenha **uma** tabela em
/// vez de dois `match` espelhados: a lição que o próprio cabeçalho deste arquivo
/// carrega é que dois braços separados param de ser inversos sem ninguém notar, e
/// aqui seriam oito. Com a tabela, um kind novo entra **uma** vez e os dois lados
/// o ganham juntos.
#[cfg(feature = "physics")]
#[derive(Clone, Copy)]
pub(crate) enum JointField {
    MotorTarget,
    MotorSpeed,
    RestLength,
    MaxLength,
}

#[cfg(feature = "physics")]
impl JointField {
    fn of(self, j: &ph2d_physics_ecs::PhysicsJoint) -> &f32 {
        match self {
            JointField::MotorTarget => &j.motor_target,
            JointField::MotorSpeed => &j.motor_speed,
            JointField::RestLength => &j.rest_length,
            JointField::MaxLength => &j.max_length,
        }
    }

    fn of_mut(self, j: &mut ph2d_physics_ecs::PhysicsJoint) -> &mut f32 {
        match self {
            JointField::MotorTarget => &mut j.motor_target,
            JointField::MotorSpeed => &mut j.motor_speed,
            JointField::RestLength => &mut j.rest_length,
            JointField::MaxLength => &mut j.max_length,
        }
    }
}

#[cfg(feature = "physics")]
pub(crate) fn joint_field(prop: PropKind) -> Option<JointField> {
    Some(match prop {
        PropKind::JointMotorTarget => JointField::MotorTarget,
        PropKind::JointMotorSpeed => JointField::MotorSpeed,
        PropKind::JointRestLength => JointField::RestLength,
        PropKind::JointMaxLength => JointField::MaxLength,
        _ => return None,
    })
}

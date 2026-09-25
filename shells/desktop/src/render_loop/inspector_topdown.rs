//! **A secção TOP-DOWN PLAYER: o instantâneo e o dreno** (TOP-20 #13, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — é o mesmo molde do [`super::inspector_factory`]:
//! quem lê o mundo para o painel e quem escreve a edição de volta fazem a MESMA tradução, e separá-los
//! seria a porta pela qual as duas divergem.
//!
//! # ⚠️ A tradução painel ⇄ lei tem UMA porta de cada lado
//!
//! O painel fala em tags de segmentado (`InspectorViewpoint`) e a lei fala em variantes
//! (`ph2d_topdown::Viewpoint`). ⛔ Ligá-las pelo `as u8` faria reordenar um enum trocar o que um
//! clique escreve — **e compila**. As duas tabelas abaixo são a porta, e há gate de ida-e-volta.

use ph2d_ecs::{Entity, World};
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
    TopDownFieldEdit,
};
use ph2d_physics_ecs::{BodyKind, PlatformPlayer, RigidBody, TopDownPlayer};
use ph2d_topdown::{direction::DirectionMode, rotation::RotationMode, viewpoint::Viewpoint};

/// A tradução lei → painel das direcções.
fn dir_para_painel(d: DirectionMode) -> InspectorMoveDirections {
    match d {
        DirectionMode::Free => InspectorMoveDirections::Free,
        DirectionMode::EightWay => InspectorMoveDirections::Eight,
        DirectionMode::FourWay => InspectorMoveDirections::Four,
        DirectionMode::AxisX => InspectorMoveDirections::AxisX,
        DirectionMode::AxisY => InspectorMoveDirections::AxisY,
    }
}

/// E a de volta.
fn dir_para_lei(d: InspectorMoveDirections) -> DirectionMode {
    match d {
        InspectorMoveDirections::Free => DirectionMode::Free,
        InspectorMoveDirections::Eight => DirectionMode::EightWay,
        InspectorMoveDirections::Four => DirectionMode::FourWay,
        InspectorMoveDirections::AxisX => DirectionMode::AxisX,
        InspectorMoveDirections::AxisY => DirectionMode::AxisY,
    }
}

fn view_para_painel(v: Viewpoint) -> InspectorViewpoint {
    match v {
        Viewpoint::TopDown => InspectorViewpoint::TopDown,
        Viewpoint::Isometric2to1 => InspectorViewpoint::Iso2to1,
        Viewpoint::Isometric30 => InspectorViewpoint::Iso30,
        Viewpoint::Custom => InspectorViewpoint::Custom,
    }
}

fn view_para_lei(v: InspectorViewpoint) -> Viewpoint {
    match v {
        InspectorViewpoint::TopDown => Viewpoint::TopDown,
        InspectorViewpoint::Iso2to1 => Viewpoint::Isometric2to1,
        InspectorViewpoint::Iso30 => Viewpoint::Isometric30,
        InspectorViewpoint::Custom => Viewpoint::Custom,
    }
}

fn face_para_painel(r: RotationMode) -> InspectorFacing {
    match r {
        RotationMode::None => InspectorFacing::None,
        RotationMode::ToMovement => InspectorFacing::Movement,
        RotationMode::Snap90 => InspectorFacing::Snap4,
        RotationMode::Snap45 => InspectorFacing::Snap8,
    }
}

fn face_para_lei(f: InspectorFacing) -> RotationMode {
    match f {
        InspectorFacing::None => RotationMode::None,
        InspectorFacing::Movement => RotationMode::ToMovement,
        InspectorFacing::Snap4 => RotationMode::Snap90,
        InspectorFacing::Snap8 => RotationMode::Snap45,
    }
}

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub(crate) fn build_topdown_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorTopDownInfo> {
    let e = Entity::from_bits(bits);
    let c = world.get::<TopDownPlayer>(e)?;
    let law = c.law();
    let corpo = world.get::<RigidBody>(e);
    Some(InspectorTopDownInfo {
        entity_bits: bits,
        speed: law.speed,
        acceleration: law.acceleration,
        deceleration: law.deceleration,
        directions: dir_para_painel(law.direction),
        viewpoint: view_para_painel(law.viewpoint),
        viewpoint_angle_deg: law.viewpoint_angle_deg,
        facing: face_para_painel(law.rotation),
        turn_speed_deg: law.rotation_speed_deg,
        min_slide_angle_deg: law.min_slide_angle_deg,
        max_slides: u32::from(law.max_slides),
        default_controls: law.default_controls,
        knockback_recovery: law.knockback_recovery,
        has_body: corpo.is_some(),
        // ⚠️ **A pergunta é `== Kinematic` e não `!= Dynamic`**: um corpo estático também não serve,
        // e a forma negativa deixaria-o passar como se estivesse bem.
        body_is_kinematic: corpo.is_some_and(|b| b.kind == BodyKind::Kinematic),
        conflicts_with_platformer: world.get::<PlatformPlayer>(e).is_some(),
        clock_playing,
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub(crate) fn apply_topdown_edit(world: &mut World, bits: u64, edit: &TopDownFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    let Some(mut c) = world.get_mut::<TopDownPlayer>(e) else {
        return false;
    };
    let mut law = c.law();
    match edit {
        TopDownFieldEdit::Speed(v) => law.speed = v.max(0.0),
        TopDownFieldEdit::Acceleration(v) => law.acceleration = v.max(0.0),
        TopDownFieldEdit::Deceleration(v) => law.deceleration = v.max(0.0),
        TopDownFieldEdit::Directions(d) => law.direction = dir_para_lei(*d),
        TopDownFieldEdit::Viewpoint(v) => law.viewpoint = view_para_lei(*v),
        // ⚠️ **A cerca é a do PAINEL escrita também AQUI**: o campo é alcançável por outra rota
        // (um script, um ficheiro), e uma elevação fora de `(0, 90)` não descreve tabuleiro nenhum.
        TopDownFieldEdit::ViewpointAngle(v) => law.viewpoint_angle_deg = v.clamp(1.0, 89.0),
        TopDownFieldEdit::Facing(f) => law.rotation = face_para_lei(*f),
        TopDownFieldEdit::TurnSpeed(v) => law.rotation_speed_deg = v.max(0.0),
        TopDownFieldEdit::MinSlideAngle(v) => law.min_slide_angle_deg = v.clamp(0.0, 89.0),
        // ⚠️ **Pelo menos UM**: com zero deslizes o corpo pára em toda parede, e o componente
        // deixaria de fazer a única coisa que ele existe para fazer.
        TopDownFieldEdit::MaxSlides(n) => {
            law.max_slides = u8::try_from((*n).clamp(1, 8)).unwrap_or(4);
        }
        TopDownFieldEdit::DefaultControls(b) => law.default_controls = *b,
        TopDownFieldEdit::KnockbackRecovery(v) => law.knockback_recovery = v.max(0.0),
    }
    *c = TopDownPlayer::from_law(law);
    true
}

#[cfg(test)]
#[path = "inspector_topdown_tests.rs"]
mod tests;

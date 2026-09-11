//! **O contorno de uma plataforma que está a ser ATRAVESSADA** (W20) — irmão
//! por ASSUNTO do `physics_overlay_scene_tests.rs`, que passou o teto de 600.
//!
//! ⚠️ O corte é o do assunto, e não do tamanho: aqui mora a única pergunta do
//! overlay que não é *"onde está esta forma?"* e sim *"ela está sólida para
//! alguém agora?"* — o estado que toda a classe de defeitos da descida deixava
//! invisível.

use super::tests::{camera, window};
use super::{PASSABLE_RGBA, STATIC_RGBA, outlines};

/// **Uma prancha que o player está a atravessar NÃO se desenha como chão.**
///
/// ⚠️ O motivo não é enfeite: toda a classe de defeitos da descida (W12/W20) é
/// *a prancha ficou fantasma e ninguém viu* — um estado que muda a colisão da
/// cena inteira e não aparece na tela é um estado que o artista descobre por
/// acidente. E a marca é da PLATAFORMA, não do player, porque é ela que deixa
/// de ser sólida.
///
/// Mutação: apagar o braço `passable` do `outline_rgba` deixa as duas leituras
/// idênticas e este gate vermelho.
#[test]
fn a_platform_being_passed_through_is_not_drawn_as_solid_ground() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, OneWayPlatform, RigidBody};
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        OneWayPlatform,
    ));

    let solid = outlines(true, false, &mut sim, &[], false, &camera(), window());
    assert_eq!(solid.len(), 1);
    assert_eq!(
        solid[0].1, STATIC_RGBA,
        "sem ninguem a atravessar, a prancha e' chao estatico como qualquer outro"
    );

    let ghost = outlines(true, false, &mut sim, &[], true, &camera(), window());
    assert_eq!(
        ghost[0].1, PASSABLE_RGBA,
        "com uma descida em curso ela tem de dizer que NAO esta' la'"
    );
}

/// E o contorno de uma plataforma que NÃO é one-way não muda por causa de uma
/// descida — o bit viaja no corpo do player e vale para as jump-through, não
/// para o chão sólido.
#[test]
fn a_drop_does_not_dim_solid_ground() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let drawn = outlines(true, false, &mut sim, &[], true, &camera(), window());
    assert_eq!(
        drawn[0].1, STATIC_RGBA,
        "chao solido continua solido durante uma descida"
    );
}

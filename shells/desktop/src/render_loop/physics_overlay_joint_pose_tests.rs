//! **O que de um joint se POSA** — os gates da W-J3.
//!
//! Irmão do `physics_overlay_joints_tests`, separado dele por RESPONSABILIDADE
//! (e pelo cap de 600 LOC do shell): lá se prova o que o joint DIZ de si (o
//! vocabulário de glifos da W-J1), aqui se prova que o que ele diz é a mesma
//! coisa que se AGARRA — o grip sobre a parede desenhada, o grip sobre o anel, e
//! o fantasma que mostra a pose sem escrevê-la.

use super::{camera, view, window};
use ph2d_physics_ecs::JointKind;

/// **O grip de uma parede está EXATAMENTE onde a parede é desenhada.**
///
/// `limit_end_screen` responde *"onde agarro?"* e `limit_arc` desenha *"onde
/// termina o alcance?"*. Se fossem duas derivações, a que discordasse seria a
/// invisível — o retângulo de hit — e o artista clicaria na parede sem pegar
/// nada. O gate prova que o ponto do grip é um ponto do CAMINHO desenhado.
///
/// Mutação-testada: mudar o raio numa das duas (por exemplo `LIMIT_ARC_PX + 1.0`
/// no grip) tira o ponto do caminho e isto fica vermelho.
#[test]
fn the_limit_grip_sits_on_the_wall_that_is_drawn() {
    use crate::render_loop::physics_overlay_joint_glyphs::{limit_arc, limit_end_screen};
    let cam = camera();
    let win = window();
    let limits = [-0.7_f32, 1.1];
    let path = limit_arc(&cam, win, [0.5, -0.25], 0.3, limits, 0.0);
    let pts: Vec<ph2d_vector::Point> = path
        .elements()
        .iter()
        .filter_map(|e| match e {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    for l in limits {
        let grip = limit_end_screen(&cam, win, [0.5, -0.25], 0.3, l);
        let near = pts
            .iter()
            .map(|p| (p.x - grip.x).hypot(p.y - grip.y))
            .fold(f64::INFINITY, f64::min);
        assert!(
            near < 1e-6,
            "o grip do limite {l:.3} caiu a {near:.4} px do desenho mais próximo — \
             a parede que se vê e a que se agarra são dois lugares"
        );
    }
}

/// **O grip do anel fica SOBRE o anel, na direção de B.** Em mundo, porque um
/// comprimento é um comprimento: o grip cresce com o zoom junto com o anel.
#[test]
fn the_length_grip_sits_on_the_ring_towards_body_b() {
    use crate::render_loop::physics_overlay_joint_glyphs::length_handle_world;
    let a = [1.0_f32, 2.0];
    let b = [4.0_f32, 6.0]; // 3-4-5: a 5 m de A
    let g = length_handle_world(a, b, 2.0);
    let r = (g[0] - a[0]).hypot(g[1] - a[1]);
    assert!((r - 2.0).abs() < 1e-5, "sobre o anel de raio 2, got {r:.4}");
    // …e colinear com A→B (o produto vetorial some).
    let cross = (b[0] - a[0]) * (g[1] - a[1]) - (b[1] - a[1]) * (g[0] - a[0]);
    assert!(cross.abs() < 1e-4, "fora da direção de B, cross {cross:.5}");
    // Degenerado: B sobre A cai onde `ring_px` começa a desenhar.
    let d = length_handle_world(a, a, 2.0);
    assert_eq!(d, [a[0] + 2.0, a[1]]);
}

/// **O FANTASMA desenha, e NADA mais.**
///
/// Ele é o collider de B girado em torno da âncora até o ângulo que o limite
/// nomeia — função pura da view e do número. O corpo real só se move quando o
/// solver o move, e é essa separação que torna possível posar um limite com a
/// simulação parada.
///
/// Mutação-testada: fazer o fantasma escrever `Transform` de B (a "ajuda" óbvia)
/// derruba a metade que compara a pose antes/depois.
#[test]
fn the_ghost_draws_the_limit_pose_without_moving_the_body() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, SimWorld, Transform};
    use ph2d_physics_ecs::{Collider, ColliderShape};

    let mut sim = SimWorld::new();
    let arm = sim
        .world_mut()
        .spawn((
            Name::new("Arm".to_string()),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(1.0, 0.0)),
        ))
        .id();
    let mut v = view(JointKind::Pin);
    v.body_b = Some(arm);
    v.centre_b = [1.0, 0.0];
    let views = [v];
    let before = *sim.world().get::<Transform>(arm).expect("arm");

    let flat = crate::render_loop::physics_overlay_joints::limit_ghost(
        &sim,
        &views,
        Some((views[0].entity, 0.0)),
        &camera(),
        window(),
    )
    .expect("um limite em 0 já tem fantasma (sobre o corpo)");
    let turned = crate::render_loop::physics_overlay_joints::limit_ghost(
        &sim,
        &views,
        Some((views[0].entity, std::f32::consts::FRAC_PI_2)),
        &camera(),
        window(),
    )
    .expect("e a 90° também");

    // O corpo NÃO se moveu.
    let after = *sim.world().get::<Transform>(arm).expect("arm");
    assert_eq!(
        (after.translation.x, after.translation.y, after.rotation),
        (before.translation.x, before.translation.y, before.rotation),
        "o fantasma escreveu na pose real do corpo B"
    );

    // E a silhueta MOVEU com o limite (senão ele desenharia o corpo onde já está,
    // que é o que a agulha viva do arco já diz).
    let bbox = |p: &ph2d_vector::BezPath| {
        p.elements()
            .iter()
            .fold((f64::MAX, f64::MAX), |acc, e| match e {
                ph2d_vector::PathEl::MoveTo(q) | ph2d_vector::PathEl::LineTo(q) => {
                    (acc.0.min(q.x), acc.1.min(q.y))
                }
                _ => acc,
            })
    };
    let (fx, fy) = bbox(&flat);
    let (tx, ty) = bbox(&turned);
    assert!(
        (fx - tx).abs() + (fy - ty).abs() > 5.0,
        "a silhueta a 0° e a 90° saíram no mesmo lugar — o fantasma não segue o limite"
    );

    // Sem arrasto, sem fantasma.
    assert!(
        crate::render_loop::physics_overlay_joints::limit_ghost(
            &sim,
            &views,
            None,
            &camera(),
            window()
        )
        .is_none()
    );
}

//! **O Slider: o eixo é a ROTAÇÃO da entidade-joint** (W-J5).
//!
//! O 5º tipo é o espelho do Pin — um grau de liberdade de TRANSLAÇÃO em vez de
//! rotação — e a pergunta de projeto que ele traz é *onde mora o eixo?*.
//!
//! A resposta é a que Godot e Unreal dão, e a que este componente já implicava:
//! **na rotação da própria entidade-joint**. O `Transform` de um joint é onde a
//! *colocação* dele vive (a translação é a âncora), então a direção de uma
//! colocação vive na rotação — e o eixo fica autorável no dia um, pelo campo
//! Rotation do Inspector, com zero widget novo.
//!
//! Estes gates pinam as três metades: a rotação AIMA o trilho, o curso PARA o
//! carro, e a troca de tipo RE-SEMEIA o alcance (porque `limit_min/max` carregam
//! a unidade do tipo — radianos num Pin, metros num Slider).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Um trilho estático em `(0, 5)` e um carro no MESMO ponto — um Slider
/// compartilha uma âncora, e ela é o zero do curso.
fn rig(joint_rot: f32, limits: Option<[f32; 2]>) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Rail"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Car"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    let mut t = Transform::from_translation(Vec2::new(0.0, 5.0));
    t.rotation = joint_rot;
    sim.world_mut().spawn((
        Name::new("Rail Joint"),
        PhysicsJoint {
            body_a: stable_name_id("Rail"),
            body_b: stable_name_id("Car"),
            kind: JointKind::Slider,
            limits_enabled: limits.is_some(),
            limit_min: limits.map_or(0.0, |l| l[0]),
            limit_max: limits.map_or(0.0, |l| l[1]),
            ..PhysicsJoint::default()
        },
        t,
    ));
    sim
}

fn car_after(sim: &mut SimWorld, ticks: u64) -> [f32; 2] {
    let mut b = PhysicsBridge::new();
    for t in 1..=ticks {
        b.dispatch(sim, false, t);
    }
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    let (_, t) = q
        .iter(sim.world())
        .find(|(n, _)| n.as_str() == "Car")
        .expect("Car alive");
    [t.translation.x, t.translation.y]
}

/// **A rotação da entidade-joint aima o trilho.**
///
/// O MESMO rig, três rotações. Horizontal (0°) o carro não pode cair — a
/// gravidade é perpendicular ao trilho; vertical (−90°) ele desce em linha reta;
/// a 45° ele corre pela diagonal. Um eixo ignorado deixaria os três iguais.
///
/// ⚠️ O caso horizontal é o CONTROLE e é o que dá sentido aos outros dois: sem
/// ele, "o carro desceu" seria satisfeito por um corpo em queda livre.
///
/// Mutação: `axis_locals` devolver sempre `[1, 0]` ⇒ os três param no
/// horizontal e as duas asserções de queda ficam vermelhas.
#[test]
fn the_joints_rotation_aims_the_rail() {
    let flat = car_after(&mut rig(0.0, None), 90);
    assert!(
        flat[1] > 4.9,
        "trilho horizontal: a gravidade e perpendicular, o carro nao desce \
         (o CONTROLE) — caiu para {flat:?}"
    );

    let down = car_after(&mut rig(-std::f32::consts::FRAC_PI_2, None), 90);
    assert!(
        down[1] < 4.0 && down[0].abs() < 0.05,
        "trilho vertical: desce em linha reta, got {down:?}"
    );

    let diag = car_after(&mut rig(-std::f32::consts::FRAC_PI_4, None), 90);
    let (dx, dy) = (diag[0], diag[1] - 5.0);
    assert!(
        dx > 0.1,
        "trilho a 45 graus: o carro tem de andar, got {diag:?}"
    );
    assert!(
        (dy / dx + 1.0).abs() < 0.1,
        "e pela DIAGONAL (dy/dx = -1), got {diag:?} ratio {}",
        dy / dx
    );
}

/// **O curso é em METROS e para o carro.**
///
/// Trilho vertical, curso `[-0.5, 0.5]`: o carro cai meio metro e fica. Sem
/// limites ele continua — o controle.
///
/// Mutação: `has_limits()` devolver só `is_hinge()` ⇒ o Slider perde o alcance,
/// o carro passa de 0,5 m e isto fica vermelho.
#[test]
fn a_limited_slider_stops_the_car_after_half_a_metre() {
    let free = car_after(&mut rig(-std::f32::consts::FRAC_PI_2, None), 150);
    let capped = car_after(
        &mut rig(-std::f32::consts::FRAC_PI_2, Some([-0.5, 0.5])),
        150,
    );
    assert!(
        free[1] < 3.5,
        "o controle: sem limites ele segue caindo, got {free:?}"
    );
    assert!(
        (5.0 - capped[1] - 0.5).abs() < 0.06,
        "com curso de meio metro ele para em y = 4.5, got {capped:?}"
    );
}

/// **A troca de tipo re-semeia o alcance — porque a UNIDADE muda.**
///
/// `limit_min/max` carregam a unidade do tipo: radianos num Pin, metros num
/// Slider. Sem re-semear, os ±45° de um Pin (±0,785 rad) viram ±0,785 **metros**
/// de curso — um número que ninguém digitou — e um trilho de 0,5 m lido como
/// radianos vira uma dobradiça de 28,6°.
///
/// ⚠️ E o inverso é gateado junto: Pin→Weld→Pin **preserva** os ângulos, que é a
/// promessa que o componente faz sobre trocar de tipo. Só uma troca de UNIDADE
/// re-semeia.
#[test]
fn switching_between_a_hinge_and_a_rail_re_seeds_the_range() {
    // A porta que os dois lados leem.
    assert!(JointKind::Slider.limits_in_metres());
    assert!(!JointKind::Pin.limits_in_metres());
    assert!(JointKind::Slider.has_limits() && JointKind::Pin.has_limits());
    assert!(!JointKind::Spring.has_limits() && !JointKind::Weld.has_limits());

    let hinge = PhysicsJoint::default_limits(JointKind::Pin);
    let rail = PhysicsJoint::default_limits(JointKind::Slider);
    assert!(
        (hinge[1] - std::f32::consts::FRAC_PI_4).abs() < 1e-6,
        "a dobradica nasce em +-45 graus (radianos), got {hinge:?}"
    );
    assert!(
        (rail[1] - PhysicsJoint::DEFAULT_STROKE).abs() < 1e-6,
        "o trilho nasce em +-0,5 m, got {rail:?}"
    );
    assert!(
        (hinge[1] - rail[1]).abs() > 0.2,
        "os dois defaults tem de DIFERIR, senao a re-semeadura nao pode ser \
         observada: {hinge:?} vs {rail:?}"
    );
}

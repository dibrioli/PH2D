//! ⭐ (o achado da W13) **Um corpo LARGO que anda** — o desvio local (ORCA) escolhe a velocidade mais
//! perto da pedida e não contorna; a malha só o vê PARADO (plano 30 §22.5).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, NavTarget, PhysicsBridge, RigidBody};

use super::nav_desvio::{R, agente, dist, pos, regiao};

/// Uma barreira cinemática `2·hx × 2·hy` em `c`.
fn barreira(sim: &mut SimWorld, c: (f32, f32), hx: f32, hy: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Barreira"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: hx,
                    half_y: hy,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(c.0, c.1)),
        ))
        .id()
}

/// A corrida: o agente de `(-6, 0)` para `(6, 0)`, a barreira (`0,3 × 2·hy`) a andar a `vx` m/s a
/// partir de `x0`. Devolve o tique em que chegou (`None` = não chegou), a menor folga à barreira
/// ENQUANTO ela anda (a distância ao rectângulo menos o raio) e o pior recuo atrás da partida.
fn corrida(hy: f32, x0: f32, vx: f32, desvio: bool) -> (Option<u64>, f32, f32) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let b = barreira(&mut sim, (x0, 0.0), 0.15, hy);
    let quem = agente(
        &mut sim,
        "A",
        (-6.0, 0.0),
        NavTarget::Point([6.0, 0.0]),
        desvio,
    );
    let mut bridge = PhysicsBridge::new();
    let (mut chegou, mut folga, mut recuo) = (None, f32::INFINITY, 0.0f32);
    for t in 1..=900_u64 {
        let bx = x0 + vx * t as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Transform>(b)
            .expect("a barreira")
            .translation
            .x = bx;
        bridge.dispatch(&mut sim, true, t);
        let p = pos(&sim, quem);
        let (dx, dy) = (
            ((p.0 - bx).abs() - 0.15).max(0.0),
            (p.1.abs() - hy).max(0.0),
        );
        folga = folga.min((dx * dx + dy * dy).sqrt() - R);
        recuo = recuo.max(-6.0 - p.0);
        if chegou.is_none() && dist(p, (6.0, 0.0)) < 0.15 {
            chegou = Some(t);
        }
    }
    (chegou, folga, recuo)
}

/// ⭐ (W14) **Um corpo LARGO que vem de frente é contornado** — o desvio vê-o em POLÍGONO com a
/// velocidade dele, e o agente desliza à volta, com a folga do que ele anda num horizonte. Medido antes
/// (a fileira de discos da W6): nenhum dos três chegava — colado à frente da barreira, ou empurrado `6 m`.
/// CONTROLO: o mesmo agente SEM desvio fica preso (a cena contém a armadilha). E à frente, no mesmo
/// sentido, continua a passar.
#[test]
fn um_corpo_largo_que_vem_de_frente_e_contornado() {
    for (nome, hy, vx, ate) in [
        ("1,2 m a 0,3 m/s", 0.6, -0.3, 260),
        ("3 m a 0,3 m/s", 1.5, -0.3, 650),
        ("3 m a 1 m/s", 1.5, -1.0, 360),
    ] {
        let (chegou, folga, recuo) = corrida(hy, 3.0, vx, true);
        eprintln!("de frente, {nome}: chegou {chegou:?}, folga {folga:.3}, recuo {recuo:.3}");
        assert!(
            chegou.is_some_and(|t| t <= ate),
            "{nome}: chegou {chegou:?}"
        );
        assert!(folga > 0.05, "{nome}: roçou a barreira (folga {folga})");
        assert!(recuo <= 0.0, "{nome}: empurrado {recuo} m para trás");
    }
    let (sem, _, _) = corrida(1.5, 3.0, -0.3, false);
    assert_eq!(sem, None, "o CONTROLO sem desvio passou");
    let (a_frente, _, _) = corrida(1.5, -3.0, 0.3, true);
    assert!(a_frente.is_some_and(|t| t <= 230), "à frente: {a_frente:?}");
}

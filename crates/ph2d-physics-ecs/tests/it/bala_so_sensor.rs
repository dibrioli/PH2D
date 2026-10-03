//! ⭐⭐⭐ **A bala SÓ-SENSOR — a hitbox que perfura** (o defeito G do plano 28, achado pela sonda
//! `mede_o_golpe_que_chega`): o `move_character_from` devolvia deslocamento nulo a um corpo sem forma
//! SÓLIDA, logo ela ficava parada onde nasceu, em silêncio. Hoje ela anda livre e o toque sai pelo
//! sensor.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, GravityScale, Health, PhysicsBridge,
    ProjectileMotion, RigidBody,
};

fn caixa(h: f32, sensor: bool) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: h,
            half_y: h,
        },
        density: 1.0,
        is_sensor: sensor,
        ..Collider::default()
    }
}

fn corpo(
    sim: &mut SimWorld,
    nome: &str,
    kind: BodyKind,
    col: Collider,
    x: f32,
) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody { kind },
            col,
            Transform::from_translation(Vec2::new(x, 0.0)),
        ))
        .id()
}

/// ⭐⭐⭐ **Atravessa a parede e FERE o alvo do outro lado** — também a `240 m/s` (`4 m` por tique,
/// mais que o alvo de `0,8 m`). O CONTROLO: a mesma bala SÓLIDA pára na parede e não fere.
///
/// **Mutação que deve sangrar:** o ramo do só-sensor do `move_character_from` devolver zero.
#[test]
fn uma_bala_so_sensor_atravessa_a_parede_e_fere() {
    for (v, sensor) in [(12.0_f32, true), (240.0, true), (12.0, false)] {
        let mut sim = SimWorld::new();
        let alvo = corpo(
            &mut sim,
            "Alvo",
            BodyKind::Kinematic,
            caixa(0.4, false),
            0.0,
        );
        sim.world_mut().entity_mut(alvo).insert(Health::default());
        corpo(
            &mut sim,
            "Parede",
            BodyKind::Static,
            caixa(0.3, false),
            -1.5,
        );
        let bala = corpo(
            &mut sim,
            "Bala",
            BodyKind::Kinematic,
            caixa(0.1, sensor),
            -3.0,
        );
        sim.world_mut().entity_mut(bala).insert((
            GravityScale(0.0),
            ProjectileMotion {
                initial_speed: v,
                range: 0.0,
                ..ProjectileMotion::default()
            },
            Damage {
                amount: 20.0,
                ..Damage::default()
            },
        ));
        let mut ponte = PhysicsBridge::new();
        for t in 0..=60 {
            ponte.dispatch(&mut sim, true, t);
        }
        let vida = ponte.health_of(alvo).map_or(f64::NAN, |s| s.vida().pontos);
        let x = sim
            .world()
            .get::<Transform>(bala)
            .map_or(f32::NAN, |t| t.translation.x);
        if sensor {
            assert_eq!(vida, 80.0, "a {v} m/s a bala-sensor não feriu");
            assert!(
                x > 0.4,
                "a {v} m/s a bala-sensor não passou o alvo (x = {x})"
            );
        } else {
            assert_eq!(vida, 100.0, "a bala SÓLIDA atravessou a parede");
            assert!(x < -1.5, "a bala SÓLIDA passou a parede (x = {x})");
        }
    }
}

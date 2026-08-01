//! **A sonda:** quantos contatos um corpo COMPOSTO reporta ao encostar no chão?
//!
//! `cargo test -p ph2d-physics-ecs --release measure_compound_contact -- --ignored
//! --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const HALF_X: f32 = 0.6;
const HALF_Y: f32 = 0.25;

fn cuboid(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

/// `compound = false` ⇒ UMA caixa larga (o CONTROLE), mesma silhueta e massa.
fn raft(sim: &mut SimWorld, compound: bool) -> Entity {
    let hull = if compound { HALF_X } else { HALF_X * 2.0 };
    let x = if compound { 0.0 } else { HALF_X };
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Raft"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboid(hull, HALF_Y),
            Transform::from_translation(Vec2::new(x, 1.0)),
        ))
        .id();
    if compound {
        sim.world_mut().spawn((
            Name::new("Raft Deck"),
            cuboid(HALF_X, HALF_Y),
            Transform::from_translation(Vec2::new(HALF_X * 2.0, 0.0)),
            ChildOf(body),
        ));
    }
    body
}

fn ground(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(10.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
}

fn run(compound: bool) -> (PhysicsBridge, SimWorld, Entity) {
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let raft = raft(&mut sim, compound);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=180 {
        bridge.dispatch(&mut sim, true, t);
    }
    (bridge, sim, raft)
}

/// Quantos pares de COLLIDER a composta tem ativos na rampa? (o desempate de
/// profundidade so' existe com dois.)
#[test]
#[ignore = "sonda de medição"]
fn measure_ramp_pairs() {
    use ph2d_ecs::Transform as T;
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(10.0, 0.5),
        T {
            translation: Vec2::new(0.0, -0.5),
            rotation: -0.25,
            ..T::default()
        },
    ));
    let r = raft(&mut sim, true);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=90 {
        bridge.dispatch(&mut sim, true, t);
    }
    let c: Vec<_> = bridge
        .contacts()
        .iter()
        .filter(|c| c.a == r || c.b == r)
        .collect();
    println!("\n=== rampa: {} entrada(s) fundida(s) ===", c.len());
    for x in &c {
        println!(
            "   ponto ({:.4}, {:.4})  impulso {:.6}",
            x.point[0], x.point[1], x.impulse
        );
    }
    let t = ph2d_ecs::world_transform(sim.world(), r).unwrap();
    println!(
        "   pose do corpo: x {:.4} y {:.4} rot {:.4} rad",
        t.translation.x, t.translation.y, t.rotation
    );
}

#[test]
#[ignore = "sonda de medição"]
fn measure_compound_contact() {
    println!("\n=== quantos contatos uma jangada reporta pousada no chao ===");
    for compound in [false, true] {
        let (bridge, _sim, raft) = run(compound);
        let lane = if compound { "COMPOSTA " } else { "controle " };
        let mine: Vec<_> = bridge
            .contacts()
            .iter()
            .filter(|c| c.a == raft || c.b == raft)
            .collect();
        let soma: f32 = mine.iter().map(|c| c.impulse).sum();
        println!(
            "  {lane} contatos {}   count {}   impulso somado {soma:.6}",
            mine.len(),
            bridge.contact_count(raft),
        );
        for c in &mine {
            println!(
                "      ponto ({:>7.4}, {:>7.4})  impulso {:.6}  impacto {:.6}",
                c.point[0], c.point[1], c.impulse, c.impact
            );
        }
    }
    println!("  (a silhueta e a massa sao IGUAIS nas duas)");
}

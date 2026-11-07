//! O Density de um corpo que não é Dynamic: o cartão de física mostra-o em Static e Kinematic. Muda alguma coisa no
//! que acontece? Medição da GEF02 (crítica do Physics Body: «não confirmei para Static»). Um corpo Dynamic na mesma
//! cena é o controlo: nele o Density tem de mudar o resultado, senão a cena não mede nada.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, PhysicsBridge, RigidBody,
};

const TICKS: u64 = 300;
/// O Kinematic anda por este tanto a cada passo, e empurra quem encontra.
const PASSO_KINEMATIC: f32 = 0.02;

/// Uma caixa lançada contra o corpo medido e uma bola largada em cima dele. Devolve os bits de onde as duas acabam.
fn corrida(kind: BodyKind, density: f32) -> [u32; 6] {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    let medido = w
        .spawn((
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                density,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    let caixa = w
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            InitialVelocity {
                linvel: [-6.0, 0.0],
                angvel: 0.0,
            },
            Transform::from_translation(Vec2::new(5.0, 0.5)),
        ))
        .id();
    let bola = w
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                restitution: 0.5,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.4, 6.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=TICKS {
        if kind == BodyKind::Kinematic {
            mover(&mut sim, medido);
        }
        bridge.dispatch(&mut sim, true, tick);
    }
    let pose = |e: Entity| {
        let t = sim.world().get::<Transform>(e).unwrap();
        [
            t.translation.x.to_bits(),
            t.translation.y.to_bits(),
            t.rotation.to_bits(),
        ]
    };
    let [a, b] = [pose(caixa), pose(bola)];
    [a[0], a[1], a[2], b[0], b[1], b[2]]
}

fn mover(sim: &mut SimWorld, e: Entity) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.x += PASSO_KINEMATIC;
    }
}

#[test]
fn o_density_nao_muda_nada_num_corpo_static() {
    let base = corrida(BodyKind::Static, 1.0);
    assert_eq!(base, corrida(BodyKind::Static, 0.01));
    assert_eq!(base, corrida(BodyKind::Static, 1000.0));
}

#[test]
fn o_density_nao_muda_nada_num_corpo_kinematic() {
    let base = corrida(BodyKind::Kinematic, 1.0);
    assert_eq!(base, corrida(BodyKind::Kinematic, 0.01));
    assert_eq!(base, corrida(BodyKind::Kinematic, 1000.0));
}

/// O controlo: sem ele, as duas igualdades acima passariam numa cena em que nada encosta em nada.
#[test]
fn o_density_muda_o_resultado_num_corpo_dynamic() {
    let base = corrida(BodyKind::Dynamic, 1.0);
    assert_ne!(base, corrida(BodyKind::Dynamic, 0.01));
    assert_ne!(base, corrida(BodyKind::Dynamic, 1000.0));
}

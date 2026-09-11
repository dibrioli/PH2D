//! A ponte dobra o `AreaFalloff` opcional na sim, e um rewind o RE-ARMA (W-AreaFalloff).
//!
//! A `ph2d-physics` prova a régua e a lei; esta é a metade ECS, testada pelo RESULTADO que
//! um artista confere: dois corpos na mesma coluna de vento, um no olho e outro na margem,
//! saem empurrados diferente — e voltam a sair assim depois de um scrub até t=0, o que só
//! pode acontecer se o falloff ridou o `BodyDesc` de que o mundo se reconstrói (lei 2).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, AreaFalloff, BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsSettings,
    RigidBody,
};

/// Sem gravidade de mundo: a zona é a única coisa agindo, então o que sobra é o que ela fez.
fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// A meia-extensão da coluna de vento. Grande de propósito — o corpo da margem tem de
/// continuar DENTRO durante a corrida, senão "andou menos" é só "saiu".
const HALF: f32 = 12.0;

/// Uma coluna de vento estática que empurra em +X. `falloff` liga o desvanecimento.
fn wind(sim: &mut SimWorld, falloff: f32) {
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: HALF,
                    half_y: HALF,
                },
                density: 1.0,
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            AreaEffector { force: [4.0, 0.0] },
        ))
        .id();
    if falloff > 0.0 {
        sim.world_mut().entity_mut(e).insert(AreaFalloff(falloff));
    }
}

fn drifter(sim: &mut SimWorld, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.5 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ))
        .id()
}

fn play_to(bridge: &mut PhysicsBridge, sim: &mut SimWorld, tick: u64) {
    let from = bridge.last_stepped();
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
    }
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.x
}

/// Roda a cena e devolve o quanto cada corpo (olho, margem) andou; opcionalmente rebobina
/// até t=0 e roda de novo, que é o teste do re-arme.
fn run(falloff: f32, rewind: bool) -> (f32, f32) {
    let mut sim = SimWorld::new();
    wind(&mut sim, falloff);
    let near = drifter(&mut sim, 0.0);
    let far = drifter(&mut sim, 0.9 * HALF);
    let (x0n, x0f) = (x_of(&sim, near), x_of(&sim, far));
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 12);
    if rewind {
        bridge.dispatch(&mut sim, false, 0);
        play_to(&mut bridge, &mut sim, 12);
    }
    for h in [near, far] {
        let x = x_of(&sim, h);
        assert!(
            x.abs() < HALF,
            "a fixture perdeu o fenômeno: o corpo saiu da zona (x = {x})"
        );
    }
    (x_of(&sim, near) - x0n, x_of(&sim, far) - x0f)
}

/// **A ponte dobra o falloff, e um rewind o preserva.**
///
/// O controle vem junto e é o que dá sentido à medida: sem o componente os dois corpos, na
/// mesma linha do mesmo vento, andam o mesmo.
#[test]
fn the_bridge_folds_the_zones_falloff_and_a_rewind_preserves_it() {
    let (flat_near, flat_far) = run(0.0, false);
    assert!(
        (flat_near - flat_far).abs() < 1e-3,
        "controle: sem `AreaFalloff` o campo é uniforme, mas o do olho andou {flat_near} e \
         o da margem {flat_far}"
    );

    let (near, far) = run(1.0, false);
    assert!(
        near > far * 3.0,
        "com `AreaFalloff(1)` o corpo do olho ({near}) tem de andar muito mais que o da \
         margem ({far}) — a ponte não está dobrando o componente na sim"
    );

    // O mesmo, depois de um scrub até t=0 e um replay. Se o falloff não ridasse o
    // `BodyDesc`, o mundo reconstruído perderia o desvanecimento e os dois voltariam a
    // andar igual — em silêncio, e só depois de o artista mexer na régua.
    let (rn, rf) = run(1.0, true);
    assert!(
        (rn - near).abs() < 1e-3 && (rf - far).abs() < 1e-3,
        "depois de um rewind o falloff não foi re-armado (({near}, {far}) -> ({rn}, {rf}))"
    );
}

/// **O falloff sozinho não faz uma zona.** Ele é um MODIFICADOR: sem força nem torque não
/// há o que atenuar, então a área continua inerte e nada é registrado — nem para acordar
/// um corpo dormente.
#[test]
fn a_falloff_alone_does_not_make_a_zone() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: HALF,
                    half_y: HALF,
                },
                density: 1.0,
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    sim.world_mut().entity_mut(e).insert(AreaFalloff(1.0));
    let b = drifter(&mut sim, 0.0);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 30);
    let x = x_of(&sim, b);
    assert!(
        x.abs() < 1e-6,
        "um falloff sem empurrão não é uma zona; nada deveria ter se movido, e o corpo \
         está em {x}"
    );
}

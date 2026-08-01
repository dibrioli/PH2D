//! **Sensors / triggers** — ADR-0131 W7. A sensor collider passes through (no
//! contact forces) but the solver reports what overlaps it, which the bridge
//! publishes as a trigger state. These gates drive the real sim: a body falls
//! THROUGH a sensor and is detected, a solid collider blocks and never
//! triggers, a scene with no sensors reports nothing, and disarming physics
//! clears the state.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

/// A static box at `y`, either a sensor or a solid platform.
fn bar(sim: &mut SimWorld, y: f32, half_y: f32, is_sensor: bool) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y,
                },
                is_sensor,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id()
}

fn ball(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id()
}

fn run(mut sim: SimWorld, ticks: u64) -> (SimWorld, PhysicsBridge) {
    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge)
}

/// **A sensor detects a body inside it but does not block it.**
///
/// The sensor sits at `y ≈ 1.05` (spanning 0.3..1.8, clear of the floor top at
/// 0.2). The ball drops from 5, PASSES THROUGH the sensor, and rests on the
/// floor at `y ≈ 0.5` — where it still overlaps the sensor's lower edge, so at
/// rest the sensor is triggered with the ball inside it.
///
/// Mutation-tested twice over: dropping `.sensor(desc.is_sensor)` in `spawn_body`
/// turns the bar solid, so the ball rests ON it (≈ 2.1, not 0.5) AND produces a
/// contact instead of an intersection (nothing triggers); dropping the
/// `rebuild_triggers` call leaves the trigger state empty.
#[test]
fn a_sensor_detects_a_body_inside_it_but_does_not_block_it() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let sensor = bar(&mut sim, 1.05, 0.75, true);
    let b = ball(&mut sim);

    let (_, bridge) = run(sim, 300);

    let ball_y = bridge.body_pose(b).expect("ball has a body").1;
    assert!(
        (ball_y - 0.5).abs() < 0.1,
        "the ball rested at y={ball_y}; a sensor must NOT block it — it should pass \
         through and land on the floor at ≈ 0.5, not rest on the bar (≈ 2.1)"
    );
    assert!(
        bridge.is_triggered(sensor),
        "the sensor did not fire with a body inside it"
    );
    assert!(
        bridge.bodies_inside(sensor).contains(&b),
        "the ball is inside the sensor but not in its bodies_inside list: {:?}",
        bridge.bodies_inside(sensor)
    );
    assert_eq!(
        bridge.triggered_sensors(),
        vec![sensor],
        "exactly the sensor should be triggered"
    );
}

/// **A solid collider blocks the ball and never triggers** — the control that
/// gives the sensor test its meaning. Same geometry, `is_sensor = false`: the
/// ball rests ON the bar, and nothing is a trigger.
#[test]
fn a_solid_collider_blocks_and_never_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let platform = bar(&mut sim, 1.05, 0.75, false);
    let b = ball(&mut sim);

    let (_, bridge) = run(sim, 300);

    let ball_y = bridge.body_pose(b).expect("ball has a body").1;
    assert!(
        ball_y > 1.5,
        "the ball rested at y={ball_y}; a SOLID bar must block it (rest ≈ 2.1), not \
         let it fall through"
    );
    assert!(
        !bridge.is_triggered(platform),
        "a solid collider reported a trigger — a solid pair produces a contact, never \
         an intersection"
    );
    assert!(
        bridge.triggered_sensors().is_empty(),
        "a scene whose only bar is solid has a triggered sensor: {:?}",
        bridge.triggered_sensors()
    );
}

/// **A scene with no sensors reports no triggers** — the no-cost guard. The
/// `intersecting_body_pairs` query is empty, so `rebuild_triggers` returns
/// before it allocates, and a non-trigger scene pays nothing.
#[test]
fn a_scene_with_no_sensors_reports_no_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let _ = ball(&mut sim);

    let (_, bridge) = run(sim, 300);
    assert!(
        bridge.triggered_sensors().is_empty(),
        "a scene with no sensors reported triggers: {:?}",
        bridge.triggered_sensors()
    );
}

/// **Disarming physics clears the trigger state.** With the solver off (`hold`),
/// no sim runs, so a lingering "something is inside" would light a sensor that
/// nothing is inside anymore.
#[test]
fn disarming_physics_clears_the_triggers() {
    let mut sim = SimWorld::new();
    floor(&mut sim);
    let sensor = bar(&mut sim, 1.05, 0.75, true);
    let _ = ball(&mut sim);

    let mut bridge = PhysicsBridge::new();
    for t in 1..=300 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(
        bridge.is_triggered(sensor),
        "precondition: the sensor should be triggered before disarming"
    );

    bridge.hold(&mut sim, 301);
    assert!(
        !bridge.is_triggered(sensor),
        "the trigger state survived a hold (physics disarmed) — it must clear"
    );
    assert!(bridge.triggered_sensors().is_empty());
}

// ---------------------------------------------------------------------------
// W-PartSensor — o sensor é uma propriedade da FORMA, não do corpo
// ---------------------------------------------------------------------------

/// Um personagem: tronco **sólido** (o corpo) + uma peça embaixo dele, e uma
/// plataforma estática que a peça sobrepõe. `part_is_sensor` é a única diferença
/// entre os dois braços — é ele que a wave liga.
fn character(part_is_sensor: bool) -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let ground = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let torso = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.4,
                    half_y: 1.0,
                },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.5)),
        ))
        .id();
    // O pé: pendurado 1,0 abaixo do centro do tronco, ou seja MERGULHADO no chão
    // — é preciso haver sobreposição para haver o que reportar.
    let foot = sim
        .world_mut()
        .spawn((
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.2,
                },
                is_sensor: part_is_sensor,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -1.0)),
            ph2d_ecs::ChildOf(torso),
        ))
        .id();
    (sim, ground, torso, foot)
}

/// **O SENSOR DE PÉ** — o caso mais comum que existe num módulo 2D, e o que a
/// W-PartFace passou a oferecer na tela sem que ele levasse a lugar nenhum.
///
/// A peça de fato ATRAVESSA (medido: o tronco assenta em 1,4990 com ela sensora
/// contra 1,6990 com ela sólida), então o chip sempre chegou ao solver. O que
/// não chegava era o **canal**: o par reportado é `(tronco, chão)`, o teste
/// perguntava se o collider próprio do TRONCO era sensor — não é —, e a
/// sobreposição era descartada. Como o overlay é o único consumidor deste canal,
/// o efeito visível era a peça-sensor desenhada apagada **para sempre**.
#[test]
fn a_sensor_part_is_a_trigger_and_names_itself() {
    let (mut sim, ground, torso, foot) = character(true);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(
        bridge.is_triggered(foot),
        "a peça-sensor não disparou; triggered_sensors() = {:?}",
        bridge.triggered_sensors()
    );
    assert_eq!(
        bridge.triggered_sensors(),
        vec![foot],
        "quem acende tem de ser a FORMA que o artista marcou, não o corpo dela"
    );
    // ⚠️ O que está DENTRO é um CORPO. Reportar a forma faria quem pergunta
    // *"quem entrou?"* receber uma das peças de um objeto em vez do objeto.
    assert_eq!(
        bridge.bodies_inside(foot),
        &[ground],
        "o que está dentro do sensor não é o corpo que entrou"
    );
    assert!(
        !bridge.is_triggered(torso),
        "o TRONCO acendeu — o collider próprio dele é sólido, e marcar uma peça \
         não pode transformar o corpo inteiro num gatilho"
    );
}

/// **A metade oposta, e ela é o controle:** com a peça SÓLIDA nada dispara, e o
/// corpo é escorado por ela.
///
/// Sem este braço o gate acima não distinguiria *"a peça-sensor é vista"* de
/// *"qualquer peça acende o mapa"*.
#[test]
fn a_solid_part_never_triggers_and_props_the_body_up() {
    let (mut sim, _, torso, foot) = character(false);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(
        bridge.triggered_sensors().is_empty(),
        "uma peça SÓLIDA acendeu o mapa de triggers: {:?}",
        bridge.triggered_sensors()
    );
    assert!(!bridge.is_triggered(foot));
    let y = ph2d_ecs::world_transform(sim.world(), torso)
        .expect("transform")
        .translation
        .y;
    assert!(
        y > 1.6,
        "o pé sólido não escorou o tronco (y = {y:.4}) — se ele atravessa, os \
         dois braços deste par medem a mesma coisa"
    );
}

/// **Uma peça-sensor num corpo que se move continua sendo vista** — o mapa é
/// reconstruído todo dispatch, e o sensor de pé só serve se responder *enquanto*
/// o personagem anda.
///
/// O oráculo é a TRANSIÇÃO: o pé começa longe do chão (nada dentro), o corpo cai,
/// e ele passa a reportar. Um mapa construído uma vez no spawn passaria no gate
/// acima e falharia neste.
#[test]
fn a_sensor_part_lights_when_the_body_arrives_and_not_before() {
    let (mut sim, ground, torso, foot) = character(true);
    // Sobe o personagem para bem longe do chão.
    sim.world_mut()
        .get_mut::<Transform>(torso)
        .expect("transform")
        .translation
        .y = 8.0;
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 0);
    assert!(
        !bridge.is_triggered(foot),
        "o pé disparou no ar, a 8 m do chão"
    );
    for t in 1..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        bridge.bodies_inside(foot),
        &[ground],
        "o pé não reportou o chão depois de o personagem pousar"
    );
}

/// **O que está DENTRO é o corpo, mesmo quando quem entra é uma PEÇA.**
///
/// A fixture é o que torna este gate possível: em toda cena onde quem entra é um
/// corpo simples a forma e o corpo são a MESMA entidade, e reportar um ou outro
/// é indistinguível. Aqui quem cruza a barra sensora é o **pé** de um
/// personagem, e a resposta certa é o personagem.
#[test]
fn what_is_inside_is_the_body_even_when_a_part_is_what_entered() {
    let mut sim = SimWorld::new();
    // Uma barra sensora larga e alta, atravessando a queda.
    let gate = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.6,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    // Um personagem de duas formas, com o TRONCO acima da barra e só o PÉ
    // (a peça, sólida) mergulhado nela.
    //
    // ⚠️ **Dynamic, e a fixture nasceu ERRADA como Static:** rapier não reporta
    // interseção entre dois corpos fixos (o default de `ActiveCollisionTypes`
    // cobre DYNAMIC-vs-*, nunca FIXED-vs-FIXED), então o par nem chegava ao
    // grafo e o gate media um vazio. `GravityScale(0)` o deixa parado sem
    // recorrer a travas, para a pose ser a autorada e o oráculo, exato.
    let torso = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.4,
                    half_y: 0.4,
                },
                ..Collider::default()
            },
            ph2d_physics_ecs::GravityScale(0.0),
            Transform::from_translation(Vec2::new(0.0, 2.4)),
        ))
        .id();
    let foot = sim
        .world_mut()
        .spawn((
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -1.2)),
            ph2d_ecs::ChildOf(torso),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    for t in 0..=5u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        bridge.bodies_inside(gate),
        &[torso],
        "o sensor listou uma FORMA em vez do objeto que entrou (o pé é {foot:?})"
    );
}

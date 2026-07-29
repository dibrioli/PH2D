//! **A MONTAGEM de uma roldana, do lado do ECS** (W-Pulley W3) — a autoria da
//! *cadernal móvel*.
//!
//! O kernel (o Jacobiano do eixo, a massa efetiva, a vantagem mecânica medida) é
//! gateado em `ph2d-physics/tests/pulley_tackle.rs`. Aqui ficam as perguntas que
//! só existem deste lado da fronteira:
//!
//! 1. a ponte resolve o NOME do corpo e entrega o handle ao passe;
//! 2. o eixo local é semeado **UMA vez**, contra a pose de REPOUSO;
//! 3. mover o corpo em repouso **não desliza** o eixo por ele (o fix do
//!    W-AnchorFollow, uma família adiante);
//! 4. um nome que não resolve deixa a roldana no CENÁRIO, inerte e não quebrada.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// Um bloco com uma roldana montada nele e uma corda que passa por ela.
///
/// `mount` é o NOME que a roldana cita — `"Block"` monta, qualquer outra coisa é
/// um nome que não resolve.
fn rig(mount: &str) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Dead", -1.0, 6.0, BodyKind::Static);
    body("Block", 0.0, 2.0, BodyKind::Dynamic);
    body("Haul", 1.0, 4.0, BodyKind::Dynamic);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Dead"),
            body_b: stable_name_id("Haul"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Rope Wheel 1"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 0,
            radius: 0.3,
            body: stable_name_id(mount),
            ..Default::default()
        },
        // ⚠️ **Deslocado do centro do bloco de propósito**: com o eixo exatamente
        // no centro o local semeado é `[0, 0]`, que é indistinguível de *nunca
        // convertido* — a fixture não conteria o fenômeno que ela mede.
        Transform::from_translation(Vec2::new(0.25, 2.4)),
    ));
    sim
}

fn wheel_of(sim: &mut SimWorld) -> PulleyWheel {
    let mut q = sim.world_mut().query::<(&Name, &PulleyWheel)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == "Rope Wheel 1")
        .map(|(_, w)| *w)
        .expect("a roldana existe")
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn move_body(sim: &mut SimWorld, name: &str, dx: f32, dy: f32) {
    let e = entity_of(sim, name);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.x += dx;
        t.translation.y += dy;
    }
}

/// **A ponte resolve o nome e semeia o eixo local uma vez, da pose de REPOUSO.**
#[test]
fn the_bridge_mounts_the_wheel_and_seeds_the_axle_once() {
    let mut sim = rig("Block");
    assert!(
        !wheel_of(&mut sim).mounted,
        "ela nasce sem semente — é o sentinela que diz *nunca convertido*"
    );
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let w = wheel_of(&mut sim);
    assert!(w.mounted, "a ponte tinha de semear o eixo local");
    // O bloco nasce em (0, 2) e o eixo em (0.25, 2.4): o local é a diferença.
    assert!(
        (w.local[0] - 0.25).abs() < 1.0e-4 && (w.local[1] - 0.4).abs() < 1.0e-4,
        "o eixo local saiu {:?}, e a geometria diz [0.25, 0.4]",
        w.local
    );
    // E o passe recebe o handle: a arena carrega a montagem, não um `None`.
    assert!(
        bridge
            .pulley_wheel_arena()
            .first()
            .is_some_and(|w| w.body.is_some()),
        "a roldana chegou à arena SEM corpo — o nome não foi resolvido"
    );
}

/// **Mover o BLOCO não desliza o eixo por ele** — o fix do W-AnchorFollow, uma
/// família adiante.
///
/// ⚠️ Era o bug medido em **2 m** no pino do joint: um ponto de MUNDO
/// re-derivado contra a pose viva a cada reconcile caminha pelo corpo. Aqui o
/// local é lido INALTERADO e o `Transform` da roldana é derivado dele, então o
/// eixo acompanha o bloco em vez de escorregar.
#[test]
fn moving_the_block_carries_the_axle_instead_of_sliding_it() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let before = wheel_of(&mut sim).local;

    move_body(&mut sim, "Block", 2.0, -0.5);
    bridge.dispatch(&mut sim, false, 0);
    let after = wheel_of(&mut sim);
    assert!(
        (after.local[0] - before[0]).abs() < 1.0e-4 && (after.local[1] - before[1]).abs() < 1.0e-4,
        "o eixo DESLIZOU pelo bloco: {before:?} -> {:?}",
        after.local
    );
    // E o centro de desenho seguiu o bloco: eixo = bloco + local.
    let e = entity_of(&mut sim, "Rope Wheel 1");
    let t = *sim.world().get::<Transform>(e).expect("a roldana tem pose");
    assert!(
        (t.translation.x - 2.25).abs() < 1.0e-3 && (t.translation.y - 1.9).abs() < 1.0e-3,
        "o centro de desenho ficou em ({}, {}); o bloco andou para (2, 1.5) e o \
         eixo local é [0.25, 0.4]",
        t.translation.x,
        t.translation.y
    );
}

/// **Um nome que não resolve deixa a roldana no CENÁRIO** — inerte, não
/// quebrada, a mesma cura que a corda órfã e as bindings da timeline recebem.
///
/// E o `Transform` dela fica ONDE O ARTISTA O PÔS: sem corpo não há de onde
/// derivar centro nenhum, e reescrevê-lo seria a segunda porta para um fato que
/// já tem dono.
#[test]
fn an_unresolved_mount_leaves_the_wheel_in_the_scenery() {
    let mut sim = rig("Nobody");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(
        !wheel_of(&mut sim).mounted,
        "não há corpo para converter contra: nada a semear"
    );
    assert!(
        bridge
            .pulley_wheel_arena()
            .first()
            .is_some_and(|w| w.body.is_none()),
        "uma montagem que não resolve não pode chegar montada ao passe"
    );
    let e = entity_of(&mut sim, "Rope Wheel 1");
    let t = *sim.world().get::<Transform>(e).expect("a roldana tem pose");
    assert!(
        (t.translation.x - 0.25).abs() < 1.0e-6 && (t.translation.y - 2.4).abs() < 1.0e-6,
        "o centro autorado foi reescrito por uma montagem que não existe"
    );
}

//! `intersecting_body_pairs` reports a sensor overlap — ADR-0131 W7.
//!
//! The low-level half of the trigger primitive: a sensor collider passes
//! through but the narrow phase records its overlaps, and this reads them back
//! as body pairs. The ECS bridge turns those into a trigger state; here we prove
//! the wrapper reports them at all, and that a solid-only pair reports nothing.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Distância de centro a centro que põe as duas caixas **no grafo e fora do
/// toque** — o único regime onde o filtro `intersecting` é observável. Medido:
/// 2,0005 / 2,001 / 2,002 entram no grafo; 2,005 em diante já não.
const NEAR_MISS_Y: f32 = 2.001;

fn body(w: &mut PhysicsWorld, y: f32, body_type: RigidBodyType, is_sensor: bool) {
    w.spawn_body(BodyDesc {
        body_type,
        x: 0.0,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
}

/// A sensor overlapping a dynamic body is reported as one pair after a step.
/// Mutation-tested: dropping `.sensor(desc.is_sensor)` in `spawn_body` makes the
/// pair a solid contact instead of an intersection, and the pair count drops to
/// zero.
#[test]
fn a_sensor_overlap_is_reported_as_a_body_pair() {
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, true); // a static sensor at origin
    body(&mut w, 0.0, RigidBodyType::Dynamic, false); // a dynamic body inside it
    w.step();

    let pairs = w.intersecting_body_pairs();
    assert_eq!(
        pairs.len(),
        1,
        "a sensor overlapping a body should report exactly one pair, got {pairs:?}"
    );
}

/// Two SOLID overlapping bodies report NO intersection pair — a solid pair is a
/// contact, never an intersection. The control that makes the test above about
/// sensors rather than about "any overlap".
#[test]
fn a_solid_overlap_reports_no_pair() {
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, false);
    body(&mut w, 0.0, RigidBodyType::Dynamic, false);
    w.step();

    assert!(
        w.intersecting_body_pairs().is_empty(),
        "two solid bodies reported an intersection — that should be a contact"
    );
}

/// **A varredura de colliders é a PRIMITIVA, e ela pergunta se o par de fato se
/// intersecta** (W-PartSensor).
///
/// ⚠️ O gate existe porque a filtragem por `intersecting` era **invisível à
/// suíte deste wrapper**: uma mutação que a removia deixava os dois gates acima
/// VERDES e só sangrava na crate vizinha. Um filtro que só o consumidor observa
/// é um filtro que alguém remove na refatoração seguinte.
///
/// ⚠️ **A fixture tem de morar DENTRO da janela do grafo, e ela foi MEDIDA:**
/// duas caixas de meia-altura 1 entram no grafo de interseção enquanto a
/// distância entre os centros está em `(2,0 ; ~2,002]` — a margem de predição do
/// broad phase. A 2,4 (o palpite natural) o par **nem entra no grafo**, o flag
/// `intersecting` nunca é consultado, e o gate fica VERDE sob a mutação: uma
/// varredura afastada demais mede a ausência do PAR, não a do filtro.
#[test]
fn the_collider_sweep_reports_a_real_overlap_and_not_a_near_miss() {
    // Sobrepostas: um par.
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, true);
    body(&mut w, 0.0, RigidBodyType::Dynamic, false);
    w.step();
    assert_eq!(
        w.intersecting_collider_pairs().len(),
        1,
        "duas formas sobrepostas por um sensor deviam dar UM par de colliders"
    );

    // Quase-toque: nenhum.
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, true);
    body(&mut w, NEAR_MISS_Y, RigidBodyType::Fixed, false);
    w.step();
    assert!(
        w.intersecting_collider_pairs().is_empty(),
        "formas que NÃO se tocam foram reportadas: {:?}",
        w.intersecting_collider_pairs()
    );
}

/// **A projeção em corpos concorda com a primitiva** — ela é derivada, e um dia
/// em que as duas discordam é o dia em que alguém leu o grafo duas vezes.
#[test]
fn the_body_projection_agrees_with_the_collider_sweep() {
    let mut w = PhysicsWorld::new();
    body(&mut w, 0.0, RigidBodyType::Fixed, true);
    body(&mut w, 0.0, RigidBodyType::Dynamic, false);
    w.step();
    assert_eq!(
        w.intersecting_body_pairs().len(),
        w.intersecting_collider_pairs().len(),
        "cada corpo tem uma forma nesta cena, então as duas contagens têm de bater"
    );
    // E o dono de cada collider reportado é nomeável.
    for (c1, c2) in w.intersecting_collider_pairs() {
        assert!(w.collider_body(c1).is_some(), "collider sem corpo: {c1:?}");
        assert!(w.collider_body(c2).is_some(), "collider sem corpo: {c2:?}");
    }
}

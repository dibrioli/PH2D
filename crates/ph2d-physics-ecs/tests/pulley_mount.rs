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

/// Onde a arena diz que o eixo da roldana montada está.
fn arena_centre(bridge: &PhysicsBridge) -> [f32; 2] {
    bridge
        .pulley_wheel_arena()
        .first()
        .expect("a roldana está na arena")
        .centre
}

/// **Um quadro que não deve tique nenhum publica o eixo ONDE ELE ESTÁ** — o
/// tremor do gizmo que o smoke da talha reportou (W-Pulley W3).
///
/// ⚠️ **Era só DESENHO, e a forma do defeito é a razão:** `prepare` reinstala a
/// arena a cada dispatch com o centro derivado da pose de **REPOUSO** (é o que a
/// colheita do ECS conhece), e o único lugar que a punha na pose VIVA era o laço
/// de sub-passos, DENTRO do `step`. Um quadro mais rápido que o tique — que é o
/// caso normal a 60 Hz de tique com o monitor à frente — não dá passo nenhum, e
/// publicava a roldana **onde o artista a autorou**. O solver nunca leu esse
/// número: quem o lê é o pintor.
///
/// Medido com o defeito de volta: **1,27 m** de salto entre um quadro e o
/// seguinte, crescendo conforme o bloco viaja.
///
/// O oráculo é o SALTO entre os dois quadros, não um valor absoluto: o que o
/// artista vê é a roldana ir e voltar, e um limiar sobre a posição não distingue
/// *parada no lugar certo* de *parada no lugar errado*.
#[test]
fn a_frame_that_owes_no_tick_publishes_the_axle_where_it_is() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let authored = arena_centre(&bridge);

    let mut worst = 0.0_f32;
    let mut travelled = 0.0_f32;
    for t in 1..=40_u64 {
        bridge.dispatch(&mut sim, true, t);
        let stepped = arena_centre(&bridge);
        // O MESMO alvo outra vez: o quadro que chegou antes do tique seguinte.
        bridge.dispatch(&mut sim, true, t);
        let held = arena_centre(&bridge);
        worst = worst.max(dist(stepped, held));
        travelled = travelled.max(dist(authored, stepped));
    }
    // ⚠️ A fixture TEM de conter o fenômeno: sem o bloco viajar, a pose de
    // repouso e a viva coincidem e o defeito é invisível por construção.
    assert!(
        travelled > 0.5,
        "o bloco mal saiu do lugar ({travelled:.3} m): esta fixture não pode \
         provar nada sobre um eixo que ANDA"
    );
    assert!(
        worst < 1.0e-4,
        "o eixo SALTOU {worst:.4} m num quadro que não avançou tique nenhum — a \
         arena publicou o centro derivado da pose de REPOUSO"
    );
}

/// **E o eixo publicado é onde o corpo ESTÁ ao fim do tique** — a outra metade da
/// mesma pergunta, e ela sozinha não bastaria: um centro congelado no lugar certo
/// passaria pelo gate do salto e ainda estaria errado.
///
/// ⚠️ **Este gate fecha, de carona, o atraso de um SUB-PASSO que o W3 deixou
/// aberto.** O laço de sub-passos refresca os eixos ANTES de aplicar o passe (é o
/// que o solver precisa), então ao fim do `step` a arena descreve a pose do
/// começo do último sub-passo — o desenho ia atrasado. Medido antes do refresco de
/// fim de dispatch: **4,8 mm a 22,7 mm**, crescendo com a velocidade (a sonda
/// `probe_axle_lag_after_a_stepping_frame`); depois dele, **0,00000 m**.
///
/// Por isso a leitura é logo APÓS o quadro que deu tique, e não depois de um
/// quadro parado: é ali que o atraso vivia.
#[test]
fn the_published_axle_rides_the_body_not_the_authored_pose() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    for t in 1..=40_u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let published = arena_centre(&bridge);

    // O bloco, e o eixo local que a ponte semeou nele.
    let e = entity_of(&mut sim, "Block");
    let block = *sim.world().get::<Transform>(e).expect("o bloco tem pose");
    let local = wheel_of(&mut sim).local;
    let (sin, cos) = (block.rotation.sin(), block.rotation.cos());
    let expected = [
        block.translation.x + local[0] * cos - local[1] * sin,
        block.translation.y + local[0] * sin + local[1] * cos,
    ];
    assert!(
        dist(published, expected) < 1.0e-3,
        "a arena publicou {published:?}; o corpo mais o eixo local dizem {expected:?}"
    );
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// **Arrastar o bloco PAUSADO leva o eixo no MESMO quadro** — e este gate existe
/// para pinar ONDE o refresco mora.
///
/// ⚠️ Pôr a chamada junto da INSTALAÇÃO da arena (em `prepare`) cura o quadro sem
/// tique e **estraga este caso**: ali o `settle` ainda não rodou, então a pose
/// viva do corpo é a do quadro anterior e o refresco sobrescreveria o centro
/// derivado do repouso — que, pausado, é o CERTO — por um atrasado. O fim do
/// dispatch é o único ponto por onde as quatro saídas passam.
#[test]
fn dragging_a_body_while_paused_carries_the_axle_in_the_same_frame() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let before = arena_centre(&bridge);

    move_body(&mut sim, "Block", 2.0, -0.5);
    bridge.dispatch(&mut sim, false, 0);
    let after = arena_centre(&bridge);

    assert!(
        dist(after, [before[0] + 2.0, before[1] - 0.5]) < 1.0e-3,
        "o bloco andou (2, -0.5) e a arena publicou {after:?} contra {before:?} — \
         o eixo ficou um quadro atrás do corpo que o carrega"
    );
}

#[test]
#[ignore = "measurement, not a gate"]
fn probe_axle_lag_after_a_stepping_frame() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    println!("\n=== o eixo publicado x o corpo, LOGO APOS um quadro que deu tique ===");
    for t in 1..=40_u64 {
        bridge.dispatch(&mut sim, true, t);
        let published = arena_centre(&bridge);
        let e = entity_of(&mut sim, "Block");
        let b = *sim.world().get::<Transform>(e).unwrap();
        let local = wheel_of(&mut sim).local;
        let (s, c) = (b.rotation.sin(), b.rotation.cos());
        let expected = [
            b.translation.x + local[0] * c - local[1] * s,
            b.translation.y + local[0] * s + local[1] * c,
        ];
        if t % 8 == 0 {
            println!("t={t:>3} atraso = {:.5} m", dist(published, expected));
        }
    }
}

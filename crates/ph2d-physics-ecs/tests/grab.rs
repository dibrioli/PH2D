//! **A MÃO no nível da entidade** (W-Grab) — e, sobretudo, o que ela faz ao
//! DETERMINISMO.
//!
//! A física da mola vive no wrapper e tem gates lá (`ph2d-physics/tests/grab.rs`).
//! O que só existe aqui é a costura: a tradução entidade→handle, e as duas regras
//! que mantêm o invariante do scrub de pé — *o ring nunca guarda estado cutucado*
//! e *um salto de relógio solta a mão*.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Um corpo na origem-y dada, sem gravidade no mundo (o palco mede o SEGUIDOR).
fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, x: f32, y: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut bridge = PhysicsBridge::new();
    bridge.set_gravity(0.0, 0.0);
    (SimWorld::new(), bridge)
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("transform")
        .translation
        .x
}

/// Avança `ticks` a partir de `from`, um tick por dispatch (o que o play faz).
fn play(bridge: &mut PhysicsBridge, sim: &mut SimWorld, from: u64, ticks: u64) {
    for i in 1..=ticks {
        bridge.dispatch(sim, true, from + i);
    }
}

#[test]
fn the_entity_door_carries_the_body_and_the_readback_shows_it() {
    let (mut sim, mut bridge) = scene();
    let ball = body(&mut sim, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    // Um dispatch para o mundo existir (o reconcile roda no prólogo).
    bridge.dispatch(&mut sim, true, 1);
    assert!(bridge.grab(ball, [0.0, 0.0]), "a mão pega pela ENTIDADE");
    assert!(bridge.is_grabbing());
    bridge.move_grab([3.0, 0.0]);
    play(&mut bridge, &mut sim, 1, 60);
    assert!(
        x_of(&sim, ball) > 2.9,
        "o corpo tinha de chegar ao cursor, e o `Transform` tinha de mostrar: x={:.3}",
        x_of(&sim, ball)
    );
    // O CONTROLE: sem mão, o mesmo dispatch não move nada.
    let (mut sim2, mut bridge2) = scene();
    let ball2 = body(&mut sim2, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    play(&mut bridge2, &mut sim2, 0, 61);
    assert_eq!(x_of(&sim2, ball2), 0.0, "sem mão, nada se move");
}

#[test]
fn the_entity_door_refuses_what_cannot_be_held() {
    let (mut sim, mut bridge) = scene();
    let stat = body(&mut sim, "Wall", BodyKind::Static, 0.0, 0.0);
    let kin = body(&mut sim, "Lift", BodyKind::Kinematic, 3.0, 0.0);
    let bodiless = sim
        .world_mut()
        .spawn((
            Name::new("Sprite"),
            Transform::from_translation(Vec2::new(6.0, 0.0)),
        ))
        .id();
    bridge.dispatch(&mut sim, true, 1);
    assert!(!bridge.grab(stat, [0.0, 0.0]), "estático recusa");
    assert!(!bridge.grab(kin, [3.0, 0.0]), "kinematic recusa");
    assert!(!bridge.grab(bodiless, [6.0, 0.0]), "sem corpo recusa");
    assert!(!bridge.is_grabbing(), "e nada ficou pego");
}

/// **Regra 1** — pegar DESCARTA o ring, e nada é gravado enquanto a mão está lá.
///
/// O oráculo é o CUSTO do scrub: com a janela cheia o rewind semeia do checkpoint
/// mais recente e replaya o resto; com a janela vazia ele reconstrói do repouso e
/// replaya tudo. Sem esta regra a resposta para o MESMO tick dependeria de o cache
/// guardar um estado cutucado ou não — o defeito que a auditoria do W4b nomeou.
///
/// ⚠️ **O alvo do scrub é 55, e a 1ª versão deste gate usava 35 — onde a mutação
/// SOBREVIVIA.** Pegar limpa o ring de qualquer forma, então um alvo abaixo de
/// todo checkpoint da era-da-mão cai no rebuild pelos DOIS caminhos: a fixture não
/// continha o fenômeno. O tick 55 é coberto por um checkpoint que a gravação
/// proibida teria criado (o `STRIDE` grava em 50), e é só ali que as duas
/// respostas divergem.
#[test]
fn grabbing_drops_the_checkpoint_ring_and_stops_recording() {
    // Primeiro o CONTROLE: sem mão nenhuma, o scrub para 55 é barato (semeia de 50).
    let (mut sim, mut bridge) = scene();
    body(&mut sim, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    play(&mut bridge, &mut sim, 0, 60);
    let before = bridge.steps_taken();
    bridge.dispatch(&mut sim, true, 55);
    let cheap = bridge.steps_taken() - before;
    assert!(
        cheap <= 10,
        "com a janela cheia o scrub tinha de ser curto: {cheap} passos"
    );

    // E agora com a mão: mesma cena, mesmo scrub.
    let (mut sim, mut bridge) = scene();
    let ball = body(&mut sim, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    play(&mut bridge, &mut sim, 0, 40);
    assert!(bridge.grab(ball, [0.0, 0.0]));
    // Mais ticks COM a mão em voo — é aqui que a janela se re-encheria.
    bridge.move_grab([2.0, 0.0]);
    play(&mut bridge, &mut sim, 40, 20);
    let before = bridge.steps_taken();
    bridge.dispatch(&mut sim, true, 55);
    let full = bridge.steps_taken() - before;
    assert_eq!(
        full, 55,
        "sob a mão o scrub tem de replayar do repouso (55), não semear de um checkpoint cutucado"
    );
}

/// **Regra 2** — um salto de relógio SOLTA a mão.
///
/// Sem isto o replay arrastaria a tralha da mão consigo e puxaria cada tick
/// replayado na direção de onde o cursor está AGORA: a corrida nova sairia
/// diferente da que o artista autorou, e o Reset deixaria de significar *volte à
/// cena*.
///
/// ⚠️ **A mutação me corrigiu: hoje o `release_grab` do `rewind_to` é REDUNDANTE,
/// e é a Regra 1 que o torna redundante.** Removê-lo deixa este gate VERDE, porque
/// com o ring sempre vazio sob a mão todo rewind cai no `rebuild_from_rest`, que
/// constrói um `PhysicsWorld` NOVO — e a tralha da mão morre com o antigo. Ele
/// fica por ser a camada que continua correta se a Regra 1 mudar de forma (alguém
/// permitir gravação sob a mão faria o `ring.seed` restaurar para DENTRO do mundo
/// vivo, e aí a tralha sobreviveria). Defesa em camadas, dita em vez de suposta
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_rewind_lets_go_of_the_hand() {
    let (mut sim, mut bridge) = scene();
    let ball = body(&mut sim, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    play(&mut bridge, &mut sim, 0, 20);
    assert!(bridge.grab(ball, [0.0, 0.0]));
    bridge.move_grab([5.0, 0.0]);
    play(&mut bridge, &mut sim, 20, 10);
    assert!(bridge.is_grabbing(), "a fixture precisa da mão em voo");
    // Reset.
    bridge.dispatch(&mut sim, true, 0);
    assert!(!bridge.is_grabbing(), "o rewind tinha de soltar");
    assert!(
        x_of(&sim, ball).abs() < 1e-4,
        "e a cena volta à pose AUTORADA: x={:.6}",
        x_of(&sim, ball)
    );
    // E seguir tocando dali reproduz a cena autorada, sem resíduo de puxão.
    play(&mut bridge, &mut sim, 0, 30);
    assert!(
        x_of(&sim, ball).abs() < 1e-4,
        "nenhum resto da mão sobrevive ao rewind: x={:.6}",
        x_of(&sim, ball)
    );
}

/// As marcas de desenho: o cursor e o ponto de pega. O 2º é derivado da pose
/// VIVA, então ele **anda com o corpo** — se fosse memorizado, o zigzag apontaria
/// para onde o corpo estava quando foi pego.
#[test]
fn the_marks_follow_the_body_not_the_grab_moment() {
    let (mut sim, mut bridge) = scene();
    let ball = body(&mut sim, "Ball", BodyKind::Dynamic, 0.0, 0.0);
    bridge.dispatch(&mut sim, true, 1);
    assert!(bridge.grab_marks().is_none(), "sem mão, sem marca");
    assert!(bridge.grab(ball, [0.0, 0.0]));
    let (cursor0, hold0) = bridge.grab_marks().expect("marca");
    assert_eq!(cursor0, [0.0, 0.0]);
    assert_eq!(hold0, [0.0, 0.0], "no press os dois pontos coincidem");
    bridge.move_grab([3.0, 0.0]);
    // ⚠️ **UM tick, com o corpo ainda ATRÁS do cursor** — e a 1ª versão deste gate
    // media depois de 60, onde o corpo JÁ CHEGOU: ali `hold == cursor`, e a
    // mutação que devolve o cursor no lugar do ponto de pega passava. Duas
    // grandezas que têm de diferir coincidiam por FASE da fixture
    // ([[feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase]]).
    play(&mut bridge, &mut sim, 1, 1);
    let (cursor1, hold1) = bridge.grab_marks().expect("marca");
    assert_eq!(cursor1, [3.0, 0.0], "o cursor é onde a mão está");
    assert!(
        hold1[0] < 0.5,
        "o ponto de pega está no CORPO, que ainda não chegou: x={:.3}",
        hold1[0]
    );
    assert!(
        (hold1[0] - x_of(&sim, ball)).abs() < 0.01,
        "e ele é a pose VIVA do corpo: pega={:.3} corpo={:.3}",
        hold1[0],
        x_of(&sim, ball)
    );
    // E quando o corpo chega, os dois convergem — é a mola assentando.
    play(&mut bridge, &mut sim, 2, 90);
    let (_, hold2) = bridge.grab_marks().expect("marca");
    assert!(
        hold2[0] > 2.9,
        "o ponto de pega VIAJOU com o corpo até o cursor: x={:.3}",
        hold2[0]
    );
    bridge.release_grab();
    assert!(bridge.grab_marks().is_none(), "soltou, sem marca");
}

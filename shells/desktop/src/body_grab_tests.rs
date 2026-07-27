//! **A porta do press da MÃO** (W-Grab) — as três condições, headless.
//!
//! O que este arquivo pode tocar é a DECISÃO (`take_hold`), que é pura o
//! bastante para rodar sem janela: as condições 1 e 2 chegam como argumentos e a
//! 3 é do wrapper. A costura com o ponteiro real vive no arch-gate
//! `tests/the_grab_is_wired_to_the_pointer.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn scene(kind: BodyKind) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    let mut bridge = PhysicsBridge::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Body"),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    // Um dispatch para o mundo rapier existir (o reconcile roda no prólogo).
    bridge.dispatch(&mut sim, true, 1);
    (sim, bridge, e)
}

/// **Condição 1: o relógio tem de estar ANDANDO.** Em repouso o press é
/// autoria de pose (e o Alt carrega o rig — W-JG), então a mão não pode aparecer
/// lá: ela brigaria com o `settle`, que leva o corpo de volta ao `Transform`
/// autorado a cada frame parado.
#[test]
fn the_hand_only_takes_hold_while_the_clock_runs() {
    let (_sim, mut bridge, e) = scene(BodyKind::Dynamic);
    assert!(
        !crate::body_grab::take_hold(&mut bridge, e, [0.0, 0.0], false, true),
        "parado, a mão não pega"
    );
    assert!(!bridge.is_grabbing());
    assert!(
        crate::body_grab::take_hold(&mut bridge, e, [0.0, 0.0], true, true),
        "tocando, pega"
    );
    assert!(bridge.is_grabbing());
}

/// **Condição 2: a física tem de estar ARMADA.** Com o toggle `Physics` do
/// transporte desligado a ponte faz `hold` e **não dá passo nenhum**, então a
/// mola não puxaria nada: um gesto que pega e não move lê como ferramenta
/// quebrada, não como ferramenta ausente.
#[test]
fn the_hand_only_takes_hold_when_physics_is_armed() {
    let (_sim, mut bridge, e) = scene(BodyKind::Dynamic);
    assert!(
        !crate::body_grab::take_hold(&mut bridge, e, [0.0, 0.0], true, false),
        "física desarmada, a mão não pega"
    );
    assert!(!bridge.is_grabbing());
}

/// **Condição 3: só corpo DINÂMICO** — e a recusa é do wrapper, não de uma cópia
/// da regra aqui (um joint não move massa infinita). É o `false` que deixa o
/// chamador seguir com o caminho de sempre, o que mantém *selecionar* e
/// *arrastar* um cenário estático funcionando durante o play.
#[test]
fn the_hand_refuses_a_body_it_could_not_move() {
    for kind in [BodyKind::Static, BodyKind::Kinematic] {
        let (_sim, mut bridge, e) = scene(kind);
        assert!(
            !crate::body_grab::take_hold(&mut bridge, e, [0.0, 0.0], true, true),
            "{kind:?} não é pegável"
        );
        assert!(!bridge.is_grabbing());
    }
}

/// As duas condições do chamador são **E**, não **OU** — o quadrado inteiro,
/// porque uma regra que mascara a outra é como uma delas some em silêncio
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn both_caller_conditions_are_required() {
    for (playing, simulating, expect) in [
        (false, false, false),
        (false, true, false),
        (true, false, false),
        (true, true, true),
    ] {
        let (_sim, mut bridge, e) = scene(BodyKind::Dynamic);
        assert_eq!(
            crate::body_grab::take_hold(&mut bridge, e, [0.0, 0.0], playing, simulating),
            expect,
            "playing={playing} simulating={simulating}"
        );
    }
}

/// **A SONDA da cena 52** — os números que a mensagem do smoke afirma, medidos
/// sobre as MESMAS peças que o artista abre (`physics_smoke_grab::spawn_props`).
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_52 -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime os números da cena 52"]
fn probe_smoke_52() {
    let named = |sim: &mut SimWorld, want: &str| -> Entity {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == want)
            .map(|(e, _)| e)
            .expect("entidade da cena")
    };
    let x_of = |sim: &SimWorld, e: Entity| -> f32 { at_of(sim, e)[0] };
    // ⚠️ A pega é na pose VIVA, não na de spawn: os corpos assentam antes do
    // gesto, e pegar na altura de spawn seria pegar o ar acima do corpo (um
    // ponto rigidamente ligado a ele, como um cabo) — o artista clica NO corpo.
    fn at_of(sim: &SimWorld, e: Entity) -> [f32; 2] {
        let t = sim.world().get::<Transform>(e).expect("transform");
        [t.translation.x, t.translation.y]
    }
    let stage = || {
        let mut sim = SimWorld::new();
        let mut bridge = PhysicsBridge::new();
        crate::physics_smoke_grab::spawn_props(sim.world_mut());
        // Um segundo de assentamento antes de qualquer gesto (a torre encosta).
        for t in 1..=60 {
            bridge.dispatch(&mut sim, true, t);
        }
        (sim, bridge, 60_u64)
    };

    // 1. A DUPLA: o MESMO gesto (levar a mão 3 m para a direita em meio segundo,
    //    depois segurar) em cada corpo.
    let mut travelled = Vec::new();
    for who in ["Light Ball", "Heavy Crate"] {
        let (mut sim, mut bridge, mut t) = stage();
        let e = named(&mut sim, who);
        let at = at_of(&sim, e);
        let x0 = at[0];
        assert!(bridge.grab(e, at));
        for i in 1..=90 {
            let f = (f32::from(u16::try_from(i.min(30)).unwrap())) / 30.0;
            bridge.move_grab([x0 + 3.0 * f, at[1]]);
            t += 1;
            bridge.dispatch(&mut sim, true, t);
        }
        travelled.push(x_of(&sim, e) - x0);
    }
    println!(
        "  DUPLA: leve andou {:.2} m, pesado andou {:.2} m -- razao {:.3}",
        travelled[0],
        travelled[1],
        travelled[0] / travelled[1]
    );

    // 2. A PAREDE: puxar o 'Pusher' para x=-3, atravessando o muro em x=-6.
    let (mut sim, mut bridge, mut t) = stage();
    let pusher = named(&mut sim, "Pusher");
    let at = at_of(&sim, pusher);
    assert!(bridge.grab(pusher, at));
    for _ in 0..90 {
        bridge.move_grab([-3.0, at[1]]);
        t += 1;
        bridge.dispatch(&mut sim, true, t);
    }
    println!(
        "  PAREDE: o cursor foi para x=-3,0 e o caixote parou em x={:.2}",
        x_of(&sim, pusher)
    );

    // 3. O ARREMESSO: mão a 8 m/s e solta.
    let (mut sim, mut bridge, mut t) = stage();
    let ball = named(&mut sim, "Light Ball");
    let at = at_of(&sim, ball);
    let x0 = at[0];
    assert!(bridge.grab(ball, at));
    for i in 1..=30 {
        let d = f32::from(u16::try_from(i).unwrap()) * 8.0 / 60.0;
        bridge.move_grab([x0 + d, at[1]]);
        t += 1;
        bridge.dispatch(&mut sim, true, t);
    }
    let at_release = x_of(&sim, ball);
    bridge.release_grab();
    for _ in 0..60 {
        t += 1;
        bridge.dispatch(&mut sim, true, t);
    }
    println!(
        "  ARREMESSO: soltou em x={:.2} e a bola viajou {:.2} m depois do release",
        at_release,
        x_of(&sim, ball) - at_release
    );
}

//! **A porta do press da MÃO** (W-Grab) — as três condições, headless.
//!
//! O que este arquivo pode tocar é a DECISÃO (`take_hold`), que é pura o
//! bastante para rodar sem janela: as condições 1 e 2 chegam como argumentos e a
//! 3 é do wrapper. A costura com o ponteiro real vive no arch-gate
//! `tests/the_grab_is_wired_to_the_pointer.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InteractionSettings, InteractionTool, PhysicsBridge,
    RigidBody,
};

/// A ferramenta na configuração DEFAULT — a mão. As portas tomam as settings
/// agora, e é elas que decidem de qual família a ferramenta é.
fn hand() -> InteractionSettings {
    InteractionSettings::default()
}

/// A ferramenta armada num modo de PONTO (o estouro), para as recusas cruzadas.
fn blast() -> InteractionSettings {
    InteractionSettings {
        tool: InteractionTool::Explode,
        ..InteractionSettings::default()
    }
}

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
        !crate::body_grab::take_hold(&mut bridge, &hand(), e, [0.0, 0.0], false, true),
        "parado, a mão não pega"
    );
    assert!(!bridge.is_grabbing());
    assert!(
        crate::body_grab::take_hold(&mut bridge, &hand(), e, [0.0, 0.0], true, true),
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
        !crate::body_grab::take_hold(&mut bridge, &hand(), e, [0.0, 0.0], true, false),
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
            !crate::body_grab::take_hold(&mut bridge, &hand(), e, [0.0, 0.0], true, true),
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
            crate::body_grab::take_hold(&mut bridge, &hand(), e, [0.0, 0.0], playing, simulating),
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

// ── As ferramentas de PONTO (W-Hand) ────────────────────────────────────────

/// **As duas famílias são EXCLUSIVAS, e a porta de cada uma recusa a outra.**
///
/// É a metade estrutural desta wave: a mão pendura no pick de canvas (para a
/// seleção seguir acontecendo) e as de ponto são interceptadas ANTES dele. Se as
/// duas portas aceitassem a mesma ferramenta, um press com o estouro em mãos
/// dispararia o estouro **e** pegaria o corpo debaixo dele.
#[test]
fn the_two_tool_families_refuse_each_other() {
    let (_sim, mut bridge, e) = scene(BodyKind::Dynamic);
    // A mão recusa quando a ferramenta é de ponto.
    assert!(
        !crate::body_grab::take_hold(&mut bridge, &blast(), e, [0.0, 0.0], true, true),
        "a mão pegou com o ESTOURO em mãos"
    );
    assert!(!bridge.is_grabbing());
    // E a porta de ponto recusa quando a ferramenta é a mão.
    assert!(
        crate::body_grab::poke_at(&mut bridge, &hand(), [0.0, 0.0], true, true).is_none(),
        "a porta de ponto consumiu o press com a MÃO em mãos"
    );
    assert!(!bridge.is_poking());
}

/// **O estouro e o campo honram as MESMAS duas condições de chamador que a mão.**
///
/// Sem passo não há força, então um gesto oferecido com o relógio parado (ou com
/// a física desarmada) é um clique que não faz nada — a assinatura de *"a
/// ferramenta está quebrada"* em vez de *"a ferramenta não está aqui"*.
#[test]
fn the_point_tools_need_the_clock_and_the_toggle() {
    for tool in [InteractionTool::Explode, InteractionTool::Attract] {
        let settings = InteractionSettings {
            tool,
            ..InteractionSettings::default()
        };
        let (_sim, mut bridge, _e) = scene(BodyKind::Dynamic);
        assert!(
            crate::body_grab::poke_at(&mut bridge, &settings, [0.0, 0.0], false, true).is_none(),
            "{tool:?} disparou com o relógio parado"
        );
        assert!(
            crate::body_grab::poke_at(&mut bridge, &settings, [0.0, 0.0], true, false).is_none(),
            "{tool:?} disparou com a física desarmada"
        );
        assert!(
            crate::body_grab::poke_at(&mut bridge, &settings, [0.0, 0.0], true, true).is_some(),
            "{tool:?} não disparou com as duas condições satisfeitas"
        );
    }
}

/// **A porta de ponto faz o que a ferramenta escolhida diz** — o estouro conta
/// corpos e NÃO deixa campo armado; a atração arma um campo e não estoura nada.
///
/// Um gate que só pedisse `Some(_)` das duas passaria com as duas ligadas no
/// mesmo braço, que é a regressão mais fácil deste `match`.
#[test]
fn each_point_tool_does_its_own_thing() {
    let (_sim, mut bridge, _e) = scene(BodyKind::Dynamic);
    let hit = crate::body_grab::poke_at(&mut bridge, &blast(), [0.0, 0.0], true, true);
    assert_eq!(hit, Some(1), "o estouro não contou o corpo sob ele");
    assert!(
        !bridge.is_poking(),
        "o estouro deixou um cutucão SUSTENTADO em voo — ele é um impulso"
    );

    let (_sim, mut bridge, _e) = scene(BodyKind::Dynamic);
    let pull = InteractionSettings {
        tool: InteractionTool::Attract,
        ..InteractionSettings::default()
    };
    assert!(crate::body_grab::poke_at(&mut bridge, &pull, [0.0, 0.0], true, true).is_some());
    assert!(
        bridge.attract_marks().is_some(),
        "a atração não armou campo nenhum"
    );
}

/// **O flash do estouro envelhece e some.** Um canal próprio, então nada no mundo
/// o apagaria por conta — sem o passo de envelhecimento a marca fica na tela para
/// sempre, descrevendo um estouro de dez minutos atrás.
#[test]
fn the_blast_flash_ages_out() {
    let mut flash = Some(([1.0_f32, 2.0], 3.0_f32, crate::body_grab::BLAST_FLASH_TICKS));
    for _ in 0..crate::body_grab::BLAST_FLASH_TICKS - 1 {
        crate::body_grab::age_blast_flash(&mut flash);
        assert!(flash.is_some(), "o flash morreu cedo demais");
    }
    crate::body_grab::age_blast_flash(&mut flash);
    assert!(flash.is_none(), "o flash sobreviveu à própria vida");
}

//! **A REAÇÃO** (W6) — a 3ª lei, com o rapier de verdade.
//!
//! A pergunta desta wave não é sobre o personagem: é sobre **o que ele faz ao
//! mundo**. Uma cápsula flutuante que só empurra a si mesma é um fantasma, e o
//! oráculo de um fantasma é o corpo que ele deveria ter movido.
//!
//! # ⚠️ Por que a jangada não tem gravidade
//!
//! `GravityScale(0)` nela é o que torna a medição ATRIBUÍVEL: sem isso a
//! jangada cai por conta própria e separar *"afundou porque o personagem pesa"*
//! de *"afundou porque tudo cai"* viraria uma subtração de dois números grandes.
//! Com gravidade zero, **todo milímetro que ela anda é do personagem**.
//!
//! ⚠️ E ela **não** leva `LockRotation` — os corpos desta suíte travam a rotação
//! por desenho (é o que se faz com um personagem 2D), e travá-la na jangada
//! apagaria justamente o torque que a metade INCLINAR mede.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, LockRotation, PhysicsBridge, PlatformPlayer,
    PlayerInput, RigidBody,
};

/// A altura de flutuação — a mesma das outras fixtures, pelo mesmo motivo
/// geométrico (ver `platform_scene.rs`).
const FLOAT: f32 = 0.9;

/// Uma jangada DINÂMICA sem peso próprio, e um player em cima dela.
///
/// `support`/`movement` são os dois escalares da 3ª lei; `at_x` é onde o
/// personagem pousa em relação ao centro da jangada — é ele que decide se a
/// pergunta é *afunda?* ou *inclina?*.
fn raft(support: f32, movement: f32, at_x: f32) -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    let raft = sim
        .world_mut()
        .spawn((
            Name::new("Raft"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 3.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                reaction_support: support,
                reaction_movement: movement,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(at_x, 0.25 + FLOAT)),
        ))
        .id();
    (sim, PhysicsBridge::new(), raft, player)
}

fn pose_of(sim: &SimWorld, who: &str) -> (f32, f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some((t.translation.x, t.translation.y, t.rotation));
        }
    }
    found.unwrap_or_else(|| panic!("{who} tem de existir"))
}

/// Roda `ticks` com o drive dado e devolve a pose da jangada.
fn run(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    player: Entity,
    drive: f32,
    ticks: u64,
) -> (f32, f32, f32) {
    for t in 1..=ticks {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(sim, true, t);
    }
    pose_of(sim, "Raft")
}

/// **A JANGADA AFUNDA** — o gate da wave.
///
/// ⚠️ O oráculo é a jangada, nunca o personagem: com a reação ligada ele desce
/// junto (a perna o mantém à altura de flutuação sobre um chão que baixou), e
/// medir o personagem confundiria *"a jangada cedeu"* com *"a mola relaxou"*.
#[test]
fn the_raft_sinks_under_the_players_weight() {
    let sunk = {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
        run(&mut sim, &mut bridge, player, 0.0, 90).1
    };
    let ghost = {
        let (mut sim, mut bridge, _raft, player) = raft(0.0, 0.0, 0.0);
        run(&mut sim, &mut bridge, player, 0.0, 90).1
    };
    eprintln!("jangada com reacao {sunk:.4} m · fantasma {ghost:.4} m");
    assert!(
        ghost.abs() < 1.0e-4,
        "sem reacao a jangada nao pode se mexer: {ghost:.6}"
    );
    assert!(
        sunk < -0.2,
        "com reacao ela tem de AFUNDAR: {sunk:.4} m contra {ghost:.4}"
    );
}

/// **E ela INCLINA** — o torque, que é o que separa `apply_impulse_at_point` de
/// `apply_impulse`.
///
/// ⚠️ O oráculo é a comparação entre pousar no CENTRO e pousar na BORDA, e não
/// um ângulo absoluto: no centro o braço é zero e o torque tem de ser zero por
/// GEOMETRIA, então o par afirma a lei em vez de um número.
#[test]
fn standing_on_the_edge_tips_the_raft_and_the_centre_does_not() {
    let centre = {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
        run(&mut sim, &mut bridge, player, 0.0, 90).2
    };
    let edge = {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 2.2);
        run(&mut sim, &mut bridge, player, 0.0, 90).2
    };
    eprintln!("inclinacao no centro {centre:.4} rad · na borda {edge:.4} rad");
    assert!(
        centre.abs() < 0.02,
        "no centro o braco e' zero, logo o torque tambem: {centre:.4} rad"
    );
    assert!(
        edge.abs() > 0.1,
        "na borda ela tem de INCLINAR: {edge:.4} rad"
    );
}

/// ⚠️ **O TAPETE fica quieto no default, e o número o liga.**
///
/// Andar sobre uma plataforma não pode empurrá-la para trás — é atrito honesto
/// e péssimo de jogar, e por isso `reaction_movement` nasce em zero. O gate tem
/// as duas metades porque só a primeira seria satisfeita por um escalar morto.
#[test]
fn walking_does_not_shove_the_platform_until_the_scale_says_so() {
    let quiet = {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
        run(&mut sim, &mut bridge, player, 1.0, 60).0
    };
    let carpet = {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 1.0, 0.0);
        run(&mut sim, &mut bridge, player, 1.0, 60).0
    };
    eprintln!("deriva lateral: default {quiet:.4} m · com o tapete ligado {carpet:.4} m");
    assert!(
        quiet.abs() < 0.05,
        "no default a plataforma nao escorrega: {quiet:.4} m"
    );
    assert!(
        carpet < -0.15,
        "com o escalar ligado ela e' empurrada para TRAS: {carpet:.4} m"
    );
}

/// **PULAR de uma jangada a empurra para baixo** — a decolagem é contato.
///
/// ⚠️ E a gravidade de FASE do arco não é: se ela vazasse para a reação, o
/// personagem empurraria a jangada durante a subida inteira, **sem a estar
/// tocando**. O oráculo separa os dois porque mede a jangada no tick seguinte à
/// decolagem e de novo bem depois, quando o personagem está longe.
#[test]
fn jumping_off_the_raft_shoves_it_down_and_the_arc_does_not() {
    let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
    // Assenta.
    for t in 1..=60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let settled = pose_of(&sim, "Raft").1;

    // Um tick com o botão: a decolagem.
    bridge.set_player_input(
        player,
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        },
    );
    bridge.dispatch(&mut sim, true, 61);
    let kicked = pose_of(&sim, "Raft").1;

    // E agora o arco inteiro, com o personagem subindo. Três leituras igualmente
    // espaçadas: elas é que separam INÉRCIA de EMPURRÃO.
    let mut marks = [kicked; 3];
    for (i, mark) in marks.iter_mut().enumerate() {
        for t in 0..12 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, 62 + (i as u64) * 12 + t);
        }
        *mark = pose_of(&sim, "Raft").1;
    }

    eprintln!(
        "jangada: assentada {settled:.4} · pos-chute {kicked:.4} · no ar {:.4} {:.4} {:.4}",
        marks[0], marks[1], marks[2]
    );
    assert!(
        kicked < settled - 0.005,
        "a decolagem tem de empurrar a jangada para BAIXO: {kicked:.4} contra {settled:.4}"
    );

    // ⚠️ **O oráculo é a SEGUNDA DIFERENÇA, e é a metade que importa.** A
    // jangada não tem gravidade e nada mais a toca, então o que o chute lhe deu
    // é velocidade CONSTANTE — os três intervalos têm de medir o mesmo. Se a
    // gravidade de fase do arco vazasse para a reação, ela seria uma
    // ACELERAÇÃO, e os intervalos cresceriam. Uma barra sobre a distância
    // percorrida nao distinguiria os dois casos.
    let d1 = marks[1] - marks[0];
    let d2 = marks[2] - marks[1];
    eprintln!(
        "intervalos iguais: {d1:.5} e {d2:.5} (segunda diferenca {:.5})",
        d2 - d1
    );
    assert!(
        (d2 - d1).abs() < 0.02,
        "a jangada tem de DERIVAR, nao acelerar: os intervalos foram {d1:.5} e {d2:.5} \
         -- a gravidade de fase esta vazando para a reacao"
    );
}

/// ⚠️ **A gravidade de FASE não empurra a jangada** — e a fixture é um PULINHO,
/// de propósito.
///
/// A gravidade de fase é uma ficção aplicada ao personagem no ar; devolvê-la ao
/// chão seria ele empurrar uma plataforma que não está tocando. Mas com o pulo
/// de fábrica ela é **inobservável**: a subida corre em `takeoff_gravity = 1.0`
/// (extra exatamente zero) e o personagem sai do alcance do raio (1,4 m) muito
/// antes do ápice, então o vazamento nunca coincide com haver chão em que
/// empurrar — a primeira versão deste gate ficou VERDE com o vazamento
/// instalado.
///
/// Com `jump_height = 0.1` o `v0` sai em ~1,4 m/s, **abaixo** do `peak_speed`,
/// então o personagem entra no ramo do ÁPICE já no primeiro tick e sobe uns
/// poucos centímetros — dentro do alcance o tempo todo. É a janela em que as
/// duas coisas são verdadeiras ao mesmo tempo.
///
/// O oráculo é a **velocidade constante** da jangada depois do chute: sem
/// gravidade própria e sem ninguém a tocar, ela deriva. Um vazamento é uma
/// ACELERAÇÃO.
#[test]
fn the_arcs_phase_gravity_never_reaches_the_raft() {
    let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.jump_height = 0.1;
    }
    for t in 1..=60 {
        bridge.dispatch(&mut sim, true, t);
    }
    // O tick da decolagem, e depois o pulinho inteiro dentro do alcance.
    let mut marks = [0.0_f32; 4];
    for (i, mark) in marks.iter_mut().enumerate() {
        for t in 0..4 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, 61 + (i as u64) * 4 + t);
        }
        *mark = pose_of(&sim, "Raft").1;
    }
    let d = [
        marks[1] - marks[0],
        marks[2] - marks[1],
        marks[3] - marks[2],
    ];
    eprintln!("pulinho: marcos {marks:?} · intervalos {d:?}");
    // ⚠️ **A barra saiu da MEDIÇÃO dos dois lados, e é apertada de propósito.**
    // Com o produto correto os três intervalos são o MESMO `f32` (segunda
    // diferença exatamente 0,000000): a jangada deriva, e derivar é uma
    // igualdade, não uma aproximação. Com o vazamento instalado eles crescem
    // 0,0026 por passo. Uma barra de 0,01 — a primeira que escrevi — deixava os
    // dois lados passarem.
    const MAX_SECOND_DIFFERENCE: f32 = 0.001;
    assert!(
        (d[1] - d[0]).abs() < MAX_SECOND_DIFFERENCE && (d[2] - d[1]).abs() < MAX_SECOND_DIFFERENCE,
        "depois do chute a jangada DERIVA; intervalos crescendo sao a gravidade \
         de fase vazando: {d:?}"
    );
}

/// ⚠️ **Sobre chão ESTÁTICO a reação não custa nada** — a regressão da wave.
///
/// Um corpo estático tem massa infinita e absorve sem se mexer, então ligar ou
/// desligar a 3ª lei tem de dar a MESMA trajetória do personagem, ao bit. É o
/// que torna esta wave uma adição em vez de uma mudança no que já foi smokado.
#[test]
fn on_static_ground_the_reaction_changes_nothing_at_all() {
    fn walk_on_rock(support: f32, movement: f32) -> (f32, f32) {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
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
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ));
        let player = sim
            .world_mut()
            .spawn((
                Name::new("Player"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    reaction_support: support,
                    reaction_movement: movement,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(0.0, 0.5 + FLOAT)),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        for t in 1..=120 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive: 1.0,
                    jump: t > 60,
                    down: false,
                },
            );
            bridge.dispatch(&mut sim, true, t);
        }
        let (x, y, _) = pose_of(&sim, "Player");
        (x, y)
    }
    let on = walk_on_rock(1.0, 1.0);
    let off = walk_on_rock(0.0, 0.0);
    eprintln!("no chao estatico: com reacao {on:?} · sem {off:?}");
    assert_eq!(
        on, off,
        "sobre rocha a 3a lei tem de ser BYTE-IDENTICA a nao existir"
    );
}

/// ⚠️ **UM PLAYER PESADO AFUNDA MAIS** — a MAGNITUDE da 3ª lei.
///
/// As metades de *direção* e de *ponto* já têm gates; esta é a de **quanto**, e
/// ela nasceu de uma mutação que sobreviveu: tirar a massa da conversão
/// (`a·m·dt` → `a·dt`) deixa a jangada afundando com força independente de quem
/// está em cima, e todos os outros gates continuam verdes porque ela afunda de
/// qualquer jeito.
///
/// ⚠️ **O oráculo é DEPENDÊNCIA, não proporção**, e o número diz por quê: 0,5 →
/// 1 → 2 → 4 kg dão −0,171 / −0,300 / −0,481 / −0,690, ou seja **sublinear**. A
/// jangada afunda enquanto o personagem a acompanha, a mola re-equilibra e o
/// peso transmitido cai junto — um gate que exigisse o dobro para o dobro
/// estaria a afirmar uma física que este sistema acoplado não tem.
#[test]
fn a_heavier_player_sinks_the_raft_further() {
    fn sink_with(mass: f32) -> f32 {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
        {
            let mut e = sim.world_mut().entity_mut(player);
            e.insert(ph2d_physics_ecs::MassOverride(mass));
        }
        run(&mut sim, &mut bridge, player, 0.0, 30).1
    }
    let light = sink_with(0.5);
    let heavy = sink_with(4.0);
    eprintln!("afundamento: leve (0,5 kg) {light:.4} m · pesado (4 kg) {heavy:.4} m");
    assert!(
        heavy < light * 2.0,
        "o peso tem de entrar na conta: leve {light:.4} contra pesado {heavy:.4}"
    );
}

/// ⚠️ **UMA JANGADA ADORMECIDA ACORDA quando alguém pisa nela.**
///
/// Um corpo dormindo não é integrado pelo rapier, então uma plataforma parada há
/// tempo simplesmente **ignoraria** a reação — ela só afundaria quando alguma
/// outra coisa esbarrasse nela, que é a assinatura de *"a física parou de
/// funcionar"* que o `move_grab` já mediu uma vez.
///
/// ⚠️ **A fixture tem de deixar a jangada DORMIR de verdade**: 400 ticks com a
/// reação desligada. Sem essa espera o corpo nunca adormece, o `wake_up` não tem
/// o que acordar, e o gate fica verde sem tocar no assunto — foi o que aconteceu
/// com a primeira mutação desta wave.
#[test]
fn a_sleeping_raft_wakes_when_the_player_leans_on_it() {
    let (mut sim, mut bridge, _raft, player) = raft(0.0, 0.0, 0.0);
    for t in 1..=400 {
        bridge.dispatch(&mut sim, true, t);
    }
    let asleep = pose_of(&sim, "Raft").1;
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.reaction_support = 1.0;
    }
    for t in 401..=460 {
        bridge.dispatch(&mut sim, true, t);
    }
    let woken = pose_of(&sim, "Raft").1;
    eprintln!("jangada adormecida {asleep:.4} m · 60 ticks depois de ligar a reacao {woken:.4} m");
    assert!(
        asleep.abs() < 1.0e-4,
        "ela tem de estar parada (e dormindo) antes: {asleep:.6}"
    );
    assert!(woken < -0.2, "e tem de ACORDAR e afundar: {woken:.4} m");
}

/// **SONDA** — a proporcionalidade da 3ª lei e o despertar da jangada.
///
/// `cargo test -p ph2d-physics-ecs --test platform_raft measure_the_third_law -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_third_law() {
    eprintln!("massa do player | afundamento em 30 ticks");
    for mass in [0.5_f32, 1.0, 2.0, 4.0] {
        let (mut sim, mut bridge, _raft, player) = raft(1.0, 0.0, 0.0);
        {
            let mut e = sim.world_mut().entity_mut(player);
            e.insert(ph2d_physics_ecs::MassOverride(mass));
        }
        let y = run(&mut sim, &mut bridge, player, 0.0, 30).1;
        eprintln!("{mass:>15.1} | {y:>22.4}");
    }

    eprintln!("\n--- a jangada ADORMECIDA ---");
    let (mut sim, mut bridge, _raft, player) = raft(0.0, 0.0, 0.0);
    for t in 1..=400 {
        bridge.dispatch(&mut sim, true, t);
    }
    let asleep = pose_of(&sim, "Raft").1;
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.reaction_support = 1.0;
    }
    for t in 401..=460 {
        bridge.dispatch(&mut sim, true, t);
    }
    let woken = pose_of(&sim, "Raft").1;
    eprintln!("antes de ligar a reacao {asleep:.4} · 60 ticks depois {woken:.4}");
}

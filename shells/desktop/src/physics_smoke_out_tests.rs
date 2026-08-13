//! **A cena 105 dirigida HEADLESS** — os números da mensagem saem daqui, e não
//! do olho.
//!
//! ⚠️ **A política do plano exige isto:** duas cenas desta linha já afirmaram
//! coisas que a medição desmentiu, então o roteiro só é escrito depois de a
//! sonda correr.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlayerEvent, PlayerInput};

use super::build;

fn rig() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    let bits = build(sim.world_mut());
    (sim, PhysicsBridge::new(), Entity::from_bits(bits))
}

/// **O sujeito nasce a publicar, e as quatro capacidades estão ARMADAS.**
///
/// ⚠️ Cada uma delas nasce DESLIGADA no app, então uma cena que as deixasse
/// assim julgaria o silêncio — e o gate ficaria verde sobre um roteiro cujos
/// passos 3 e 4 não podem acontecer.
#[test]
fn the_scene_arms_everything_the_script_asks_for() {
    let (sim, _, p) = rig();
    assert!(
        sim.world()
            .get::<ph2d_physics_ecs::PlayerSignals>(p)
            .is_some(),
        "a cena e' a unica que arma o opt-in"
    );
    let cfg = sim
        .world()
        .get::<ph2d_physics_ecs::PlatformPlayer>(p)
        .copied()
        .expect("o sujeito e' um player");
    assert!(cfg.wall_slide_speed > 0.0, "a parede");
    assert!(cfg.dash_speed > 0.0, "o arranque");
    assert!(cfg.ledge_grab > 0.0, "a beirada");
    assert!(cfg.jump_height > 0.0, "o pulo, que vem do ponto de partida");
}

/// **O passo 2 do roteiro acontece de facto:** correndo para a direita e
/// saltando, o canal entrega o pulo do CHÃO e depois a aterragem.
///
/// ⚠️ **A ORDEM é o oráculo**, e não a mera presença: um canal que publicasse
/// tudo no fim do dispatch daria os mesmos dois nomes na ordem errada, e é a
/// ordem que um consumidor de som consome.
#[test]
fn the_run_and_the_jump_land_in_the_scripted_order() {
    let (mut sim, mut bridge, p) = rig();
    let mut seen: Vec<PlayerEvent> = Vec::new();
    let mut tick = 0u64;

    // Assenta na doca.
    for _ in 0..30 {
        tick += 1;
        bridge.set_player_input(p, PlayerInput::default());
        bridge.dispatch(&mut sim, true, tick);
    }
    // Corre para a direita e salta ao chegar perto do vão.
    for i in 0..240u64 {
        tick += 1;
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: 1.0,
                jump: i % 40 == 0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        seen.extend(bridge.player_events().iter().map(|(_, e)| *e));
    }

    let first_jump = seen.iter().position(|e| {
        matches!(
            e,
            PlayerEvent::Jumped {
                kind: ph2d_physics_ecs::JumpKind::Ground
            }
        )
    });
    let first_land = seen
        .iter()
        .position(|e| matches!(e, PlayerEvent::Landed { .. }));
    let (j, l) = (
        first_jump.expect("o roteiro manda saltar, e o canal tem de o dizer"),
        first_land.expect("e aterrar"),
    );
    assert!(
        j < l,
        "salta ANTES de aterrar — a ordem e' o que um consumidor consome: {seen:?}"
    );
}

/// **O readout que o log imprime existe** — e é o que torna o passo 0 do
/// roteiro uma instrução em vez de uma esperança.
#[test]
fn the_readout_the_script_promises_is_published() {
    let (mut sim, mut bridge, p) = rig();
    for t in 1..=30u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let v = bridge
        .player_view(p)
        .copied()
        .expect("o passo 0 promete uma linha por meio segundo");
    assert_eq!(
        v.footing,
        ph2d_physics_ecs::FootingKind::Ground,
        "assentado na doca ele esta' no chao"
    );
    assert_eq!(v.facing, 1.0, "ele nasce virado para a direita");
}

/// **Os cinco nomes que o roteiro promete SAEM do percurso.**
///
/// ⚠️ **É o gate que a política do plano exige**, e ele nasceu para consertar o
/// roteiro e não para o confirmar: dois arranjos anteriores da cena não davam
/// pulo de parede NENHUM (colada à ponta do degrau ela apenas o BLOQUEIA, e
/// atrás do início ele chega a ela pelo CHÃO, onde a lei recusa agarrar-se). Sem
/// isto o passo 3 seria uma promessa que a cena não cumpre.
///
/// A censura medida do percurso: `apex ×20 · landed ×10 · jumped.ground ×8 ·
/// jumped.wall ×10 · ledge_grabbed ×2 · dashed ×1`.
#[test]
fn the_course_fires_every_name_the_script_promises() {
    let (mut sim, mut bridge, p) = rig();
    let mut names: Vec<String> = Vec::new();
    let mut tick = 0u64;
    for _ in 0..30 {
        tick += 1;
        bridge.set_player_input(p, PlayerInput::default());
        bridge.dispatch(&mut sim, true, tick);
    }
    for i in 0..900u64 {
        tick += 1;
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: 1.0,
                jump: i % 25 == 0,
                dash: i == 400,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        names.extend(bridge.signal_events(&sim).into_iter().map(|s| s.name));
    }
    for want in [
        "player.jumped.ground",
        "player.landed",
        "player.jumped.wall",
        "player.ledge_grabbed",
        "player.dashed",
    ] {
        assert!(
            names.iter().any(|n| n == want),
            "o roteiro promete {want} e o percurso nao o produziu: {names:?}"
        );
    }
}

/// **A SONDA que escreveu o roteiro** — dirige o percurso inteiro e imprime
/// cada sinal com o tique em que ele saiu.
///
/// `cargo test -p ph2d-host-desktop --bins the_course_names_what_it_fires -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn the_course_names_what_it_fires() {
    let (mut sim, mut bridge, p) = rig();
    sim.world_mut()
        .entity_mut(p)
        .insert(ph2d_physics_ecs::PlayerSignals);
    let mut tick = 0u64;
    for _ in 0..30 {
        tick += 1;
        bridge.set_player_input(p, PlayerInput::default());
        bridge.dispatch(&mut sim, true, tick);
    }
    for i in 0..900u64 {
        tick += 1;
        let x = sim
            .world()
            .get::<ph2d_ecs::Transform>(p)
            .map_or(0.0, |t| t.translation.x);
        bridge.set_player_input(
            p,
            PlayerInput {
                // A perna ESQUERDA do percurso primeiro (a parede), depois a
                // direita (o vão e o degrau) — é o roteiro, na ordem dele.
                drive: 1.0,
                jump: i % 25 == 0,
                dash: i == 400,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        for s in bridge.signal_events(&sim) {
            if s.name.starts_with("player.") {
                eprintln!("  t={tick:4}  x={x:6.2}  {}", s.name);
            }
        }
    }
    let x = sim
        .world()
        .get::<ph2d_ecs::Transform>(p)
        .map_or(0.0, |t| t.translation.x);
    eprintln!("  fim: x = {x:.2}");
}

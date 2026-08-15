//! **O EMPURRÃO DE FORA, medido antes de decidir a forma** (`W-Launch`,
//! plano 10 §5).
//!
//! Três perguntas, e a terceira decide o desenho da wave inteira:
//!
//! 1. **O que uma explosão faz a um player, hoje, em cada um dos três modos?** O
//!    plano afirma *"sob Spring o `walk` RESISTE; sob Snap/Pure não faz nada"* —
//!    e uma premissa herdada de outro documento não é uma medição desta wave.
//! 2. **Quanto tempo o efeito dura?** Se o `walk` come o empurrão no tique
//!    seguinte, a cura não é *"entregar velocidade"*, é **calar o controlo
//!    aéreo** por uma janela — e o primitivo para isso já existe
//!    (`wall_jump_lockout`).
//! 3. **E o jogador a segurar a direção CONTRÁRIA muda a resposta?** É esse o
//!    caso em que um empurrão *"que funciona"* desaparece na mão de quem joga.
//!
//! Rode: `cargo test -p ph2d-physics-ecs --test measure_launch --release
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, PlayerInput, PlayerMode, RigidBody};
use scene_fixture::{pose, scene};

/// A força da explosão, em impulso do rapier.
///
/// ⚠️ **MEDIDO, não escolhido:** a primeira leitura usou `40` e mandou o corpo
/// **24,3 m** no primeiro meio segundo — a coluna não descrevia um empurrão,
/// descrevia um projéctil. Com `8` o deslocamento fica na ordem de grandeza de
/// um passo, que é onde a pergunta *"o caminhar come isto?"* tem resposta
/// legível.
const BLAST: f32 = 8.0;
/// O raio dela — grande o bastante para o personagem estar dentro com folga.
const BLAST_RADIUS: f32 = 6.0;

/// Os três modos, com o nome que os smokes usam.
///
/// ⚠️ **O modo é um PAR, não um componente:** a porta é o `pose_owner`, e ela
/// pergunta ao KIND do corpo que a ponte de facto construiu antes de olhar para
/// o `PlayerMode`. Inserir só o componente sobre um corpo `Dynamic` mede o MESMO
/// caminho três vezes — foi o que o controlo positivo da sonda do teto de queda
/// apanhou (`measure_terminal.rs`).
const MODES: [(Option<PlayerMode>, &str); 3] = [
    (None, "Spring"),
    (Some(PlayerMode::Kinematic), "Snap"),
    (Some(PlayerMode::Pure), "Pure"),
];

/// Uma cena plana com o personagem em repouso no chão, no modo pedido.
fn standing(mode: Option<PlayerMode>) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    if let Some(m) = mode {
        sim.world_mut().entity_mut(player).insert((
            m,
            RigidBody {
                kind: BodyKind::Kinematic,
            },
        ));
    }
    // Deixa-o assentar: um corpo a cair mede a queda, não o empurrão.
    for t in 1..=60_u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge, player)
}

/// Explode ao lado dele e devolve quanto ele andou em cada meio segundo.
fn blast(mode: Option<PlayerMode>, drive: f32) -> Vec<f32> {
    let (mut sim, mut bridge, player) = standing(mode);
    let (x0, y0) = pose(&sim);
    // ⚠️ **À ESQUERDA e abaixo dele**, para o empurrão ser lateral e para cima —
    // uma explosão exactamente debaixo dos pés mede a perna, não o empurrão.
    let hit = bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], BLAST_RADIUS, BLAST);
    let mut out = Vec::new();
    let mut tick = 0_u64;
    for _ in 0..6 {
        let before = pose(&sim).0;
        for _ in 0..30 {
            tick += 1;
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, tick + 60);
        }
        out.push(pose(&sim).0 - before);
    }
    if hit == 0 {
        // A explosão não alcançou ninguém: as colunas abaixo seriam sobre nada.
        out.iter_mut().for_each(|v| *v = f32::NAN);
    }
    out
}

/// **O CONTROLE POSITIVO — a explosão alcança mesmo alguém?**
///
/// ⚠️ Ela devolve quantos corpos empurrou, e um zero aqui torna todas as outras
/// colunas leituras sobre nada. Um corpo cinemático pode nem estar na lista que
/// ela varre — e isso é um ACHADO, não um erro da sonda.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_that_the_blast_reaches_anyone() {
    eprintln!("  modo         corpos empurrados pela explosao");
    for (mode, tag) in MODES {
        let (sim, mut bridge, _) = standing(mode);
        let (x0, y0) = pose(&sim);
        let hit = bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], BLAST_RADIUS, BLAST);
        eprintln!("  {tag:<10}   {hit}");
    }
}

/// **A premissa: o que uma explosão faz a um player hoje?**
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_a_blast_does_today() {
    for drive in [0.0_f32, -1.0] {
        eprintln!(
            "\n  == o jogador {} ==",
            if drive == 0.0 {
                "SOLTA tudo"
            } else {
                "segura a direcao CONTRARIA"
            }
        );
        eprintln!("  modo         deslocamento por meio segundo (m), 0..3 s");
        for (mode, tag) in MODES {
            let d = blast(mode, drive);
            eprintln!(
                "  {tag:<10}   {}",
                d.iter()
                    .map(|v| format!("{v:7.3}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

/// **A que decide o desenho: quanto tempo o empurrão SOBREVIVE?**
///
/// ⚠️ Se o caminhar o come dentro de poucos tiques, a cura não é *"entregar
/// velocidade"* — é **calar o controlo aéreo** por uma janela, e o primitivo
/// para isso já existe (`JumpState::wall_lock`, que o `lib.rs` lê para pôr o
/// motor da caminhada em `Motor::default()`). A sonda imprime a velocidade
/// horizontal tique a tique com o jogador SOLTO e com ele a segurar a direção
/// contrária — é a segunda coluna que diz se um empurrão *"que funciona"*
/// desaparece na mão de quem joga.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_how_long_a_push_survives_the_walk() {
    for drive in [0.0_f32, -1.0] {
        eprintln!(
            "\n  == velocidade horizontal apos a explosao, jogador {} ==",
            if drive == 0.0 {
                "SOLTO"
            } else {
                "a segurar o CONTRARIO"
            }
        );
        let (mut sim, mut bridge, player) = standing(None);
        let (x0, y0) = pose(&sim);
        let hit = bridge.explode(&sim, [x0 - 1.0, y0 - 0.5], BLAST_RADIUS, BLAST);
        assert!(hit > 0, "a explosao tem de alcancar alguem");
        let mut prev = pose(&sim).0;
        for t in 1..=60_u64 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t + 60);
            let x = pose(&sim).0;
            if t <= 12 || t % 6 == 0 {
                eprintln!("    t={t:>3}  vx = {:>7.3} m/s", (x - prev) * 60.0);
            }
            prev = x;
        }
    }
}

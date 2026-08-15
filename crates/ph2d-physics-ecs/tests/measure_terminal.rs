//! **O TETO DE QUEDA, medido antes de decidir a forma** (`W-Fall`, plano 10 §4).
//!
//! Duas perguntas, e a segunda decide o desenho da wave inteira:
//!
//! 1. **Existe velocidade terminal hoje?** A auditoria 09 diz que não. Uma
//!    premissa herdada de outro documento não é uma medição desta wave, e o §0
//!    manda reconferir o número antes de construir sobre ele.
//! 2. **O PLANEIO já vale nos DOIS modos?** O plano prescreve duas
//!    implementações — *"sob Spring é um boost contra a gravidade do solver; sob
//!    Snap é um clamp no `KinematicState`"* —, mas o freio do planeio sai da lei
//!    como um **`Motor`**, e o `kinematic_advance` lê `motor.boost`. Se ele já
//!    atravessa, então UMA porta serve os dois modos e a prescrição do plano está
//!    a duplicar trabalho que a arquitetura já fez.
//!
//! Rode: `cargo test -p ph2d-physics-ecs --test measure_terminal --release
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{
    BodyKind, PhysicsBridge, PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};
use scene_fixture::{pose, scene};

/// De quão alto se larga, em metros acima do repouso.
///
/// ⚠️ **Alto de propósito, e o número é ARITMÉTICA e não gosto:** uma velocidade
/// terminal é onde a queda PÁRA de acelerar, e o oráculo é a coluna de descida
/// deixar de crescer. Se o corpo POUSA dentro da janela medida, as últimas
/// colunas descrevem o CHÃO e leem-se como *"assentou"* — a fixture a mentir
/// exatamente sobre a pergunta que ela existe para responder. Uma queda livre
/// percorre `½·g·t²`, então oito segundos custam **314 m**; com 1000 m a janela
/// inteira é queda, com folga.
const DROP: f32 = 1000.0;

/// Uma cena plana com o personagem NO AR, a [`DROP`] metros acima do repouso.
fn falling(mode: Option<PlayerMode>) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    // ⚠️ **O modo é um par, não um componente.** A porta é o `pose_owner`, e ela
    // pergunta ao KIND do corpo que a ponte de facto construiu antes de olhar
    // para o `PlayerMode` — *"é o corpo que existe que importa, não o que foi
    // pedido"*. Inserir só o componente sobre um corpo `Dynamic` deixa as duas
    // colunas a medir o MESMO caminho, e foi o que o controlo positivo apanhou.
    if let Some(m) = mode {
        sim.world_mut().entity_mut(player).insert((
            m,
            RigidBody {
                kind: BodyKind::Kinematic,
            },
        ));
    }
    let y = pose(&sim).1;
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(player) {
        t.translation.y = y + DROP;
    }
    (sim, bridge, player)
}

/// Deixa cair `secs` segundos e devolve a velocidade de descida a cada segundo.
///
/// ⚠️ **A velocidade é DERIVADA da pose entre tiques, e não lida de um campo** —
/// os dois modos guardam o estado em sítios diferentes (o corpo do rapier num,
/// o `KinematicState` noutro), e uma sonda que perguntasse a um deles estaria a
/// medir modos diferentes com réguas diferentes.
fn descent_per_second(mode: Option<PlayerMode>, hold_jump: bool, secs: u64) -> Vec<f32> {
    let (mut sim, mut bridge, player) = falling(mode);
    let y = |sim: &SimWorld| pose(sim).1;
    let mut out = Vec::new();
    let mut tick = 0_u64;
    for _ in 0..secs {
        let before = y(&sim);
        for _ in 0..60 {
            tick += 1;
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: hold_jump,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, tick);
        }
        out.push(before - y(&sim));
    }
    out
}

/// **O CONTROLE POSITIVO — o modo está mesmo vivo nesta fixture?**
///
/// ⚠️ **Ele existe porque as duas colunas saíram IDÊNTICAS**, e colunas iguais
/// são a assinatura de uma sonda que não contém o fenómeno. A altura de REPOUSO
/// é o discriminante conhecido: os dois modos pousam a alturas diferentes **de
/// propósito** (um paira na mola, o outro encosta), então se elas divergirem o
/// componente está a morder e a igualdade das outras colunas é um ACHADO.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_that_the_mode_is_actually_live() {
    eprintln!("  modo         altura de repouso (m)");
    for (mode, tag) in [(None, "Spring"), (Some(PlayerMode::Kinematic), "Snap")] {
        // ⚠️ **Este probe larga de PERTO, ao contrário dos outros dois.** Uma
        // altura de repouso só existe depois de o corpo POUSAR, e o [`DROP`] das
        // sondas de queda é alto justamente para que ninguém pouse dentro da
        // janela — usá-lo aqui mediria meio-voo com o nome de repouso.
        let (mut sim, mut bridge, player) = falling(mode);
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(player) {
            t.translation.y = 3.0;
        }
        for t in 1..=600 {
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, t);
        }
        let _ = player;
        eprintln!("  {tag:<10}   {:.4}", pose(&sim).1);
    }
}

/// **A premissa: existe velocidade terminal hoje?**
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_whether_a_fall_ever_settles() {
    eprintln!("  modo         descida por segundo (m), 1..8 s");
    for (mode, tag) in [(None, "Spring"), (Some(PlayerMode::Kinematic), "Snap")] {
        let d = descent_per_second(mode, false, 8);
        eprintln!(
            "  {tag:<10}   {}",
            d.iter()
                .map(|v| format!("{v:6.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

/// **A que decide o desenho: o PLANEIO já atravessa para o modo Snap?**
///
/// ⚠️ Se a coluna do Snap assentar com o botão apertado, a porta do `Motor` já
/// serve os dois modos — e a prescrição do plano (duas implementações) descreve
/// trabalho que a arquitetura já fez.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_whether_the_glide_already_crosses_to_the_kinematic_mode() {
    eprintln!("  modo         botao   descida por segundo (m), 1..6 s");
    for (mode, tag) in [(None, "Spring"), (Some(PlayerMode::Kinematic), "Snap")] {
        for held in [false, true] {
            // Um planeio autorado: 4 m/s de teto sob o dedo.
            let (mut sim, mut bridge, player) = falling(mode);
            if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
                p.glide_fall_speed = 4.0;
            }
            let y = |sim: &SimWorld| pose(sim).1;
            let mut out = Vec::new();
            let mut tick = 0_u64;
            for _ in 0..6 {
                let before = y(&sim);
                for _ in 0..60 {
                    tick += 1;
                    bridge.set_player_input(
                        player,
                        PlayerInput {
                            jump: held,
                            ..PlayerInput::default()
                        },
                    );
                    bridge.dispatch(&mut sim, true, tick);
                }
                out.push(before - y(&sim));
            }
            eprintln!(
                "  {tag:<10}   {:<5}   {}",
                if held { "SIM" } else { "nao" },
                out.iter()
                    .map(|v| format!("{v:6.2}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

//! **O PULO MÚLTIPLO, medido antes de qualquer barra** (`W-MultiJump`).
//!
//! ⚠️ **A pergunta que decide a wave não é *"o segundo pulo sai?"*, é *quanto
//! ele SOBE?*** — um pulo do ar dado em plena QUEDA pode ser comido pela
//! velocidade que já havia, e aí a feature dispara, o gate de contagem fica
//! verde, e o jogador vê o personagem afundar meio metro antes de subir.
//!
//! ⚠️ **E a segunda pergunta é o instante:** a altura de um pulo do ar depende
//! de QUANDO ele é dado (no ápice do primeiro, ou já caindo), e é isso que
//! decide se `air_jump_height` é uma promessa ou uma sugestão.
//!
//! Rode: `cargo test -p ph2d-physics-ecs --test measure_multi_jump --release
//! -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// Uma cena plana com o pulo do ar autorado.
fn rig(air_jumps: u32, air_h: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.air_jumps = air_jumps;
        p.air_jump_height = air_h;
    }
    (sim, bridge, player)
}

/// Assenta a mola e devolve o `y` de repouso.
fn settle(sim: &mut SimWorld, bridge: &mut PhysicsBridge, tick: &mut u64) -> f32 {
    for _ in 0..30 {
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
    }
    pose(sim).1
}

/// Um TOQUE: um tique com o botão preso, um com ele solto.
///
/// ⚠️ **Dois tiques, e não um**, porque a lei lê a BORDA (`held && !was_held`):
/// segurar não re-dispara, então um "toque" que nunca solta produz UM pulo e
/// mais nada — que é exatamente o que se quer medir sobre o primeiro, e o que
/// impediria o segundo de existir.
fn tap(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, tick: &mut u64) {
    for held in [true, false] {
        bridge.set_player_input(
            player,
            PlayerInput {
                jump: held,
                ..PlayerInput::default()
            },
        );
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
    }
}

/// Corre `n` tiques com o botão solto, devolvendo o pico de `y`.
fn coast(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    player: Entity,
    tick: &mut u64,
    n: u64,
) -> f32 {
    let mut peak = f32::NEG_INFINITY;
    for _ in 0..n {
        bridge.set_player_input(player, PlayerInput::default());
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
        peak = peak.max(pose(sim).1);
    }
    peak
}

/// **Quanto o segundo pulo acrescenta, em função de QUANDO ele é dado.**
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_a_second_jump_buys() {
    println!("\n== o 2o pulo, por instante do toque (altura autorada 2,0 m) ==");
    println!("  espera  pico(1 pulo)  pico(2 pulos)  ganho");
    for wait in [1_u64, 10, 20, 30, 45, 60, 90] {
        let mut peaks = [0.0_f32; 2];
        for (slot, air) in [0_u32, 1].into_iter().enumerate() {
            let (mut sim, mut bridge, player) = rig(air, 2.0);
            let mut tick = 0_u64;
            let rest = settle(&mut sim, &mut bridge, &mut tick);
            tap(&mut sim, &mut bridge, player, &mut tick);
            let a = coast(&mut sim, &mut bridge, player, &mut tick, wait);
            tap(&mut sim, &mut bridge, player, &mut tick);
            let b = coast(&mut sim, &mut bridge, player, &mut tick, 180);
            peaks[slot] = a.max(b) - rest;
        }
        println!(
            "  {wait:>4}t  {:>11.4}  {:>13.4}  {:>+6.4}",
            peaks[0],
            peaks[1],
            peaks[1] - peaks[0]
        );
    }
}

/// **A carga recarrega no chão?** — a pergunta que o `on_ground` responde.
#[test]
#[ignore = "sonda de medicao"]
fn measure_that_the_charge_refills_on_landing() {
    println!("\n== duas rodadas de (pulo + pulo do ar), com um pouso no meio ==");
    let (mut sim, mut bridge, player) = rig(1, 2.0);
    let mut tick = 0_u64;
    let rest = settle(&mut sim, &mut bridge, &mut tick);
    for round in 1..=2 {
        tap(&mut sim, &mut bridge, player, &mut tick);
        let a = coast(&mut sim, &mut bridge, player, &mut tick, 20);
        tap(&mut sim, &mut bridge, player, &mut tick);
        let b = coast(&mut sim, &mut bridge, player, &mut tick, 200);
        println!(
            "  rodada {round}: pico {:.4} m acima do repouso (pousou em {:.4})",
            a.max(b) - rest,
            pose(&sim).1 - rest
        );
    }
}

/// **Quantas cargas um aperto SÓ consegue queimar?**
///
/// ⚠️ **O oráculo é o PICO, não um contador**, e é o oráculo certo por duas
/// razões: ele é o que o jogador VÊ, e ele mede exatamente o modo de falha —
/// um buffer que sobrevive ao próprio pulo do ar re-dispara no tique seguinte,
/// e três cargas queimadas em tiques consecutivos empilham três boosts num
/// foguete. Um contador diria *"restam 0"* e deixaria a pergunta *"e daí?"*.
#[test]
#[ignore = "sonda de medicao"]
fn measure_how_many_charges_one_press_burns() {
    println!("\n== UM toque no chao, e depois NADA (pico acima do repouso) ==");
    println!("  cargas   pico");
    for air in [0_u32, 1, 3] {
        let (mut sim, mut bridge, player) = rig(air, 2.0);
        let mut tick = 0_u64;
        let rest = settle(&mut sim, &mut bridge, &mut tick);
        tap(&mut sim, &mut bridge, player, &mut tick);
        let peak = coast(&mut sim, &mut bridge, player, &mut tick, 240);
        println!("  {air:>6}  {:>6.4} m", peak - rest);
    }
    println!("  (as tres linhas TEM de ser iguais: um aperto e' UM pulo)");
}

/// **A altura ALCANÇÁVEL, para a cena de smoke escolher a beirada.**
///
/// ⚠️ **O toque e o aperto SEGURADO dão alturas diferentes** (o `cut_gravity`
/// corta um toque curto), então a beirada de uma cena tem de sair do gesto que
/// o artista vai de facto fazer: segurar.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_reachable_height() {
    println!("\n== altura alcancavel (aperto SEGURADO), acima do repouso ==");
    println!("  cargas   1 pulo   2 pulos");
    for air in [0_u32, 1] {
        let (mut sim, mut bridge, player) = rig(air, 2.0);
        let mut tick = 0_u64;
        let rest = settle(&mut sim, &mut bridge, &mut tick);
        // Segura por 30 tiques: altura CHEIA do primeiro pulo.
        let mut peak = f32::NEG_INFINITY;
        for _ in 0..30 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            peak = peak.max(pose(&sim).1);
        }
        let single = peak - rest;
        // Solta, e no ápice aperta de novo (segurando).
        bridge.set_player_input(player, PlayerInput::default());
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        for _ in 0..120 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
            peak = peak.max(pose(&sim).1);
        }
        println!("  {air:>6}  {single:>6.3}   {:>7.3}", peak - rest);
    }
}

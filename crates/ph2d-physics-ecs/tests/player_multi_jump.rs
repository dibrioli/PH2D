//! **O PULO MÚLTIPLO** (`W-MultiJump`) — os gates de COMPORTAMENTO, com o rapier
//! de verdade.
//!
//! A lei pura tem os dela na `ph2d-platformer` (quem gasta carga, de que altura,
//! quem recarrega). Estes fazem a pergunta que só a simulação responde: *o
//! personagem de fato SOBE mais alto com o segundo pulo, um aperto só continua
//! sendo um pulo, e a carga volta ao pousar?*
//!
//! ⚠️ **Nenhum número aqui foi escolhido** — todos saíram da sonda
//! `measure_multi_jump` (`-- --ignored --nocapture`), que imprime as tabelas.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// Cena plana com o pulo do ar autorado.
fn rig(air_jumps: u32) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.air_jumps = air_jumps;
        p.air_jump_height = 2.0;
    }
    (sim, bridge, player)
}

fn settle(sim: &mut SimWorld, bridge: &mut PhysicsBridge, tick: &mut u64) -> f32 {
    for _ in 0..30 {
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
    }
    pose(sim).1
}

/// Um TOQUE — um tique preso, um solto. A lei lê a BORDA, então segurar não
/// re-dispara e o segundo toque precisa do tique solto para existir.
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

/// O pico acima do repouso de *tocar, esperar `wait` tiques, tocar de novo*.
fn two_taps(air_jumps: u32, wait: u64) -> f32 {
    let (mut sim, mut bridge, player) = rig(air_jumps);
    let mut tick = 0_u64;
    let rest = settle(&mut sim, &mut bridge, &mut tick);
    tap(&mut sim, &mut bridge, player, &mut tick);
    let a = coast(&mut sim, &mut bridge, player, &mut tick, wait);
    tap(&mut sim, &mut bridge, player, &mut tick);
    let b = coast(&mut sim, &mut bridge, player, &mut tick, 180);
    a.max(b) - rest
}

/// **O GATE DA WAVE: o segundo toque no ar SOBE mais alto.**
///
/// Medido (`measure_what_a_second_jump_buys`, toque no tique 20 do voo): **um
/// toque leva a 0,6176 m e dois levam a 1,2326** — praticamente o dobro, que é o
/// que um segundo pulo de mesma altura deve dar.
///
/// ⚠️ **A barra compara os DOIS mundos na mesma cena** (`air_jumps` 0 contra 1),
/// e não um valor absoluto: um pico absoluto teria de ser recalibrado toda vez
/// que alguém mexesse na gravidade de fase, e passaria a medir aquilo em vez
/// desta wave.
#[test]
fn a_second_tap_in_the_air_buys_a_second_jump() {
    let without = two_taps(0, 20);
    let with = two_taps(1, 20);
    assert!(
        with > without * 1.6,
        "com carga o pico tem de subir muito (medido 0,6176 -> 1,2326): \
         sem {without:.4} com {with:.4}"
    );
}

/// **UM aperto é UM pulo, com quantas cargas houver.**
///
/// Medido: `0`, `1` e `3` cargas dão **0,6176 m** — o MESMO pico.
///
/// ⚠️ **Este gate NÃO é o guardião do consumo do buffer, e a primeira versão
/// deste doc dizia que era:** a mutação que apaga o `next.buffer = 0.0` do pulo
/// do ar deixa-o VERDE, porque neste caminho a decolagem do CHÃO já zerou o
/// buffer antes de o personagem chegar ao ar — o aperto guardado nunca alcança o
/// ramo do ar. Quem apanha aquela mutação é o irmão de unidade
/// (`one_press_burns_exactly_one_charge`), cuja fixture entra JÁ no ar com uma
/// borda fresca. O que este mede é a propriedade de PRODUTO — *um aperto, um
/// pulo* — pelo caminho que o jogador percorre.
#[test]
fn one_press_is_one_jump_no_matter_how_many_charges() {
    let (mut sim, mut bridge, player) = rig(0);
    let mut tick = 0_u64;
    let rest = settle(&mut sim, &mut bridge, &mut tick);
    tap(&mut sim, &mut bridge, player, &mut tick);
    let base = coast(&mut sim, &mut bridge, player, &mut tick, 240) - rest;

    for air in [1_u32, 3] {
        let (mut sim, mut bridge, player) = rig(air);
        let mut tick = 0_u64;
        let rest = settle(&mut sim, &mut bridge, &mut tick);
        tap(&mut sim, &mut bridge, player, &mut tick);
        let peak = coast(&mut sim, &mut bridge, player, &mut tick, 240) - rest;
        assert!(
            (peak - base).abs() < 1.0e-3,
            "com {air} cargas e UM aperto o pico tem de ser o mesmo \
             (medido 0,6176 nas tres): base {base:.4} com {air} cargas {peak:.4}"
        );
    }
}

/// **A carga volta ao POUSAR** — duas rodadas de (pulo + pulo do ar) dão o mesmo
/// pico, medido: **1,2326 m nas duas**.
///
/// ⚠️ Sem a recarga a segunda rodada colapsaria para o pico de um pulo só, que é
/// o mesmo número do gate acima — então este gate e aquele **medem coisas
/// diferentes com o mesmo aparelho**, e é a rodada 2 que só existe aqui.
#[test]
fn the_charge_refills_on_landing() {
    let (mut sim, mut bridge, player) = rig(1);
    let mut tick = 0_u64;
    let rest = settle(&mut sim, &mut bridge, &mut tick);
    let mut peaks = [0.0_f32; 2];
    for peak in &mut peaks {
        tap(&mut sim, &mut bridge, player, &mut tick);
        let a = coast(&mut sim, &mut bridge, player, &mut tick, 20);
        tap(&mut sim, &mut bridge, player, &mut tick);
        let b = coast(&mut sim, &mut bridge, player, &mut tick, 200);
        *peak = a.max(b) - rest;
    }
    assert!(
        (peaks[0] - peaks[1]).abs() < 1.0e-3,
        "a 2a rodada tem de repetir a 1a (medido 1,2326 nas duas): {peaks:?}"
    );
    assert!(
        peaks[1] > 1.0,
        "e as duas tem de ser um pulo DUPLO, nao um simples: {peaks:?}"
    );
}

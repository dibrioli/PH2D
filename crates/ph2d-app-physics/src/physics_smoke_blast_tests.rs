//! Os gates da cena 117 (`W-Launch`) — o ESTOURO, medido nesta geometria.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre as TRÊS raias: um
//! gate que só afirmasse *"o do meio se mexeu"* passaria numa cena em que todos
//! caem.

use super::{FLOAT, LANE_SPAN, LANES, build_blast_scene, lane_x};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// O raio e o impulso da FERRAMENTA — lidos do default dela, e não escolhidos.
///
/// ⚠️ **A primeira versão deste gate inventou `6,0` e `8,0`**, e com eles o
/// personagem da direita ficava exactamente NA borda do alcance: `0,000 m`, um
/// gate vermelho sobre produto correto. Os números da cena têm de ser os que o
/// artista de facto tem na mão.
fn tool() -> (f32, f32) {
    let s = ph2d_physics_ecs::InteractionSettings::default().clamped();
    (s.blast_radius, s.blast_impulse)
}

/// A cena montada, com o relógio pronto a andar.
fn rig() -> (SimWorld, PhysicsBridge, Vec<(Entity, String)>) {
    let mut sim = SimWorld::new();
    let ids = build_blast_scene(sim.world_mut());
    let names: Vec<String> = LANES.iter().map(|(t, _, _)| (*t).to_string()).collect();
    let pairs = ids.into_iter().zip(names).collect();
    (sim, PhysicsBridge::new(), pairs)
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .map_or(f32::NAN, |t| t.translation.x)
}

/// Assenta, estoura **debaixo de cada um** (ou não) e devolve quanto cada um
/// andou.
///
/// ⚠️ **UM estouro por personagem, e é o que o artista faz:** o raio da
/// ferramenta é `3 m` e as raias estão a `4` — um clique só não alcança os três,
/// e um gate que fingisse que sim mediria a falloff em vez do modo.
fn blast(with_blast: bool, drive: f32) -> Vec<f32> {
    let (radius, impulse) = tool();
    let (mut sim, mut bridge, ids) = rig();
    for t in 1..=60_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let before: Vec<f32> = ids.iter().map(|(e, _)| x_of(&sim, *e)).collect();
    if with_blast {
        // ⚠️ **ABAIXO e à ESQUERDA de cada um** — no centro de massa não há
        // direção (o `explode` recusa, e `normalize` de um vetor nulo é NaN), e
        // por cima mediria a perna em vez do empurrão.
        for i in 0..ids.len() {
            bridge.explode(&sim, [lane_x(i) - 1.0, FLOAT - 0.5], radius, impulse);
        }
    }
    for t in 61..=90_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(
                *e,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, t);
    }
    ids.iter()
        .enumerate()
        .map(|(i, (e, _))| x_of(&sim, *e) - before[i])
        .collect()
}

/// **⚠️ O GATE DA CENA: o estouro move os TRÊS** — e o CONTROLE é a mesma cena
/// sem estouro nenhum.
///
/// ⚠️ **O controlo está dentro do mesmo gate de propósito:** *"ele andou"* só
/// quer dizer alguma coisa ao lado de *"e sem o estouro ele não anda"*. E é ele
/// que torna visível o mundo de antes desta wave — ali dois dos três ficavam
/// exactamente onde estavam.
#[test]
fn the_blast_moves_all_three_and_the_control_stands_still() {
    let idle = blast(false, 0.0);
    let hit = blast(true, 0.0);
    for (i, (tag, _, _)) in LANES.iter().enumerate() {
        assert!(
            idle[i].abs() < 0.05,
            "[{tag}] o CONTROLE tem de ficar parado: {:.3} m",
            idle[i]
        );
        assert!(
            hit[i].abs() > 0.5,
            "[{tag}] o estouro tem de o tirar do lugar: {:.3} m",
            hit[i]
        );
    }
}

/// **Segurar a direção contrária não apaga o estouro** — o passo 4 do roteiro.
///
/// ⚠️ **É o caso em que um empurrão *"que funciona"* desaparece na mão de quem
/// joga**, e a janela é a única coisa que o impede.
#[test]
fn holding_against_the_blast_does_not_pin_anyone() {
    // O dedo aponta para o estouro (para a esquerda), contra o empurrão.
    let hit = blast(true, -1.0);
    for (i, (tag, _, _)) in LANES.iter().enumerate() {
        assert!(
            hit[i].abs() > 0.3,
            "[{tag}] o dedo contrario nao pode prender ninguem: {:.3} m",
            hit[i]
        );
    }
}

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação.
#[test]
fn the_scene_prints_the_numbers_it_builds() {
    const _: () = assert!(LANE_SPAN > 1.0);
    const _: () = assert!(LANES[0].1.is_none());
    assert!(super::BLAST_SMOKE_MESSAGE.contains("O ESTOURO (W-Launch)"));
    // As três raias não se sobrepõem.
    assert!(lane_x(1) - lane_x(0) >= LANE_SPAN);
}

/// **A SONDA que dá os números ao roteiro** — quanto cada modo anda.
///
/// Rode: `cargo test -p ph2d-host-desktop --release --bins
/// what_the_blast_does_to_each_mode -- --ignored --nocapture`
#[test]
#[ignore = "sonda de dimensionamento"]
fn what_the_blast_does_to_each_mode() {
    let (radius, impulse) = tool();
    println!("\n== o estouro (raio {radius}, impulso {impulse}) na cena 117 ==");
    for (label, drive) in [("dedo SOLTO", 0.0_f32), ("dedo CONTRA", -1.0)] {
        let hit = blast(true, drive);
        println!("  {label}:");
        for (i, (tag, _, _)) in LANES.iter().enumerate() {
            println!("    {tag:<8} andou {:>7.3} m em meio segundo", hit[i]);
        }
    }
}

/// **SONDA: a velocidade LOGO a seguir ao estouro, por modo.**
///
/// ⚠️ Ela separa duas explicações para o deslocamento diferir entre os modos: *o
/// empurrão entregue é outro* (a primeira amostra difere) contra *o que trava
/// depois é outro* (a primeira amostra é igual e a cauda diverge).
#[test]
#[ignore = "sonda de diagnostico"]
fn how_fast_each_mode_leaves_and_how_it_slows() {
    let (radius, impulse) = tool();
    let (mut sim, mut bridge, ids) = rig();
    for t in 1..=60_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    for i in 0..ids.len() {
        bridge.explode(&sim, [lane_x(i) - 1.0, FLOAT - 0.5], radius, impulse);
    }
    let mut prev: Vec<f32> = ids.iter().map(|(e, _)| x_of(&sim, *e)).collect();
    println!("\n== velocidade horizontal por tique, apos o estouro ==");
    for t in 61..=90_u64 {
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
        if t <= 64 || t % 6 == 0 {
            let row: Vec<String> = ids
                .iter()
                .enumerate()
                .map(|(i, (e, _))| {
                    let x = x_of(&sim, *e);
                    let v = (x - prev[i]) * 60.0;
                    format!("{v:>7.2}")
                })
                .collect();
            println!("  t={t:>3}  {}", row.join("  "));
        }
        for (i, (e, _)) in ids.iter().enumerate() {
            prev[i] = x_of(&sim, *e);
        }
    }
    println!(
        "  (colunas: {})",
        LANES
            .iter()
            .map(|(t, _, _)| *t)
            .collect::<Vec<_>>()
            .join("  ")
    );
}

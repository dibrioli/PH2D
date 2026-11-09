//! Os gates da cena 116 (`W-Fall`) — o POÇO, medido nesta geometria.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre as TRÊS raias: um
//! gate que só afirmasse *"o do meio desce devagar"* passaria numa cena em que
//! ninguém cai.

use super::{CAP, DROP_TOP, FLOAT, GLIDE, GROUND_END, GROUND_TOP, LANE_SPAN, LANES, lane_x};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// Quantos tiques de sobra: a raia mais lenta desta cena resolve muito antes.
const TICKS: u64 = 1_200;

/// A cena montada, com o relógio pronto a andar.
fn rig() -> (SimWorld, PhysicsBridge, Vec<(Entity, String)>) {
    let mut sim = SimWorld::new();
    let ids = build_ids(&mut sim);
    (sim, PhysicsBridge::new(), ids)
}

fn build_ids(sim: &mut SimWorld) -> Vec<(Entity, String)> {
    let _ = super::build_terminal_scene(sim.world_mut());
    let names: Vec<&str> = LANES.iter().map(|(tag, _, _)| *tag).collect();
    let mut out: Vec<(Entity, String)> = {
        let mut q = sim.world_mut().try_query::<(Entity, &Name)>().unwrap();
        q.iter(sim.world())
            .filter(|(_, n)| names.contains(&n.as_str()))
            .map(|(e, n)| (e, n.as_str().to_string()))
            .collect()
    };
    // A ordem das raias, e não a de spawn — as colunas do relatório têm de
    // corresponder ao que o artista vê da esquerda para a direita.
    out.sort_by_key(|(_, n)| names.iter().position(|t| t == n).unwrap_or(usize::MAX));
    out
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .map(|t| t.translation.y)
        .expect("o personagem tem de existir")
}

/// Deixa cair com uma entrada fixa e devolve, por raia, **em que segundo cada um
/// chega à altura de repouso**.
///
/// ⚠️ **O oráculo é a altura de REPOUSO, e não o solo:** o personagem paira uma
/// perna acima do chão (a mola do `float_height`), então perguntar pelo `y = 0`
/// nunca teria resposta.
fn landing_seconds(hold_jump: bool) -> Vec<Option<f32>> {
    let (mut sim, mut bridge, ids) = rig();
    let rest = GROUND_TOP + FLOAT;
    let mut out = vec![None; ids.len()];
    for tick in 1..=TICKS {
        for (e, _) in &ids {
            bridge.set_player_input(
                *e,
                PlayerInput {
                    jump: hold_jump,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        for (slot, (e, _)) in ids.iter().enumerate() {
            if out[slot].is_none() && y_of(&sim, *e) <= rest + 0.05 {
                out[slot] = Some(tick as f32 / 60.0);
            }
        }
        if out.iter().all(Option::is_some) {
            break;
        }
    }
    out
}

/// **⚠️ O GATE DA CENA: o teto decide quem chega primeiro** — e a raia sem teto
/// é o CONTROLE.
///
/// As três metades importam:
///
/// - o **sem teto** tem de chegar primeiro, e por MUITO — senão a cena não
///   mostra diferença nenhuma a olho;
/// - as **duas com teto** têm de chegar JUNTAS sem o dedo — é isso que prova que
///   o teto não pergunta nada ao jogador, e é o discriminante contra o planeio;
/// - a raia de teto tem de descer perto do número AUTORADO, e não meramente
///   *mais devagar*.
#[test]
fn the_cap_decides_who_lands_first() {
    let t = landing_seconds(false);
    let (free, capped, both) = (
        t[0].expect("o sem teto tem de aterrar"),
        t[1].expect("o do teto tem de aterrar"),
        t[2].expect("o do teto+planeio tem de aterrar"),
    );

    assert!(
        capped > free * 1.8,
        "o teto tem de custar MUITO tempo, senao a cena nao mostra nada a olho: \
         sem teto {free:.2} s contra {capped:.2} s"
    );
    // ⚠️ **Sem o dedo, o planeio nao existe** — as duas raias com teto caem
    // exactamente igual, e e' o que separa uma lei da outra.
    assert!(
        (capped - both).abs() < 0.05,
        "sem o dedo as duas raias com teto tem de cair IGUAL: {capped:.2} contra {both:.2}"
    );

    // E o ritmo e' o do numero autorado: a descida util dividida pelo tempo.
    let fall = DROP_TOP - GROUND_TOP;
    let rate = fall / capped;
    assert!(
        (CAP * 0.85..CAP * 1.15).contains(&rate),
        "a descida media tem de ser o teto de {CAP:.2} m/s: {rate:.2}"
    );
}

/// **Segurar o pulo aperta APENAS a terceira raia** — o passo 3 do roteiro, e o
/// gate da COMPOSIÇÃO.
///
/// ⚠️ **As duas metades são o `min` a trabalhar**, e a segunda é a que um `max`
/// acidental derruba: o planeio (mais apertado) tem de VENCER onde está armado,
/// e as raias sem planeio não podem sentir o dedo.
#[test]
fn holding_the_button_only_slows_the_lane_that_glides() {
    let idle = landing_seconds(false);
    let held = landing_seconds(true);

    for slot in [0_usize, 1] {
        let (a, b) = (idle[slot].expect("aterra"), held[slot].expect("aterra"));
        assert!(
            (a - b).abs() < 0.05,
            "a raia {} nao tem planeio: o dedo nao pode mudar nada ({a:.2} contra {b:.2})",
            LANES[slot].0
        );
    }

    let (a, b) = (idle[2].expect("aterra"), held[2].expect("aterra"));
    assert!(
        b > a * 1.8,
        "a raia do planeio tem de ficar MUITO mais lenta com o dedo: {a:.2} contra {b:.2}"
    );
    let rate = (DROP_TOP - GROUND_TOP) / b;
    assert!(
        (GLIDE * 0.85..GLIDE * 1.25).contains(&rate),
        "e o ritmo dela e' o do PLANEIO ({GLIDE:.2} m/s), o menor dos dois tetos: {rate:.2}"
    );
}

/// **As raias não se tocam** — a geometria de uma não pode alcançar a outra.
#[test]
fn the_lanes_do_not_reach_each_other() {
    const _: () = assert!(LANE_SPAN > GROUND_END);
    assert!(lane_x(1) - lane_x(0) > GROUND_END);
}

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação.
#[test]
fn the_scene_prints_the_numbers_it_builds() {
    const _: () = assert!(DROP_TOP > GROUND_TOP);
    // ⚠️ O planeio TEM de ser o menor dos dois, senão a terceira raia seria uma
    // cópia da segunda e a composição não teria o que mostrar.
    const _: () = assert!(GLIDE < CAP);
    const _: () = assert!(LANES[0].1 == 0.0 && LANES[0].2 == 0.0);
    assert!(super::TERMINAL_SMOKE_MESSAGE.contains("O POCO (W-Fall)"));
}

/// **A SONDA que dá os números ao roteiro** — quando cada raia chega ao chão.
///
/// Rode: `cargo test -p ph2d-host-desktop --release --bins
/// when_each_lane_lands -- --ignored --nocapture`
#[test]
#[ignore = "sonda de dimensionamento"]
fn when_each_lane_lands() {
    println!("\n== quando cada raia chega ao repouso (largada de {DROP_TOP:.2} m) ==");
    for held in [false, true] {
        let t = landing_seconds(held);
        println!("  dedo no pulo: {}", if held { "SIM" } else { "nao" });
        for (slot, (tag, cap, glide)) in LANES.iter().enumerate() {
            match t[slot] {
                Some(s) => println!(
                    "    {tag:<14} (cap {cap:>5.2}, planeio {glide:>5.2})  ->  {s:>6.2} s  \
                     (ritmo medio {:>5.2} m/s)",
                    (DROP_TOP - GROUND_TOP) / s
                ),
                None => println!("    {tag:<14} nunca chegou em {TICKS} tiques"),
            }
        }
    }
}

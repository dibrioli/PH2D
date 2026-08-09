//! **A cena 103, medida HEADLESS antes de a mensagem ser escrita.**
//!
//! A política do plano 07 §6: *toda wave ganha cena com números MEDIDOS*. Este
//! arquivo dirige a MESMA cena pelas portas do produto e afirma o que a mensagem
//! promete ao artista — com as duas pistas de baixo como CONTROLE.

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

const TAGS: [&str; 3] = ["Pure", "Snap", "Spring"];

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build(sim.world_mut());
    (sim, PhysicsBridge::new())
}

fn named(sim: &SimWorld, name: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena tem de conter '{name}'"))
}

fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("pose").translation;
    (t.x, t.y)
}

fn walk(secs: u64) -> [(f32, f32); 3] {
    let (mut sim, mut bridge) = scene();
    let who: Vec<Entity> = TAGS.iter().map(|n| named(&sim, n)).collect();
    let crates: Vec<Entity> = TAGS
        .iter()
        .map(|n| named(&sim, &format!("Crate {n}")))
        .collect();
    let slings: Vec<Entity> = TAGS
        .iter()
        .map(|n| named(&sim, &format!("Sling {n}")))
        .collect();
    // ⚠️ Assenta antes de medir: no primeiríssimo tique o BVH está vazio e o
    // personagem cai um tique antes de a perna o pegar — e a prancha pendurada
    // ainda está a encontrar o repouso da mola.
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before: Vec<(f32, f32)> = (0..3)
        .map(|i| (pos(&sim, crates[i]).0, pos(&sim, slings[i]).1))
        .collect();
    for w in &who {
        bridge.set_player_input(
            *w,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
    }
    for t in 121..=(120 + secs * 60) {
        bridge.dispatch(&mut sim, true, t);
    }
    [0, 1, 2].map(|i| {
        (
            pos(&sim, crates[i]).0 - before[i].0,
            pos(&sim, slings[i]).1 - before[i].1,
        )
    })
}

/// **Passo 1 — só o puro sangue não empurra o caixote.**
#[test]
fn only_the_pure_lane_leaves_the_crate_where_it_was() {
    let [pure, snap, spring] = walk(3);
    assert!(
        spring.0 > 1.0 && snap.0 > 1.0,
        "os CONTROLES tem de empurrar: ciano {:.4} m, laranja {:.4} m",
        spring.0,
        snap.0
    );
    assert!(
        pure.0.abs() < 0.01,
        "e o verde passa sem mover nada: {:.4} m",
        pure.0
    );
}

/// **Passo 2 — só o puro sangue não afunda a plataforma pendurada.**
///
/// ⚠️ É a metade da 3ª lei que um caixote não mostra: o PESO. Sem ela a cena
/// provaria metade da wave e o artista julgaria a outra pelo silêncio.
#[test]
fn only_the_pure_lane_leaves_the_hanging_platform_where_it_was() {
    let [pure, snap, spring] = walk(3);
    assert!(
        spring.1 < -0.02 && snap.1 < -0.02,
        "os CONTROLES tem de afundar a prancha: ciano {:.4} m, laranja {:.4} m",
        spring.1,
        snap.1
    );
    assert!(
        pure.1.abs() < 0.005,
        "e o verde nao a toca: {:.4} m",
        pure.1
    );
}

/// **Passo 3 — a parede para os TRÊS.** Cenário não quer dizer fantasma.
#[test]
fn the_wall_stops_all_three_of_them() {
    let (mut sim, mut bridge) = scene();
    let who: Vec<Entity> = TAGS.iter().map(|n| named(&sim, n)).collect();
    for w in &who {
        bridge.set_player_input(
            *w,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
    }
    for t in 1..=900u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    for (i, p) in who.iter().enumerate() {
        let (x, _) = pos(&sim, *p);
        assert!(
            x < WALL_X,
            "{}: o personagem tem de PARAR na parede: x={x:.4} contra {WALL_X}",
            TAGS[i]
        );
    }
}

/// **Passo 5 — o card REACTION segue o modo, e o chip que o traz de volta fica.**
///
/// ⚠️ O gate de painel prova que o card some; este prova que a cena que o
/// artista abre de facto contém os três modos, com o `PlayerMode` escrito como
/// o chip o escreve. Uma cena que montasse os três como Kinematic passaria em
/// todos os outros gates deste arquivo.
#[test]
fn the_scene_really_ships_one_lane_of_each_mode() {
    let (sim, _) = scene();
    for (tag, mode, _) in LANES {
        let e = named(&sim, tag);
        let got = sim.world().get::<PlayerMode>(e).copied();
        if mode == PlayerMode::default() {
            assert!(got.is_none(), "{tag}: o neutro nao carrega componente");
        } else {
            assert_eq!(got, Some(mode), "{tag}: a pista tem de ser deste modo");
        }
        assert!(
            sim.world().get::<PlatformPlayer>(e).is_some(),
            "{tag}: e tem de ser um player"
        );
    }
}

/// **A tabela que a mensagem da cena cita** — rodada ANTES de ela ser escrita.
///
/// `cargo test -p ph2d-host-desktop --release --bins the_numbers_the_scene_claims -- --ignored --nocapture`
#[test]
#[ignore = "sonda"]
fn the_numbers_the_scene_claims() {
    let lanes = walk(3);
    println!("\n  pista     caixote(m)  prancha(m)");
    for (i, (krate, sling)) in lanes.iter().enumerate() {
        println!("  {:<8}  {krate:>9.4}  {sling:>9.4}", TAGS[i]);
    }
    println!();
}

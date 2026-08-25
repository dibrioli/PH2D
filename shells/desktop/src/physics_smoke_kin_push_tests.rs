//! **A cena 102, medida HEADLESS antes de a mensagem ser escrita.**
//!
//! A política do plano 07 §6: *toda wave ganha cena com números MEDIDOS*. Este
//! arquivo dirige a MESMA cena pelas portas do produto e afirma o que a mensagem
//! promete ao artista — incluindo o CONTROLE, que é a metade que faz a outra
//! significar alguma coisa.

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    (sim, PhysicsBridge::new())
}

fn named(sim: &SimWorld, name: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena tem de conter '{name}'"))
}

fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("pose").translation.x
}

/// Anda para a direita por `secs` e devolve quanto cada caixote andou, por
/// pista: `(claro, escuro)`.
fn walk(secs: u64) -> [(f32, f32); 2] {
    let (mut sim, mut bridge) = scene();
    let who: Vec<Entity> = ["Snap", "Spring"].iter().map(|n| named(&sim, n)).collect();
    let light: Vec<Entity> = ["Light Snap", "Light Spring"]
        .iter()
        .map(|n| named(&sim, n))
        .collect();
    let heavy: Vec<Entity> = ["Heavy Snap", "Heavy Spring"]
        .iter()
        .map(|n| named(&sim, n))
        .collect();
    // ⚠️ Assenta antes de medir: o primeiríssimo tique de uma cena tem o BVH
    // vazio e o personagem cai um tique antes de a perna o pegar.
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before: Vec<(f32, f32)> = (0..2)
        .map(|i| (x_of(&sim, light[i]), x_of(&sim, heavy[i])))
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
    for t in 61..=(60 + secs * 60) {
        bridge.dispatch(&mut sim, true, t);
    }
    [0, 1].map(|i| {
        (
            x_of(&sim, light[i]) - before[i].0,
            x_of(&sim, heavy[i]) - before[i].1,
        )
    })
}

/// **Passo 1 — os DOIS empurram o caixote claro**, e o dinâmico é o controle.
///
/// ⚠️ **A barra NÃO é uma razão entre as pistas, e a razão é a CENA:** os
/// caixotes acabam empilhados contra a parede, então o quanto cada um andou é
/// limitado pela GEOMETRIA e não pela lei (medido: claro 4,95 no Snap e 6,39 no
/// Spring). A paridade entre os modos é do `player_push.rs`, cuja pista é
/// LIVRE — aqui o que se afirma é que os dois empurram, que é o que o passo 1
/// pede ao olho.
#[test]
fn both_lanes_shove_the_light_crate() {
    let [snap, spring] = walk(3);
    assert!(
        spring.0 > 3.0,
        "o CONTROLE tem de empurrar: o ciano levou {:.4} m",
        spring.0
    );
    assert!(
        snap.0 > 3.0,
        "e o cinematico tambem: laranja {:.4} contra ciano {:.4}",
        snap.0,
        spring.0
    );
}

/// **Passo 2 — o caixote ESCURO anda menos, nas duas pistas.**
#[test]
fn the_heavy_crate_moves_less_in_both_lanes() {
    let lanes = walk(3);
    for (i, (light, heavy)) in lanes.iter().enumerate() {
        let lane = if i == 0 { "Snap" } else { "Spring" };
        assert!(
            *light > *heavy + 1.0,
            "{lane}: o claro tem de andar mais que o escuro ({light:.4} contra {heavy:.4})"
        );
    }
}

/// **Passo 3 — a parede estática não cede, e o personagem PARA nela.**
#[test]
fn the_static_wall_holds_both_of_them() {
    let (mut sim, mut bridge) = scene();
    let who: Vec<Entity> = ["Snap", "Spring"].iter().map(|n| named(&sim, n)).collect();
    let walls: Vec<Entity> = ["Wall Snap", "Wall Spring"]
        .iter()
        .map(|n| named(&sim, n))
        .collect();
    let before: Vec<f32> = walls.iter().map(|w| x_of(&sim, *w)).collect();
    for w in &who {
        bridge.set_player_input(
            *w,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
    }
    for t in 1..=600u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    for (i, w) in walls.iter().enumerate() {
        assert!(
            (x_of(&sim, *w) - before[i]).abs() < 1.0e-4,
            "massa infinita nao cede: {} -> {}",
            before[i],
            x_of(&sim, *w)
        );
    }
    for (i, p) in who.iter().enumerate() {
        let x = x_of(&sim, *p);
        assert!(
            x < WALL_X,
            "o personagem {i} tem de PARAR na parede: x={x:.4} contra {WALL_X}"
        );
    }
}

/// **Passo 5 — o knob DESLIGA o canal**, e o ciano não se importa.
///
/// ⚠️ É este gate que prova que o número da §14 é o mesmo que a lei lê. Sem a
/// metade do CIANO ele não distinguiria *"o knob desligou o empurrão"* de *"o
/// knob quebrou a cena"*.
#[test]
fn the_knob_turns_the_channel_off_for_snap_only() {
    fn travel(push: f32) -> (f32, f32) {
        let mut sim = SimWorld::new();
        build(sim.world_mut());
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        for tag in ["Snap", "Spring"] {
            let e = named(&sim, tag);
            let mut ent = sim.world_mut().entity_mut(e);
            if let Some(mut p) = ent.get_mut::<ph2d_physics_ecs::PlatformPlayer>() {
                p.reaction_push = push;
            }
        }
        let mut bridge = PhysicsBridge::new();
        let who: Vec<Entity> = ["Snap", "Spring"].iter().map(|n| named(&sim, n)).collect();
        let light: Vec<Entity> = ["Light Snap", "Light Spring"]
            .iter()
            .map(|n| named(&sim, n))
            .collect();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let b = [x_of(&sim, light[0]), x_of(&sim, light[1])];
        for w in &who {
            bridge.set_player_input(
                *w,
                PlayerInput {
                    drive: 1.0,
                    ..PlayerInput::default()
                },
            );
        }
        for t in 61..=240u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        (x_of(&sim, light[0]) - b[0], x_of(&sim, light[1]) - b[1])
    }
    let (snap_off, spring_off) = travel(0.0);
    let (snap_on, spring_on) = travel(1.0);
    assert!(
        snap_off.abs() < 0.01,
        "em zero o laranja volta a ser fantasma de lado: {snap_off:.4}"
    );
    assert!(
        snap_on > 1.0,
        "e em um ele empurra: {snap_on:.4} (zero deu {snap_off:.4})"
    );
    assert!(
        spring_off > 1.0 && (spring_on - spring_off).abs() < 0.1 * spring_off,
        "e o CIANO nao se importa com o knob: {spring_off:.4} contra {spring_on:.4}"
    );
}

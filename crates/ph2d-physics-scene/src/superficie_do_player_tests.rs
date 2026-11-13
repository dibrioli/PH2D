//! **A secção Surface na forma do PLAYER** — a auditoria de controlo morto irmã do Bug #14 (atrito).
//!
//! Grip, Belt e Wall Cling são lidos só pelos raios de chão e de parede de UM player
//! (`Surfaces::at`/`clings_at`), que excluem o próprio corpo: na forma do player sozinho não
//! mudam um bit. Mas DECIDEM para OUTRO player que pise nele ou deslize no flanco dele, então
//! ficam no Inspector (ao contrário do atrito, que a ponte sobrescreve sempre). One-Way era do par
//! de contacto (o gancho do rapier) e só fazia o próprio player cair pelo chão (y final −16,671 ×
//! 0,900): a ponte ignora-o na forma do player (`forma_do_player`), e o Inspector esconde-o.
//! A régua compara a trajectória inteira, bit a bit, contra o mesmo mundo sem o componente.

use bevy_ecs::world::World;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    NoWallCling, OneWayPlatform, PhysicsBridge, PlatformPlayer, PlayerInput, WalkSurface,
};

use crate::{slab, spawn_player};

/// O que se põe na forma do player.
#[derive(Clone, Copy, Debug)]
enum Na {
    Nada,
    Grip,
    Belt,
    SemCling,
    UmaVia,
}

fn poe(w: &mut World, e: Entity, na: Na) {
    let mut e = w.entity_mut(e);
    match na {
        Na::Nada => {}
        Na::Grip => {
            e.insert(WalkSurface {
                grip: 0.05,
                belt: 0.0,
            });
        }
        Na::Belt => {
            e.insert(WalkSurface {
                grip: 1.0,
                belt: 4.0,
            });
        }
        Na::SemCling => {
            e.insert(NoWallCling);
        }
        Na::UmaVia => {
            e.insert(OneWayPlatform);
        }
    }
}

/// Os tiques: anda para a parede, encosta, pula contra ela, solta, e pára.
fn entrada(t: u64) -> PlayerInput {
    PlayerInput {
        drive: if t < 140 { 1.0 } else { 0.0 },
        jump: (60..100).contains(&t),
        ..PlayerInput::default()
    }
}

fn poses(sim: &SimWorld, es: &[Entity]) -> Vec<[u32; 2]> {
    es.iter()
        .map(|&e| {
            let p = sim.world().get::<Transform>(e).expect("t").translation;
            [p.x.to_bits(), p.y.to_bits()]
        })
        .collect()
}

/// O player SOZINHO: chão, parede, paredes ligadas; a trajectória inteira.
fn sozinho(na: Na) -> Vec<Vec<[u32; 2]>> {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    slab(w, "Floor", Vec2::new(0.0, -0.5), [10.0, 0.5], 0.0, [1.0; 4]);
    slab(w, "Wall", Vec2::new(4.0, 4.0), [1.0, 4.5], 0.0, [1.0; 4]);
    let p = spawn_player(w, Vec2::new(0.0, 1.2));
    w.get_mut::<PlatformPlayer>(p).expect("player").walls_on = true;
    poe(w, p, na);
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 1..=220_u64 {
        bridge.set_player_input(p, entrada(t));
        bridge.dispatch(&mut sim, true, t);
        out.push(poses(&sim, &[p]));
    }
    out
}

/// DOIS players: o B em cima do A (Grip/Belt), e o C a cair encostado ao A (Wall Cling).
fn dois(na: Na) -> Vec<Vec<[u32; 2]>> {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    slab(w, "Floor", Vec2::new(0.0, -0.5), [10.0, 0.5], 0.0, [1.0; 4]);
    // O A num pilar fino: o flanco dele fica no AR, onde o C pode deslizar.
    slab(w, "Pillar", Vec2::new(5.0, 2.0), [0.1, 2.0], 0.0, [1.0; 4]);
    let a = spawn_player(w, Vec2::new(5.0, 5.2));
    let b = spawn_player(w, Vec2::new(5.0, 7.0));
    let c = spawn_player(w, Vec2::new(4.55, 6.2));
    for e in [a, b, c] {
        w.get_mut::<PlatformPlayer>(e).expect("player").walls_on = true;
    }
    poe(w, a, na);
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 1..=160_u64 {
        bridge.set_player_input(a, PlayerInput::default());
        bridge.set_player_input(
            b,
            PlayerInput {
                drive: if (40..60).contains(&t) { 1.0 } else { 0.0 },
                ..PlayerInput::default()
            },
        );
        bridge.set_player_input(
            c,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        out.push(poses(&sim, &[a, b, c]));
    }
    out
}

/// ⭐ **Na forma do player sozinho**, nenhum dos quatro muda um bit da trajectória: Grip, Belt e
/// Wall Cling porque os raios dele o excluem, One-Way porque a ponte o ignora na forma de um player.
/// **Mutação que deve sangrar:** a ponte voltar a ler o `OneWayPlatform` do player (tirar o
/// `one_way: false` de `lei_da_forma_do_player`): ele atravessa o chão.
#[test]
fn a_superficie_na_forma_do_player_sozinho_nenhum_decide() {
    let base = sozinho(Na::Nada);
    let y = |v: &[Vec<[u32; 2]>]| f32::from_bits(v.last().expect("tiques")[0][1]);
    for na in [Na::Grip, Na::Belt, Na::SemCling, Na::UmaVia] {
        let v = sozinho(na);
        assert!(
            v == base,
            "{na:?} na forma do player mudou a trajectória dele (y final {:.3} contra {:.3})",
            y(&v),
            y(&base)
        );
    }
}

/// A medição que separa «morto» de «só lido por OUTRO player»: com um segundo player em cima/ao lado,
/// o que a forma do A diz chega a ele? **Mutação que deve sangrar:** `reconcile_surfaces` saltar a
/// forma de um player (uma `superficie_ignorada` partilhada com o Inspector).
#[test]
fn a_superficie_na_forma_do_player_chega_a_outro_player() {
    let base = dois(Na::Nada);
    for na in [Na::Grip, Na::Belt, Na::SemCling] {
        let v = dois(na);
        // Só os OUTROS (B e C): o A sozinho já foi provado imune acima.
        let primeiro = v.iter().zip(&base).position(|(x, y)| x[1..] != y[1..]);
        eprintln!("{na:?}: primeiro tique diferente = {primeiro:?}");
        assert!(
            primeiro.is_some(),
            "{na:?} na forma do A não chegou a nenhum outro player"
        );
    }
}

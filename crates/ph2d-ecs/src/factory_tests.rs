//! Os gates da fábrica (`docs/Components/09_plano_spawner.md` §5, W1).

use super::*;
use crate::lifetime::Spawned;
use crate::tags::Tags;
use crate::{Name, Transform};
use bevy_ecs::world::World as BevyWorld;
use ph2d_core::Vec2;

fn fabrica(world: &mut BevyWorld, id: u64, x: f32, f: Factory) -> Entity {
    world
        .spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Name::new(format!("Fabrica{id}")),
            StableId(id),
            f,
        ))
        .id()
}

fn com_sinal(sinal: &str) -> Factory {
    Factory {
        master: 42,
        on_signal: sinal.into(),
        ..Default::default()
    }
}

/// **Uma fábrica sem sinal nunca nasce** — a lei da casa (*um consumidor sem nome não escuta, em vez
/// de escutar tudo*).
///
/// (Mutação: aceitar o `on_signal` vazio ⇒ ela passa a nascer com QUALQUER sinal.)
#[test]
fn a_factory_without_a_signal_never_spawns() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            master: 42,
            on_signal: String::new(),
            ..Default::default()
        },
    );
    let t = TagTree::new();
    assert!(
        tick_factories(&mut w, &t, &["", "seja o que for"])
            .births
            .is_empty()
    );
}

/// **Sem receita é SILÊNCIO** — nunca um nascimento na origem, nunca um erro de motor.
#[test]
fn a_factory_without_a_recipe_is_silent() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            master: 0,
            on_signal: "vai".into(),
            ..Default::default()
        },
    );
    let t = TagTree::new();
    assert!(tick_factories(&mut w, &t, &["vai"]).births.is_empty());
}

/// ⭐ **`burst` põe exactamente `burst` cópias, na pose de MUNDO da fábrica.**
#[test]
fn a_burst_puts_exactly_that_many_copies_at_the_factory() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        7.0,
        Factory {
            burst: 5,
            on_spawned: "nasceu".into(),
            ..com_sinal("vai")
        },
    );
    let t = TagTree::new();
    let out = tick_factories(&mut w, &t, &["vai"]);
    assert_eq!(out.births.len(), 5);
    assert!(out.births.iter().all(|b| b.at == [7.0, 0.0]));
    assert_eq!(
        out.spawned,
        vec![(out.births[0].factory, "nasceu".to_string(), 5)],
        "UM evento com a contagem dentro, nunca cinco"
    );
}

/// ⭐⭐ **O tecto do quadro crava o `burst`** — [`BURST_MAX`], medido.
///
/// ⚠️⚠️ **A fixtura pede `BURST_MAX + 7`, e NUNCA `u32::MAX`** — medido a sério em 2026-09-14: com
/// `u32::MAX` a prova de mutação que apaga o tecto faz o teste alocar o que o tecto existe para
/// impedir, e o binário chegou a **27 GB de RSS** antes de ser morto à mão. *Uma prova de mutação
/// de um TECTO tem de pôr a fixtura logo acima dele: o mutante tem de sangrar, não de rebentar a
/// máquina.*
#[test]
fn the_frame_ceiling_clamps_the_burst() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: BURST_MAX + 7,
            ..com_sinal("vai")
        },
    );
    let t = TagTree::new();
    assert_eq!(
        tick_factories(&mut w, &t, &["vai"]).births.len(),
        BURST_MAX as usize
    );
}

/// ⭐⭐ **`alive_max` conta as cópias DESTA fábrica, não as do mundo.**
///
/// ⚠️ **A fixtura tem cópias de OUTRA fábrica** (`by: 2`): sem elas, uma implementação que contasse
/// todo o `Spawned` do mundo passaria — e o defeito só apareceria numa cena com duas fábricas, que
/// é a cena normal.
///
/// (Mutação: contar `Spawned` sem olhar ao `by` ⇒ RED.)
#[test]
fn the_alive_limit_counts_only_this_factorys_copies() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: 10,
            alive_max: 3,
            ..com_sinal("vai")
        },
    );
    // Uma cópia minha, e quatro de outra fábrica.
    w.spawn((
        Transform::IDENTITY,
        Spawned {
            by: 1,
            born_tick: 0,
        },
    ));
    for _ in 0..4 {
        w.spawn((
            Transform::IDENTITY,
            Spawned {
                by: 2,
                born_tick: 0,
            },
        ));
    }
    let t = TagTree::new();
    assert_eq!(
        tick_factories(&mut w, &t, &["vai"]).births.len(),
        2,
        "o tecto e' 3 e ela ja' tem 1 — as quatro da vizinha nao contam"
    );
}

/// ⭐⭐ **`total_max` esgota a fábrica, e ela di-lo UMA vez.**
///
/// (Mutação: tirar a bandeira `exhausted_said` ⇒ ela grita a cada sinal, para sempre.)
#[test]
fn the_total_limit_says_exhausted_exactly_once() {
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: 2,
            total_max: 3,
            on_exhausted: "acabou".into(),
            ..com_sinal("vai")
        },
    );
    let t = TagTree::new();
    let a = tick_factories(&mut w, &t, &["vai"]);
    assert_eq!(a.births.len(), 2);
    assert!(a.exhausted.is_empty(), "ainda falta uma");
    let b = tick_factories(&mut w, &t, &["vai"]);
    assert_eq!(b.births.len(), 1, "so' cabe mais uma");
    assert_eq!(b.exhausted.len(), 1, "e agora ela esgotou-se");
    let c = tick_factories(&mut w, &t, &["vai"]);
    assert!(c.births.is_empty());
    assert!(c.exhausted.is_empty(), "ela disse duas vezes que acabou");
}

/// ⭐ **A mesma semente dá a MESMA chuva** — determinismo é lei da casa.
#[test]
fn the_same_seed_gives_the_same_scatter() {
    let espalha = || {
        let mut w = BevyWorld::new();
        fabrica(
            &mut w,
            1,
            0.0,
            Factory {
                burst: 8,
                seed: 12345,
                at: SpawnAt::Area { w: 10.0, h: 4.0 },
                ..com_sinal("vai")
            },
        );
        let t = TagTree::new();
        tick_factories(&mut w, &t, &["vai"])
            .births
            .into_iter()
            .map(|b| b.at)
            .collect::<Vec<_>>()
    };
    assert_eq!(espalha(), espalha());
}

/// ⭐⭐⭐ **Duas fábricas com a MESMA semente autorada caem em sítios DIFERENTES.**
///
/// ⚠️ É o defeito que a semente crua teria e que ninguém veria num teste de uma fábrica só: duas
/// chuvas lado a lado, com o mesmo valor no painel, cairiam gota a gota no mesmo ponto.
///
/// (Mutação: semear com `seed` em vez de `seed ^ identidade` ⇒ RED.)
#[test]
fn two_factories_with_the_same_authored_seed_scatter_differently() {
    let mut w = BevyWorld::new();
    let a = fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: 6,
            seed: 7,
            at: SpawnAt::Area { w: 10.0, h: 10.0 },
            ..com_sinal("vai")
        },
    );
    let b = fabrica(
        &mut w,
        2,
        0.0,
        Factory {
            burst: 6,
            seed: 7,
            at: SpawnAt::Area { w: 10.0, h: 10.0 },
            ..com_sinal("vai")
        },
    );
    let t = TagTree::new();
    let out = tick_factories(&mut w, &t, &["vai"]);
    let da: Vec<[f32; 2]> = out
        .births
        .iter()
        .filter(|x| x.factory == a)
        .map(|x| x.at)
        .collect();
    let db: Vec<[f32; 2]> = out
        .births
        .iter()
        .filter(|x| x.factory == b)
        .map(|x| x.at)
        .collect();
    assert_eq!(da.len(), 6);
    assert_eq!(db.len(), 6);
    assert_ne!(da, db, "as duas chuvas caem no mesmo sitio");
}

/// ⭐⭐⭐ **O ponto de nascimento é um objecto MARCADO** — a composição que dissolve o `SpawnPoint`.
#[test]
fn a_spawn_point_is_a_tagged_object_and_cycle_goes_round() {
    let mut tree = TagTree::new();
    let ponto = tree.create("SpawnPoint").expect("cria");
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: 5,
            at: SpawnAt::Tagged {
                tag: ponto.0,
                pick: Pick::Cycle,
            },
            ..com_sinal("vai")
        },
    );
    for (i, x) in [10.0f32, 20.0].into_iter().enumerate() {
        w.spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Name::new(format!("P{i}")),
            StableId(100 + i as u64),
            Tags::from_ids([ponto]),
        ));
    }
    let t = tree;
    let poses: Vec<f32> = tick_factories(&mut w, &t, &["vai"])
        .births
        .into_iter()
        .map(|b| b.at[0])
        .collect();
    assert_eq!(
        poses,
        vec![10.0, 20.0, 10.0, 20.0, 10.0],
        "a roda-viva tem de alternar entre os dois pontos"
    );
}

/// **Uma tag sem ninguém = ninguém nasce** — ⛔ e nunca um nascimento na origem, que é o defeito que
/// o artista leria como *«nasce tudo no canto»*.
#[test]
fn a_tag_with_nobody_in_it_spawns_nobody() {
    let mut tree = TagTree::new();
    let ponto = tree.create("SpawnPoint").expect("cria");
    let mut w = BevyWorld::new();
    fabrica(
        &mut w,
        1,
        0.0,
        Factory {
            burst: 3,
            at: SpawnAt::Tagged {
                tag: ponto.0,
                pick: Pick::Cycle,
            },
            ..com_sinal("vai")
        },
    );
    assert!(tick_factories(&mut w, &tree, &["vai"]).births.is_empty());
}

/// ⭐ **Os nascimentos saem pela ordem da IDENTIDADE das fábricas**, nunca pela da query.
///
/// ⚠️ A fixtura cria as fábricas na ordem `3, 1, 2` de propósito — sem isso as duas ordens
/// coincidiriam e o gate passaria por acaso.
#[test]
fn the_births_come_in_factory_identity_order() {
    let mut w = BevyWorld::new();
    for id in [3u64, 1, 2] {
        fabrica(
            &mut w,
            id,
            id as f32,
            Factory {
                burst: 1,
                ..com_sinal("vai")
            },
        );
    }
    let t = TagTree::new();
    let xs: Vec<f32> = tick_factories(&mut w, &t, &["vai"])
        .births
        .into_iter()
        .map(|b| b.at[0])
        .collect();
    assert_eq!(xs, vec![1.0, 2.0, 3.0]);
}

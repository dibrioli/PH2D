//! ⭐ **O bench da captura incremental** — a condição de pronto da F2 (ADR-0164 §2.7).
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test measure_incremental_capture -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]` de propósito, e a razão é a família de flakes do `CLAUDE.md` §5.0:** isto mede
//! um RELÓGIO, e um relógio sob fan-out de 10–18 mil testes em paralelo não mede nada. Corra-o com
//! a máquina calma (`load ≤ 5`) — o teste **imprime o `load` que encontrou** para que um número
//! fora de esquadria seja legível como carga, e não lido como regressão.
//!
//! # As barras, e de onde elas vêm
//!
//! Do spike de 2026-08-21 ([medicao](../../../docs/Components/pesquisa/instancias_2026-08-21/medicao_captura_incremental.md)),
//! release, 25 iterações, mediana, `n = 10 000` com `Transform+Name+RootOrder`:
//!
//! | cenário | spike | barra desta fase |
//! |---|---:|---:|
//! | nada mudou | 0,269 ms | **≤ 0,30 ms** |
//! | 10 % mudou | 0,953 ms | **≤ 1,00 ms** |
//! | *(o de hoje, para comparar)* | 23,8 ms | — |
//!
//! ⚠️ **O spike lia a coluna de ticks da tabela; esta implementação NÃO PODE** — a `ph2d-ecs` tem
//! `#![forbid(unsafe_code)]` na primeira linha, e `Table::get_changed_ticks_slice_for` devolve
//! `&[UnsafeCell<Tick>]`. A forma segura é `EntityRef::get_change_ticks_by_id` restringida à
//! interseção memorizada por archetype. **Se a barra não for cumprida, o número medido é o
//! resultado** — e a decisão (relaxar a barra ou isolar o scan numa crate que permita `unsafe`,
//! como o precedente do Opus no ADR-0116) é do Enio, com a tabela na mão.

use ph2d_ecs::scene::incremental::{CaptureCache, capture_incremental};
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::WorldSnapshot;
use ph2d_ecs::{Name, RootOrder, Transform};
use std::time::Instant;

const N: u32 = 10_000;
const ITERS: usize = 25;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    reg
}

fn load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    v[v.len() / 2]
}

/// A mediana de `ITERS` capturas, em milissegundos.
fn measure(
    world: &mut bevy_ecs::world::World,
    cache: &mut CaptureCache,
    reg: &ComponentRegistry,
    mut before_each: impl FnMut(&mut bevy_ecs::world::World),
) -> f64 {
    let mut out = WorldSnapshot::new();
    let mut samples = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        before_each(world);
        let t0 = Instant::now();
        capture_incremental(world, cache, reg, &mut out).expect("captura");
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    median(samples)
}

#[test]
#[ignore = "mede um relogio; rode com a maquina calma e --release"]
fn the_capture_costs_the_edit_not_the_world() {
    let reg = registry();
    let mut world = bevy_ecs::world::World::new();
    let entities: Vec<_> = (0..N)
        .map(|i| {
            world
                .spawn((
                    Transform::IDENTITY,
                    Name::new(format!("obj{i}")),
                    RootOrder(i),
                ))
                .id()
        })
        .collect();
    let mut cache = CaptureCache::new();

    // A primeira captura constrói tudo — não é o que se mede, é o que se paga uma vez.
    let mut out = WorldSnapshot::new();
    let t0 = Instant::now();
    let first = capture_incremental(&mut world, &mut cache, &reg, &mut out).expect("primeira");
    let first_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(first.spawned, N as usize);

    let idle = measure(&mut world, &mut cache, &reg, |_| {});
    let one_percent = measure(&mut world, &mut cache, &reg, |w| {
        for e in entities.iter().take((N / 100) as usize) {
            w.entity_mut(*e).insert(RootOrder(rand_like(*e)));
        }
    });
    let ten_percent = measure(&mut world, &mut cache, &reg, |w| {
        for e in entities.iter().take((N / 10) as usize) {
            w.entity_mut(*e).insert(RootOrder(rand_like(*e)));
        }
    });

    println!(
        "\n  captura incremental — n = {N}, mediana de {ITERS}, load = {:.2}",
        load_average()
    );
    println!("  ┌──────────────────────────┬──────────┬──────────┐");
    println!("  │ cenário                  │  medido  │  barra   │");
    println!("  ├──────────────────────────┼──────────┼──────────┤");
    println!("  │ 1.ª captura (constrói)   │ {first_ms:>6.3} ms │    —     │");
    println!("  │ nada mudou               │ {idle:>6.3} ms │ 0.300 ms │");
    println!("  │ 1 % mudou                │ {one_percent:>6.3} ms │    —     │");
    println!("  │ 10 % mudou               │ {ten_percent:>6.3} ms │ 1.000 ms │");
    println!("  └──────────────────────────┴──────────┴──────────┘");
    println!("  (hoje, sem incremental: 23.8 ms — o spike de 2026-08-21)\n");

    assert!(
        idle <= 0.30,
        "captura PARADA custou {idle:.3} ms, barra 0.300 ms (spike: 0.269). \
         Se o `load` impresso acima passa de ~5, isto e' CARGA e nao regressao — re-rode calmo."
    );
    assert!(
        ten_percent <= 1.00,
        "captura a 10 % sujo custou {ten_percent:.3} ms, barra 1.000 ms (spike: 0.953)."
    );
}

/// Um número que varia por entidade — para a escrita ser uma mudança REAL, e não absorvida pela
/// comparação de bytes (que é o que este bench NÃO quer medir).
fn rand_like(e: bevy_ecs::entity::Entity) -> u32 {
    // ⚠️ `EntityIndex` não é `u32` no bevy 0.18 — a mesma pedra que a F1 já apanhou.
    (e.to_bits() as u32)
        .wrapping_mul(2_654_435_761)
        .wrapping_add(1)
}

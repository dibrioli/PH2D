//! ⭐⭐ **A SONDA DE NASCER E MORRER** — quanto custa pôr uma cópia no mundo e tirá-la
//! (`docs/Components/09_plano_spawner.md` §6, TOP-20 #11 e #12).
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test it measure_spawn -- --include-ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]` de propósito** (família de flakes do `CLAUDE.md` §5.0): mede um RELÓGIO, e
//! imprime o `load` antes e depois da tabela. Acima de `load ~5` a leitura não vale nada.
//!
//! # A pergunta que este ficheiro existe para responder
//!
//! O `Spawner` precisa de um **tecto por tique** e o §0.0 proíbe escrevê-lo antes de o medir. A
//! suspeita, lida no código antes de medir: [`deep_copy_subtree`] chama
//! [`assign_missing_stable_ids`] **a cada cópia**, e essa porta faz duas varreduras do mundo
//! inteiro (o máximo dos ids + quem não tem id). Se for isso, o preço de nascer não é o tamanho da
//! receita — é o tamanho da CENA, e uma fábrica num mundo grande fica quadrática sem ninguém ver.
//!
//! ⇒ a tabela tem a coluna `só a identidade`, para que a atribuição seja **medida** em vez de
//! deduzida.

use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{
    ChildOf, Entity, Name, StableId, Transform, World, assign_missing_stable_ids,
    deep_copy_subtree, deep_copy_subtree_many,
};
use std::time::Instant;

const ITERS: usize = 9;

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

/// ⚠️ **O MÍNIMO ao lado da mediana, e não um deles sozinho** — a carga de fundo desta workstation
/// não desce abaixo de `~5` ([[feedback_the_background_load_of_this_workstation_never_falls_below_five]]),
/// então *esperar pela calma* nunca chega. O mínimo é a corrida menos interrompida; a mediana ao
/// lado é o que diz se a máquina estava a mentir.
fn minimo(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::INFINITY, f64::min)
}

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// **A RECEITA** — raiz + `pecas` filhos. `2` é a forma de um inimigo simples (corpo + arma).
fn receita_de(world: &mut World, pecas: u32) -> Entity {
    let root = world.spawn((Transform::IDENTITY, Name::new("Mob"))).id();
    for i in 0..pecas {
        world.spawn((
            Transform::IDENTITY,
            Name::new(format!("Peca{i}")),
            ChildOf(root),
        ));
    }
    root
}

/// `n` objectos de fundo — a CENA em que a fábrica trabalha.
fn povoar(world: &mut World, n: u32) {
    for i in 0..n {
        world.spawn((
            Transform::IDENTITY,
            Name::new(format!("obj{i}")),
            StableId(u64::from(i) + 1),
        ));
    }
}

/// O mundo de partida: a cena povoada, a receita, e toda a gente já com identidade — que é o
/// estado em que um quadro do app de facto chama a fábrica.
fn mundo(n: u32) -> (World, Entity) {
    mundo_com(n, 2)
}


fn mundo_com(n: u32, pecas: u32) -> (World, Entity) {
    let mut w = World::new();
    povoar(&mut w, n);
    let r = receita_de(&mut w, pecas);
    assign_missing_stable_ids(&mut w);
    (w, r)
}

/// **O controlo da sonda**: a cópia de facto põe três entidades no mundo, e elas nascem com
/// identidade própria. Sem isto, uma tabela de relógios podia estar a medir uma cópia vazia.
#[test]
fn the_probe_copies_three_entities_with_new_identity() {
    let (mut w, r) = mundo(8);
    let reg = reg();
    let antes = w.iter_entities().count();
    let c = deep_copy_subtree(&mut w, &reg, r, None).expect("copia");
    assign_missing_stable_ids(&mut w);
    assert_eq!(w.iter_entities().count(), antes + 3, "raiz + dois filhos");
    let id_novo = w.get::<StableId>(c.root).expect("id").0;
    let id_velho = w.get::<StableId>(r).expect("id").0;
    assert_ne!(
        id_novo, id_velho,
        "a copia nasceu com a identidade do mestre"
    );
}

/// ⭐⭐ **O preço de NASCER, e de quem ele é.**
#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_spawn() {
    let reg = reg();
    println!("load {:.2}", load_average());
    println!(
        "| cena (objectos) | copias no tique | so' a identidade (ms) | em SERIE (ms) | por copia (us) | em LOTE (ms) | por copia (us) | ganho |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        // O preço da porta da identidade sozinha, no mundo parado — a coluna que atribui a culpa.
        let mut so_id = Vec::with_capacity(ITERS);
        {
            let (mut w, _) = mundo(n);
            for _ in 0..ITERS {
                let t = Instant::now();
                std::hint::black_box(assign_missing_stable_ids(&mut w));
                so_id.push(t.elapsed().as_secs_f64() * 1e3);
            }
        }
        let so_id = minimo(&so_id);
        for &k in &[1u32, 10, 100, 256, 1_024, 4_096] {
            let mut copiar = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                // ⚠️ Mundo NOVO a cada iteração: as cópias da iteração anterior ficariam na cena
                // e a medição seguinte descreveria outro mundo.
                let (mut w, r) = mundo(n);
                let t = Instant::now();
                for _ in 0..k {
                    let _ = deep_copy_subtree(&mut w, &reg, r, None).expect("copia");
                }
                copiar.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let ms = minimo(&copiar);
            // ⭐ A MESMA medição pela porta em LOTE — a identidade paga-se uma vez.
            let mut lote = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                let (mut w, r) = mundo(n);
                let t = Instant::now();
                let _ = deep_copy_subtree_many(&mut w, &reg, r, None, k).expect("lote");
                lote.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let ml = minimo(&lote);
            println!(
                "| {n} | {k} | {so_id:.4} | {ms:.4} | {:.2} | {ml:.4} | {:.2} | {:.1}x |",
                ms * 1e3 / f64::from(k),
                ml * 1e3 / f64::from(k),
                ms / ml
            );
        }
    }
    // ⭐⭐ **De quem é o preço: da CENA ou da RECEITA?** — a coluna que a tabela de cima não tem.
    println!();
    println!("| cena | pecas na receita | copiar 10, min (ms) | por copia (us) | por PECA (us) |");
    println!("|---:|---:|---:|---:|---:|");
    for &pecas in &[2u32, 9, 29] {
        let mut copiar = Vec::with_capacity(ITERS);
        for _ in 0..ITERS {
            let (mut w, r) = mundo_com(10_000, pecas);
            let t = Instant::now();
            for _ in 0..10 {
                let _ = deep_copy_subtree(&mut w, &reg, r, None).expect("copia");
            }
            copiar.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let ms = minimo(&copiar);
        println!(
            "| 10000 | {} | {ms:.4} | {:.2} | {:.2} |",
            pecas + 1,
            ms * 1e3 / 10.0,
            ms * 1e3 / 10.0 / f64::from(pecas + 1)
        );
    }
    println!("load {:.2}", load_average());
}

/// ⭐ **O preço de MORRER** — tirar `k` raízes de três nós de uma cena de `n`.
#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_death() {
    let reg = reg();
    println!("load {:.2}", load_average());
    println!(
        "| cena (objectos) | mortes no tique | apagar, min (ms) | mediana (ms) | por morte, min (us) |"
    );
    println!("|---:|---:|---:|---:|---:|");
    for &n in &[1_000u32, 10_000] {
        for &k in &[10u32, 100, 1_000] {
            let mut apagar = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                let (mut w, r) = mundo(n);
                let vivos: Vec<Entity> = (0..k)
                    .map(|_| {
                        deep_copy_subtree(&mut w, &reg, r, None)
                            .expect("copia")
                            .root
                    })
                    .collect();
                let t = Instant::now();
                for e in &vivos {
                    w.entity_mut(*e).despawn();
                }
                apagar.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let ms = minimo(&apagar);
            println!(
                "| {n} | {k} | {ms:.4} | {:.4} | {:.2} |",
                median(apagar),
                ms * 1e3 / f64::from(k)
            );
        }
    }
    println!("load {:.2}", load_average());
}

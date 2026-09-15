//! ⭐⭐ **Os gates da porta em LOTE** (TOP-20 #11, W1) — irmão do [`super::tests`] por CAP de LOC.
//!
//! ⚠️ **Eles não são «mais uns testes da instanciação»:** a pergunta deles é a EQUIVALÊNCIA entre
//! duas portas que têm de entregar o mesmo mundo, e a medição que justifica a segunda existir.

use crate::instance_smoke::spawn_master;
use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId, Transform};

/// ⚠️ **Sem documentos vetoriais** — ver o irmão.
fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
) -> Result<Entity, super::Refusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
}

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    let mut r = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    r
}

/// ⭐⭐⭐ **A porta em LOTE deixa o mundo EXACTAMENTE como `n` chamadas à porta de um.**
///
/// ⚠️ **O oráculo é o que o artista vê**, objecto a objecto: o nome, o elo ao mestre, a ordem de
/// raiz e a forma da subárvore. Um gate que comparasse só a CONTAGEM ficaria verde sobre um lote
/// que desse a todas as cópias o mesmo nome — que é precisamente o defeito que a nomeação em lote
/// pode ter, porque ela reserva os nomes num conjunto próprio em vez de os ler do mundo.
///
/// ⚠️ **As três passagens `assign_*` saíram do laço**, e é este gate que prova que elas são
/// idempotentes: se não fossem, a `RootOrder` das cópias do lote não bateria com a das de série.
///
/// (Mutação: pôr as três de volta dentro do laço ⇒ VERDE — elas são mesmo idempotentes, e o que a
/// mutação inversa mostra é o PREÇO, não a lei. Tirar a inserção do nome no conjunto da porta em
/// lote ⇒ RED aqui.)
#[test]
fn the_batch_door_leaves_the_world_exactly_like_the_single_one() {
    let r = reg();
    // Em série.
    let mut a = SimWorld::new();
    let master_a = spawn_master(&mut a);
    let serie: Vec<Entity> = (0..3)
        .map(|_| instantiate(&mut a, &r, master_a, None).expect("instancia"))
        .collect();
    // Em lote.
    let mut b = SimWorld::new();
    let master_b = spawn_master(&mut b);
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let lote = super::instantiate_master_many(
        &mut b,
        &r,
        master_b,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
        3,
    )
    .expect("lote");

    let retrato = |sim: &SimWorld, e: Entity| {
        (
            sim.world().get::<Name>(e).map(|n| n.0.clone()),
            sim.world().get::<InstanceOf>(e).map(|i| i.master),
            sim.world()
                .get::<Children>(e)
                .map_or(0, |c| c.iter().count()),
            sim.world().get::<MasterRoot>(e).is_some(),
        )
    };
    let ra: Vec<_> = serie.iter().map(|&e| retrato(&a, e)).collect();
    let rb: Vec<_> = lote.iter().map(|&e| retrato(&b, e)).collect();
    assert_eq!(ra, rb, "o lote entregou outro mundo");
    // O controlo: os retratos NÃO são todos iguais entre si — senão a igualdade acima seria
    // trivial (três cópias indistinguíveis comparam iguais em qualquer ordem).
    assert_ne!(ra[0], ra[1], "a fixtura nao distingue as copias");

    // ⚠️⚠️ **A ORDEM DE RAIZ é a ÚNICA coisa que difere, e a diferença é da casa** — ver o corpo de
    // `instantiate_master_many`. O que se afirma dela é o que a casa de facto exige: ordens
    // **distintas e contíguas**, porque *«não se escolhe um desempate melhor, não se tem empate»*.
    let ordens = |sim: &SimWorld, v: &[Entity]| {
        let mut o: Vec<u32> = v
            .iter()
            .filter_map(|&e| sim.world().get::<ph2d_ecs::RootOrder>(e).map(|r| r.0))
            .collect();
        o.sort_unstable();
        o
    };
    assert_eq!(
        ordens(&a, &serie),
        ordens(&b, &lote),
        "as ordens do lote nao sao as mesmas, sem contar a atribuicao"
    );
    let o = ordens(&b, &lote);
    assert_eq!(o.len(), 3, "uma copia ficou sem ordem de raiz");
    assert!(
        o.windows(2).all(|w| w[1] == w[0] + 1),
        "ha' um empate ou um buraco: {o:?}"
    );
}

/// **Um lote de zero não faz nada, e não é um erro.**
#[test]
fn a_batch_of_zero_is_not_an_error() {
    let r = reg();
    let mut sim = SimWorld::new();
    let master = spawn_master(&mut sim);
    let antes = sim.world().iter_entities().count();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let v = super::instantiate_master_many(
        &mut sim,
        &r,
        master,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
        0,
    )
    .expect("zero");
    assert!(v.is_empty());
    assert_eq!(sim.world().iter_entities().count(), antes);
}

/// ⭐⭐⭐ **O PREÇO DE NASCER NA PORTA DO PRODUTO** — a medição que o
/// [`ph2d_ecs::BURST_MAX`] exige (`docs/Components/09_plano_spawner.md` §6.3).
///
/// ```text
/// cargo test -p ph2d-app-components --release --lib measure_instantiate -- --include-ignored --nocapture
/// ```
///
/// ⚠️ **`#[ignore]` de propósito** (família de flakes do `CLAUDE.md` §5.0): mede um RELÓGIO e
/// imprime o `load`. ⚠️ **Mínimo de N corridas, com a mediana ao lado** — a carga de fundo desta
/// máquina não desce abaixo de `~5`, então *esperar pela calma* nunca chega.
///
/// ⚠️⚠️ **É esta a porta do PRODUTO, e não a `deep_copy_subtree_many`**: a medição da cópia dizia
/// `2,6 µs` e o que o artista percorre passa também por `unique_name` e pelas três `assign_*`.
/// *Uma sonda que mede um sucedâneo para sempre mede outro programa.*
#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_instantiate_door() {
    use std::time::Instant;
    const ITERS: usize = 5;
    let load = || {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or(f64::NAN)
    };
    let minimo = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
        v[v.len() / 2]
    };
    let r = reg();
    println!("load {:.2}", load());
    println!("| cena | copias | em SERIE (ms) | us/copia | em LOTE (ms) | us/copia | ganho |");
    println!("|---:|---:|---:|---:|---:|---:|---:|");
    for &cena in &[1_000u32, 10_000] {
        for &k in &[16u32, 64, 256, 1_024] {
            let monta = || {
                let mut sim = SimWorld::new();
                for i in 0..cena {
                    sim.world_mut()
                        .spawn((Transform::IDENTITY, Name::new(format!("obj{i}"))));
                }
                let m = spawn_master(&mut sim);
                ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
                (sim, m)
            };
            let mut serie = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                let (mut sim, m) = monta();
                let t = Instant::now();
                for _ in 0..k {
                    let _ = instantiate(&mut sim, &r, m, None).expect("instancia");
                }
                serie.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let mut lote = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                let (mut sim, m) = monta();
                let (mut sc, mut mp) = crate::instance_docs::empty_docs();
                let t = Instant::now();
                let _ = super::instantiate_master_many(
                    &mut sim,
                    &r,
                    m,
                    None,
                    &mut crate::instance_docs::OwnedDocs {
                        vec_scene: &mut sc,
                        vec_entities: &mut mp,
                    },
                    crate::instantiate::ArtLink::Own,
                    k,
                )
                .expect("lote");
                lote.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let (ms, ml) = (minimo(&serie), minimo(&lote));
            println!(
                "| {cena} | {k} | {ms:.3} | {:.1} | {ml:.3} | {:.1} | {:.1}x |",
                ms * 1e3 / f64::from(k),
                ml * 1e3 / f64::from(k),
                ms / ml
            );
            let _ = (mediana(serie), mediana(lote));
        }
    }
    println!("load {:.2}", load());
}

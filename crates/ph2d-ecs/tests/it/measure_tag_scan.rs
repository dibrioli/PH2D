//! ⭐ **A SONDA DA CONSULTA POR TAG** — quanto custa perguntar *«quem pertence à tag `Enemy`, com os
//! descendentes dela?»* varrendo o mundo (`docs/Components/08_plano_tags.md` §6).
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test it measure_tag_scan -- --include-ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]` de propósito** (família de flakes do `CLAUDE.md` §5.0): isto mede um RELÓGIO, e
//! imprime o `load` que encontrou antes e depois da tabela.
//!
//! # A pergunta que este ficheiro existe para responder
//!
//! O `SignalActions` resolve o alvo por NOME com **duas varreduras lineares**
//! (`stable_id_for_name` + `entity_of_stable_id`), sem índice **de propósito** — o doc de
//! `stable_id.rs` diz porquê: *um mapa seria estado derivado a manter coerente com o mundo depois de
//! todo restore do undo*. A consulta por tag tem a MESMA escolha à frente: varrer a cada sinal, ou
//! manter um índice (o Godot mantém um por `SceneTree`). ⛔ Esta sonda mede se a varredura cabe no
//! orçamento — e compara-a com a varredura por nome que já shipa.
//!
//! # ⚠️ Desde a W1 ela mede a PORTA REAL, não um sucedâneo
//!
//! A 1.ª versão (antes do código) usava um componente local e uma expansão de subárvore escrita à
//! mão, e foi com ela que o plano §6.1 decidiu *«sem índice»*. A W1 trocou os dois pela porta que
//! shipa — [`tagged`] e a [`TagTree`] com a dobra —, então o número passa a incluir o que o sucedâneo
//! não pagava: a expansão da subárvore com a dobra ICU **em cada consulta**. *Uma sonda que mede um
//! sucedâneo para sempre mede outro programa.*

use ph2d_ecs::tags::{Tags, tagged};
use ph2d_ecs::{Entity, Name, StableId, World, entity_of_stable_id, stable_id_for_name};
use ph2d_tags::{TagId, TagTree};
use std::time::Instant;

const ITERS: usize = 25;

/// A árvore da sonda: `Enemy` › `Flying` › `Boss`, e a raiz irmã `Statue`.
struct Arvore {
    tree: TagTree,
    enemy: TagId,
    flying: TagId,
    boss: TagId,
    statue: TagId,
}

fn arvore() -> Arvore {
    let mut tree = TagTree::new();
    let boss = tree.create("Enemy/Flying/Boss").expect("cria");
    let enemy = tree.find("Enemy").expect("ancestral");
    let flying = tree.find("Enemy/Flying").expect("ancestral");
    let statue = tree.create("Statue").expect("cria");
    Arvore {
        tree,
        enemy,
        flying,
        boss,
        statue,
    }
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

/// `n` objectos com nome e identidade; um em cada `k` tem tags, repartidas por três casos — o que
/// pertence à raiz, o que pertence por HIERARQUIA, e a raiz IRMÃ que não pode casar.
fn povoar(world: &mut World, a: &Arvore, n: u32, k: u32) {
    for i in 0..n {
        let mut e = world.spawn((Name::new(format!("obj{i}")), StableId(u64::from(i) + 1)));
        if i % k == 0 {
            let tag = match (i / k) % 3 {
                0 => a.enemy,
                1 => a.boss,
                _ => a.statue,
            };
            e.insert(Tags::from_ids([tag]));
        }
    }
}

fn consulta(world: &World, a: &Arvore, q: TagId) -> Vec<Entity> {
    tagged(world, &a.tree, q)
}

#[test]
fn the_probe_reaches_the_subtree_and_not_the_sibling_root() {
    let a = arvore();
    let mut w = World::new();
    povoar(&mut w, &a, 30, 1);
    assert_eq!(
        consulta(&w, &a, a.enemy).len(),
        20,
        "10 na raiz + 10 por hierarquia"
    );
    assert_eq!(consulta(&w, &a, a.flying).len(), 10, "so' os Boss");
    assert_eq!(consulta(&w, &a, a.statue).len(), 10, "a raiz irma' e' dela");
}

#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_tag_scan() {
    let a = arvore();
    println!("load {:.2}", load_average());
    println!("| N | com tags | acertos | consulta por tag (ms) | alvo por NOME, hoje (ms) |");
    println!("|---:|---:|---:|---:|---:|");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        for &k in &[10u32, 1] {
            let mut world = World::new();
            povoar(&mut world, &a, n, k);
            let _ = consulta(&world, &a, a.enemy);
            let mut por_tag = Vec::with_capacity(ITERS);
            let mut acertos = 0;
            for _ in 0..ITERS {
                let t = Instant::now();
                let h = consulta(&world, &a, a.enemy);
                por_tag.push(t.elapsed().as_secs_f64() * 1e3);
                acertos = std::hint::black_box(h).len();
            }
            let alvo = format!("obj{}", n / 2);
            let mut por_nome = Vec::with_capacity(ITERS);
            for _ in 0..ITERS {
                let t = Instant::now();
                let id = stable_id_for_name(&mut world, &alvo);
                let e = entity_of_stable_id(&mut world, StableId(id));
                por_nome.push(t.elapsed().as_secs_f64() * 1e3);
                std::hint::black_box(e);
            }
            println!(
                "| {n} | {} | {acertos} | {:.4} | {:.4} |",
                n.div_ceil(k),
                median(por_tag),
                median(por_nome)
            );
        }
    }
    // ⚠️ E o que o sucedâneo não pagava, à parte: só a expansão da subárvore, com a dobra.
    let mut expande = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let t = Instant::now();
        std::hint::black_box(a.tree.subtree(a.enemy));
        expande.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "subtree(Enemy) numa arvore de 4 tags: {:.5} ms",
        median(expande)
    );
    println!("load {:.2}", load_average());
}

/// Uma árvore de `n` tags em três níveis (`Raiz r/Filha c/Neta g`, 8 filhas e 8 netas por raiz), dada
/// como um DOCUMENTO — a forma em que o load a recebe.
fn documento(n: usize) -> Vec<ph2d_tags::Tag> {
    let mut out = Vec::with_capacity(n);
    let mut id = 1u64;
    'fora: for r in 0.. {
        for c in 0..8 {
            for g in 0..8 {
                if out.len() >= n {
                    break 'fora;
                }
                out.push(ph2d_tags::Tag {
                    id: TagId(id),
                    path: format!("Raiz {r}/Filha {c}/Neta {g}"),
                });
                id += 1;
            }
        }
    }
    out
}

/// ⭐⭐ **O que a porta custa quando a ÁRVORE cresce** — a pergunta que a tabela de cima não faz (ela
/// tem 4 tags). Três relógios, porque são três chamadores com frequências diferentes:
///
/// - `restore` — o LOAD de um projecto (uma vez por abertura);
/// - `subtree` — cada consulta de `tagged` (uma por sinal por tag);
/// - `belongs` — o filtro da física (uma por evento de colisão com filtro, W2).
///
/// ⚠️ A árvore de `n` tags do `documento` materializa também os ancestrais (`Raiz r`, `Raiz r/Filha
/// c`), então a coluna `tags` é a contagem REAL depois do `restore`, não o `n` pedido.
#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_tag_tree_scale() {
    println!("load {:.2}", load_average());
    println!(
        "| n pedido | tags | restore (ms) | subtree(Raiz 0) (ms) | belongs, 1 objecto (ms) | belongs, a ULTIMA tag (ms) |"
    );
    println!("|---:|---:|---:|---:|---:|---:|");
    for &n in &[8usize, 64, 512, 2048, 8192] {
        let doc = documento(n);
        let iters = if n >= 512 { 5 } else { ITERS };
        let mut restaura = Vec::with_capacity(iters);
        let mut tree = TagTree::new();
        for _ in 0..iters {
            let d = doc.clone();
            let t = Instant::now();
            let (arvore, _) = TagTree::restore(d, 0);
            restaura.push(t.elapsed().as_secs_f64() * 1e3);
            tree = arvore;
        }
        let raiz = tree.find("Raiz 0").expect("a raiz 0 existe");
        let neta = tree.find("Raiz 0/Filha 0/Neta 0").expect("a neta existe");
        let objecto = Tags::from_ids([neta]);
        // ⚠️ O PIOR caso da porta: as duas procuras por id são lineares, e a tag e a raiz dela no
        // FIM da ordem da árvore são as que cada procura mais demora a achar. A coluna ao lado (a
        // neta da `Raiz 0`) é o melhor caso, e sem esta a tabela mostrava só esse.
        let ultima = tree.tags().last().expect("a arvore nao e' vazia").clone();
        let raiz_da_ultima = tree
            .find(ultima.path.split('/').next().expect("um nivel"))
            .expect("a raiz dela existe");
        let objecto_do_fim = Tags::from_ids([ultima.id]);
        let mut expande = Vec::with_capacity(ITERS);
        let mut pertence = Vec::with_capacity(ITERS);
        let mut pertence_fim = Vec::with_capacity(ITERS);
        for _ in 0..ITERS {
            let t = Instant::now();
            std::hint::black_box(tree.subtree(raiz));
            expande.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            std::hint::black_box(ph2d_ecs::tags::belongs(&objecto, &tree, raiz));
            pertence.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            let sim = ph2d_ecs::tags::belongs(&objecto_do_fim, &tree, raiz_da_ultima);
            pertence_fim.push(t.elapsed().as_secs_f64() * 1e3);
            assert!(
                std::hint::black_box(sim),
                "a ultima tag pertence a raiz dela"
            );
        }
        println!(
            "| {n} | {} | {:.3} | {:.5} | {:.5} | {:.5} |",
            tree.tags().len(),
            median(restaura),
            median(expande),
            median(pertence),
            median(pertence_fim)
        );
    }
    println!("load {:.2}", load_average());
}

/// ⭐⭐⭐ **O PREÇO DA COLUNA DO PAINEL** (W4) — *«quantos objectos por tag»*, para TODAS as tags.
///
/// O painel *Tags* repinta a cada quadro e precisa das `N` contagens juntas. Há duas formas, e esta
/// sonda mede as duas sobre o mesmo mundo:
///
/// - **`counts`** — uma passagem pelo mundo; cada objecto sobe a própria ancestralidade.
/// - **`tagged` em laço** — uma varredura do mundo POR TAG, que é o que já shipa para uma tag só.
///
/// ⚠️ **A segunda coluna não é um espantalho:** era a implementação óbvia, e é a que estaria lá se
/// ninguém tivesse medido. O que a torna inviável não é a constante — é o produto `tags × objectos`,
/// que num painel é pago **por quadro**.
///
/// ⚠️ **`#[ignore]`** pela razão das irmãs: mede um relógio, e imprime o `load`.
#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_tag_counts() {
    println!("load {:.2}", load_average());
    println!("| objectos | com tags | tags | counts (ms) | tagged em laco (ms) | razao |");
    println!("|---:|---:|---:|---:|---:|---:|");
    for &(n, k) in &[(1_000u32, 10u32), (10_000, 10), (10_000, 1), (100_000, 10)] {
        for &t in &[8usize, 512] {
            let doc = documento(t);
            let (tree, _) = TagTree::restore(doc, 0);
            let tags: Vec<TagId> = tree.tags().map(|g| g.id).collect();
            let mut world = World::new();
            for i in 0..n {
                let mut e = world.spawn((Name::new(format!("obj{i}")), StableId(u64::from(i) + 1)));
                if i % k == 0 {
                    e.insert(Tags::from_ids([
                        tags[(i as usize / k as usize) % tags.len()]
                    ]));
                }
            }
            let iters = if n >= 100_000 { 5 } else { ITERS };
            let mut rapido = Vec::with_capacity(iters);
            for _ in 0..iters {
                let c = Instant::now();
                std::hint::black_box(ph2d_ecs::tags::counts(&world, &tree));
                rapido.push(c.elapsed().as_secs_f64() * 1e3);
            }
            // ⚠️ O laço lento corre MENOS vezes quando é caro — senão a sonda sozinha passa minutos.
            let iters_lento = if tags.len() * n as usize > 2_000_000 {
                1
            } else {
                iters
            };
            let mut lento = Vec::with_capacity(iters_lento);
            for _ in 0..iters_lento {
                let c = Instant::now();
                let mut soma = 0usize;
                for &q in &tags {
                    soma += tagged(&world, &tree, q).len();
                }
                std::hint::black_box(soma);
                lento.push(c.elapsed().as_secs_f64() * 1e3);
            }
            let (a, b) = (median(rapido), median(lento));
            println!(
                "| {n} | {} | {} | {a:.4} | {b:.4} | {:.1}x |",
                n.div_ceil(k),
                tags.len(),
                b / a
            );
        }
    }
    println!("load {:.2}", load_average());
}

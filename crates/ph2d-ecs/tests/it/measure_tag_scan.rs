//! ⭐ **A SONDA DA CONSULTA POR TAG** — quanto custa perguntar *«quem tem a tag `enemy`?»* varrendo
//! o mundo, ANTES de a feature existir (`docs/Components/08_plano_tags.md` §6).
//!
//! ```text
//! cargo test -p ph2d-ecs --release --test it measure_tag_scan -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]` de propósito** (família de flakes do `CLAUDE.md` §5.0): isto mede um RELÓGIO, e
//! imprime o `load` que encontrou ao lado de cada tabela.
//!
//! # A pergunta que este ficheiro existe para responder
//!
//! O `SignalActions` resolve o alvo por NOME com **duas varreduras lineares**
//! (`stable_id_for_name` + `entity_of_stable_id`), sem índice **de propósito** — o doc de
//! `stable_id.rs` diz porquê: *um mapa seria estado derivado a manter coerente com o mundo depois de
//! todo restore do undo*. A consulta por tag tem a MESMA escolha à frente: varrer a cada sinal, ou
//! manter um índice (o Godot mantém um por `SceneTree`). ⛔ Esta sonda mede se a varredura cabe no
//! orçamento **antes** de se escolher — e compara-a com a varredura por nome que já shipa.
//!
//! ⚠️ **O componente aqui é um SUCEDÂNEO local** (`SondaTags`), com a forma que o plano propõe
//! (`BTreeSet<String>` por entidade) e a lei de casamento hierárquico dele (`t == q` ou `t` começa
//! por `q` seguido de `.`). Ele não é o produto; a W1 troca-o pela porta real e a sonda fica.

use bevy_ecs::component::Component;
use ph2d_ecs::{Entity, Name, StableId, World, entity_of_stable_id, stable_id_for_name};
use std::collections::BTreeSet;
use std::time::Instant;

const ITERS: usize = 25;

#[derive(Component)]
struct SondaTags(BTreeSet<String>);

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

/// O casamento do plano: `enemy` casa `enemy` e `enemy.flying.boss`, e NUNCA `enemyx`.
fn casa(tag: &str, q: &str) -> bool {
    tag == q || (tag.len() > q.len() && tag.starts_with(q) && tag.as_bytes()[q.len()] == b'.')
}

/// `n` objectos com nome e identidade; um em cada `k` tem tags, repartidas por três casos — o que
/// casa exacto, o que casa por hierarquia, e o PREFIXO DE TEXTO que não pode casar.
fn povoar(world: &mut World, n: u32, k: u32) {
    for i in 0..n {
        let mut e = world.spawn((Name::new(format!("obj{i}")), StableId(u64::from(i) + 1)));
        if i % k == 0 {
            let tag = match (i / k) % 3 {
                0 => "enemy",
                1 => "enemy.flying.boss",
                _ => "enemyx",
            };
            e.insert(SondaTags(BTreeSet::from([tag.to_string()])));
        }
    }
}

/// A porta proposta, varrendo: quem casa `q`, na ordem da IDENTIDADE (nunca a da query).
fn consulta(world: &mut World, q: &str) -> Vec<Entity> {
    let mut hits: Vec<(u64, Entity)> = world
        .query::<(Entity, &SondaTags, &StableId)>()
        .iter(world)
        .filter(|(_, t, _)| t.0.iter().any(|x| casa(x, q)))
        .map(|(e, _, s)| (s.0, e))
        .collect();
    hits.sort_unstable_by_key(|(id, _)| *id);
    hits.into_iter().map(|(_, e)| e).collect()
}

#[test]
fn the_probe_matches_the_hierarchy_and_not_the_text_prefix() {
    assert!(casa("enemy", "enemy"));
    assert!(casa("enemy.flying.boss", "enemy"));
    assert!(casa("enemy.flying.boss", "enemy.flying"));
    assert!(
        !casa("enemyx", "enemy"),
        "um prefixo de TEXTO nao e' um pai"
    );
    assert!(!casa("enemy", "enemy.flying"), "o pai nao casa o filho");
    let mut w = World::new();
    povoar(&mut w, 30, 1);
    assert_eq!(
        consulta(&mut w, "enemy").len(),
        20,
        "10 exactos + 10 por hierarquia"
    );
}

#[test]
#[ignore = "mede um relogio -- corra com --ignored --nocapture numa maquina calma"]
fn measure_tag_scan() {
    println!("load {:.2}", load_average());
    println!("| N | com tags | acertos | consulta por tag (ms) | alvo por NOME, hoje (ms) |");
    println!("|---:|---:|---:|---:|---:|");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        for &k in &[10u32, 1] {
            let mut world = World::new();
            povoar(&mut world, n, k);
            let _ = consulta(&mut world, "enemy");
            let mut por_tag = Vec::with_capacity(ITERS);
            let mut acertos = 0;
            for _ in 0..ITERS {
                let t = Instant::now();
                let h = consulta(&mut world, "enemy");
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
    println!("load {:.2}", load_average());
}

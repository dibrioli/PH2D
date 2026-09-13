//! ⭐ **A SONDA DA CONSULTA POR TAG** — quanto custa perguntar *«quem pertence à tag `Enemy`, com os
//! descendentes dela?»* varrendo o mundo, ANTES de a feature existir
//! (`docs/Components/08_plano_tags.md` §6).
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
//! orçamento **antes** de se escolher — e compara-a com a varredura por nome que já shipa.
//!
//! # ⚠️ O MODELO que ela mede é o das decisões do dono (2026-09-13), não o da 1.ª redacção
//!
//! A 1.ª versão desta sonda casava STRINGS por prefixo de segmento (`enemy` apanhava
//! `enemy.flying.boss`). O dono escolheu o modelo do Blender — **a pertença é uma IDENTIDADE e a
//! hierarquia vive na árvore** — ⇒ um objecto carrega um conjunto de ids, e a consulta expande
//! primeiro a SUBÁRVORE da tag pedida e depois varre. O componente aqui é um SUCEDÂNEO local
//! (`SondaTags(BTreeSet<u64>)`); a W1 troca-o pela porta real e a sonda fica.

use bevy_ecs::component::Component;
use ph2d_ecs::{Entity, Name, StableId, World, entity_of_stable_id, stable_id_for_name};
use std::collections::BTreeSet;
use std::time::Instant;

const ITERS: usize = 25;

/// A árvore da sonda: `1 = Enemy`, `2 = Enemy/Flying`, `3 = Enemy/Flying/Boss`, `4 = Statue` (raiz
/// irmã). A subárvore de `Enemy` é `{1, 2, 3}`.
const ENEMY: u64 = 1;
const FLYING: u64 = 2;
const BOSS: u64 = 3;
const STATUE: u64 = 4;

#[derive(Component)]
struct SondaTags(BTreeSet<u64>);

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

/// A expansão da árvore da sonda — o que o `TagTree::subtree` fará. ⚠️ Fica DENTRO da medição: a
/// porta real expande a cada consulta, porque guardar a expansão seria o índice que se recusa.
fn subarvore(q: u64) -> BTreeSet<u64> {
    match q {
        ENEMY => BTreeSet::from([ENEMY, FLYING, BOSS]),
        FLYING => BTreeSet::from([FLYING, BOSS]),
        other => BTreeSet::from([other]),
    }
}

/// `n` objectos com nome e identidade; um em cada `k` tem tags, repartidas por três casos — o que
/// pertence à raiz, o que pertence por HIERARQUIA, e a raiz IRMÃ que não pode casar.
fn povoar(world: &mut World, n: u32, k: u32) {
    for i in 0..n {
        let mut e = world.spawn((Name::new(format!("obj{i}")), StableId(u64::from(i) + 1)));
        if i % k == 0 {
            let tag = match (i / k) % 3 {
                0 => ENEMY,
                1 => BOSS,
                _ => STATUE,
            };
            e.insert(SondaTags(BTreeSet::from([tag])));
        }
    }
}

/// A porta proposta, varrendo: quem pertence à subárvore de `q`, na ordem da IDENTIDADE.
fn consulta(world: &mut World, q: u64) -> Vec<Entity> {
    let alvo = subarvore(q);
    let mut hits: Vec<(u64, Entity)> = world
        .query::<(Entity, &SondaTags, &StableId)>()
        .iter(world)
        .filter(|(_, t, _)| !t.0.is_disjoint(&alvo))
        .map(|(e, _, s)| (s.0, e))
        .collect();
    hits.sort_unstable_by_key(|(id, _)| *id);
    hits.into_iter().map(|(_, e)| e).collect()
}

#[test]
fn the_probe_reaches_the_subtree_and_not_the_sibling_root() {
    let mut w = World::new();
    povoar(&mut w, 30, 1);
    assert_eq!(
        consulta(&mut w, ENEMY).len(),
        20,
        "10 na raiz + 10 por hierarquia"
    );
    assert_eq!(consulta(&mut w, FLYING).len(), 10, "so' os Boss");
    assert_eq!(consulta(&mut w, STATUE).len(), 10, "a raiz irma' e' dela");
}

#[test]
#[ignore = "mede um relogio -- corra com --include-ignored --nocapture numa maquina calma"]
fn measure_tag_scan() {
    println!("load {:.2}", load_average());
    println!("| N | com tags | acertos | consulta por tag (ms) | alvo por NOME, hoje (ms) |");
    println!("|---:|---:|---:|---:|---:|");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        for &k in &[10u32, 1] {
            let mut world = World::new();
            povoar(&mut world, n, k);
            let _ = consulta(&mut world, ENEMY);
            let mut por_tag = Vec::with_capacity(ITERS);
            let mut acertos = 0;
            for _ in 0..ITERS {
                let t = Instant::now();
                let h = consulta(&mut world, ENEMY);
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

//! Os gates da cena `=63` — a ordem.
//!
//! ⚠️ **O oráculo é a PERMUTAÇÃO, não a cor**: a cena pinta para o olho, mas o que ela afirma
//! é que as três chaves produzem três ordens DIFERENTES sobre o MESMO conjunto de posições.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// As posições de cada banda, na ordem em que elas saem.
fn bands() -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sortkey_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 3, "três bandas");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            match cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze")[0]
                .as_stream()
                .get("P")
            {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// **AS TRÊS ORDENS SÃO TRÊS**, e o CONJUNTO de posições é o mesmo nas três.
///
/// ⚠️ As duas metades: se as ordens coincidissem a cena não mostraria nada; se o conjunto
/// mudasse, a banda estaria a mover peças em vez de as reordenar — e a mensagem do smoke
/// (*"as peças NÃO se movem entre as bandas"*) seria falsa.
#[test]
fn the_three_keys_give_three_orders_over_the_same_points() {
    let b = bands();
    let n = (SIDE * SIDE) as usize;
    for (i, band) in b.iter().enumerate() {
        assert_eq!(band.len(), n, "a banda {i} tem {n} peças");
    }
    // O MESMO conjunto: cada banda é uma permutação das outras (o `dy` do layout é comum a
    // todas dentro da banda, então comparamos o X, que ele não toca).
    let sorted_x = |v: &Vec<[f32; 2]>| {
        let mut xs: Vec<f32> = v.iter().map(|q| q[0]).collect();
        xs.sort_by(f32::total_cmp);
        xs
    };
    assert_eq!(sorted_x(&b[0]), sorted_x(&b[1]), "mesmo conjunto (1 vs 2)");
    assert_eq!(sorted_x(&b[0]), sorted_x(&b[2]), "mesmo conjunto (1 vs 3)");
    // E as três ORDENS diferem.
    let xs = |v: &Vec<[f32; 2]>| v.iter().map(|q| q[0]).collect::<Vec<_>>();
    assert_ne!(xs(&b[0]), xs(&b[1]), "X contra a diagonal");
    assert_ne!(xs(&b[0]), xs(&b[2]), "X contra o campo");
    assert_ne!(xs(&b[1]), xs(&b[2]), "a diagonal contra o campo");
}

/// **A BANDA 1 ESTÁ DE FACTO ORDENADA POR X** — a âncora que dá sentido às outras duas.
///
/// ⚠️ Sem ela, três listas diferentes provariam só que três números diferentes produzem três
/// resultados diferentes.
#[test]
fn the_first_band_really_runs_left_to_right() {
    let b = bands();
    let xs: Vec<f32> = b[0].iter().map(|q| q[0]).collect();
    assert!(
        xs.windows(2).all(|w| w[0] <= w[1]),
        "a banda 1 tem de sair em X crescente: {xs:?}"
    );
    // E o CONTROLE: a diagonal NÃO sai em X crescente (senão o ângulo não fez nada).
    let d: Vec<f32> = b[1].iter().map(|q| q[0]).collect();
    assert!(
        !d.windows(2).all(|w| w[0] <= w[1]),
        "a banda 2 não pode sair em X crescente — o ângulo tem de virar a ordem"
    );
}

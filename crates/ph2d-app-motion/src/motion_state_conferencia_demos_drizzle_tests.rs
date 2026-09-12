//! Os gates da cena `=67` — a chuva rala.
//!
//! ⚠️ **O par tem de separar pela IRREGULARIDADE, não pela contagem.** Um `rate` a 40% também
//! dá 40% das partículas; o que ele não dá são buracos de tamanhos diferentes. É isso que o
//! oráculo mede.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Os `id` vivos de cada jacto no instante `t`.
fn ids(t: f32) -> Vec<Vec<u32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_drizzle_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 2, "dois jactos");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            match cook.cook(&doc.graph, &reg, *s, f64::from(t)).expect("coze")[0]
                .as_stream()
                .get("id")
            {
                #[expect(clippy::cast_possible_truncation, reason = "ids inteiros num f32")]
                #[expect(clippy::cast_sign_loss, reason = "um id nunca é negativo")]
                Some(Column::Scalar(v)) => v.iter().map(|x| *x as u32).collect(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// **O JACTO RALO TEM MENOS, E É UM SUBCONJUNTO DO CHEIO** — o mesmo `rate`, o mesmo `seed`.
#[test]
fn the_thin_jet_is_a_subset_of_the_full_one_at_the_same_rate() {
    let b = ids(5.0);
    let (full, thin) = (&b[0], &b[1]);
    assert!(
        full.len() > 100,
        "o jacto cheio tem de estar cheio: {}",
        full.len()
    );
    let ratio = thin.len() as f32 / full.len() as f32;
    assert!(
        (ratio - THIN).abs() < 0.12,
        "a fracção viva tem de ficar perto de {THIN}, e deu {ratio:.3}"
    );
    for id in thin {
        assert!(full.contains(id), "o id {id} não existe no jacto cheio");
    }
}

/// **OS BURACOS SÃO IRREGULARES** — e é isto que separa a probabilidade de um `rate` menor.
///
/// ⚠️ O oráculo é a variedade dos VÃOS entre ids consecutivos. Um `rate` a 40% daria um vão
/// constante (todos iguais); a probabilidade dá vãos de 1, 2, 3, … O gate exige pelo menos
/// três tamanhos distintos, e o controle é o jacto cheio, onde o vão é sempre 1.
#[test]
fn the_holes_are_irregular_which_a_lower_rate_could_never_be() {
    let b = ids(5.0);
    let gaps = |v: &Vec<u32>| {
        let mut g: Vec<u32> = v.windows(2).map(|w| w[1] - w[0]).collect();
        g.sort_unstable();
        g.dedup();
        g
    };
    assert_eq!(gaps(&b[0]), vec![1], "no jacto cheio o vão é sempre 1");
    let thin = gaps(&b[1]);
    assert!(
        thin.len() >= 3,
        "os buracos têm de ter tamanhos diferentes, e só apareceram {thin:?}"
    );
}

/// **A CHUVA NÃO CINTILA** — a mesma partícula está viva ou morta nos dois instantes.
///
/// ⚠️ É a afirmação do produto, medida na CENA e não só na crate: se um dia alguém trocar o
/// sorteio por um hash do índice, este gate é o que reprova.
#[test]
fn a_drop_does_not_blink_as_the_window_slides() {
    let (a, b) = (ids(5.0), ids(5.37));
    assert_ne!(a[0], b[0], "a janela TEM de ter deslizado");
    let overlap: Vec<u32> = a[0].iter().copied().filter(|k| b[0].contains(k)).collect();
    assert!(
        overlap.len() > 20,
        "sobreposição pequena demais: {}",
        overlap.len()
    );
    for id in overlap {
        assert_eq!(
            a[1].contains(&id),
            b[1].contains(&id),
            "a gota {id} mudou de resposta entre os dois instantes"
        );
    }
}

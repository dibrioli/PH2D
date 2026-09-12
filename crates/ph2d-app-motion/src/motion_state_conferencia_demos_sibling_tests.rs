//! Gates da cena `=91` — **o irmão sabe e ele não** (doc 89, folhas 05 e 14).
//!
//! ⚠️ **A quarta linha é uma FONTE que lê por canal externo**, então num cook virgem ela emite
//! zero. Medir as posições dela aqui seria a sonda a acusar-se a si própria (o precedente do
//! `measure_scene_layout`) — o gate dela mede o DOCUMENTO: que a cena autora os params que a
//! célula fechou, e que o lado esquerdo NÃO os autora.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sibling_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As posições de uma banda, já sem o deslocamento do quadrante.
fn shape(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cozinha");
    let Some(Column::Vec2(p)) = out[0].as_stream().get("P") else {
        return Vec::new();
    };
    if p.is_empty() {
        return Vec::new();
    }
    let n = p.len() as f32;
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let c = [c[0] / n, c[1] / n];
    p.iter().map(|q| [q[0] - c[0], q[1] - c[1]]).collect()
}

/// A largura e a altura da figura.
fn extents(p: &[[f32; 2]]) -> (f32, f32) {
    let f = |a: usize| {
        p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
            (lo.min(q[a]), hi.max(q[a]))
        })
    };
    let (x, y) = (f(0), f(1));
    (x.1 - x.0, y.1 - y.0)
}

/// **A CENA MONTA AS OITO BANDAS**, e as três primeiras linhas cospem.
#[test]
fn the_sibling_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    for (k, &sink) in sinks.iter().take(6).enumerate() {
        assert!(
            !shape(&doc, &reg, sink).is_empty(),
            "banda {k} vazia -- as tres primeiras linhas nao dependem do shell"
        );
    }
}

/// ⭐ **O par 1: desligar o link estica a LARGURA e deixa a altura quieta.**
#[test]
fn unlinking_the_chain_spreads_only_one_axis() {
    let (doc, reg, sinks) = scene();
    let (w0, h0) = extents(&shape(&doc, &reg, sinks[0]));
    let (w1, h1) = extents(&shape(&doc, &reg, sinks[1]));
    assert!(
        (w1 - w0).abs() < 1e-3,
        "a LARGURA e' a mesma nos dois (os dois autoram o mesmo `scale` em X): {w0} vs {w1}"
    );
    assert!(
        h1 < h0 * 0.5,
        "e a ALTURA da direita tinha de ficar por escalar: {h0} vs {h1}"
    );
}

/// ⭐ **O par 2: o FLIP espelha e não encolhe.**
#[test]
fn the_flip_mirrors_without_shrinking() {
    let (doc, reg, sinks) = scene();
    let plain = shape(&doc, &reg, sinks[2]);
    let flipped = shape(&doc, &reg, sinks[3]);
    let (w0, h0) = extents(&plain);
    let (w1, h1) = extents(&flipped);
    assert!(
        (w0 - w1).abs() < 1e-3 && (h0 - h1).abs() < 1e-3,
        "a caixa e' a MESMA: {w0}x{h0} contra {w1}x{h1}"
    );
    // E cada elemento trocou o sinal do seu `y`.
    for (i, (a, b)) in plain.iter().zip(&flipped).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] + b[1]).abs() < 1e-4,
            "elemento {i}: {a:?} nao e' o espelho de {b:?}"
        );
    }
}

/// ⭐ **O par 3: `Reflection Only` fica com METADE das peças.**
#[test]
fn keeping_only_the_reflection_halves_the_count() {
    let (doc, reg, sinks) = scene();
    let both = shape(&doc, &reg, sinks[4]);
    let only = shape(&doc, &reg, sinks[5]);
    assert_eq!(
        only.len() * 2,
        both.len(),
        "metade das pecas: {} contra {}",
        only.len(),
        both.len()
    );
    // E o que sobrou é a metade ESPELHADA — a caixa dela é a de um lado só.
    //
    // ⚠️ **A régua é a LARGURA, e a primeira versão olhou a altura.** O `axis = 0` deste nó é
    // a linha de espelho VERTICAL (reflecte o `x`), então as duas metades ficam lado a lado:
    // o par inteiro é mais LARGO, e as alturas são iguais por construção (`1` contra `1`,
    // medido). *Uma régua no eixo errado acusa código correcto de não fazer nada.*
    let (w_both, _) = extents(&both);
    let (w_only, _) = extents(&only);
    assert!(
        w_only < w_both * 0.7,
        "a metade espelhada ocupa menos que o par inteiro: {w_only} contra {w_both}"
    );
}

/// **O par 4 é medido no DOCUMENTO** — ver o cabeçalho: a forma lê por canal externo e um cook
/// virgem não tem nenhum.
#[test]
fn the_shape_pair_authors_its_own_fill_and_rotation_on_one_side_only() {
    let (doc, _reg, _sinks) = scene();
    let shapes: Vec<NodeId> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("source.shape"))
        .map(|n| n.id)
        .collect();
    assert_eq!(shapes.len(), 2, "duas formas, uma por lado");
    let read = |n: NodeId, p: &str| {
        doc.graph
            .node_param_overrides(n)
            .and_then(|o| o.get(p).copied())
            .unwrap_or(0.0)
    };
    // A ordem de construção põe a esquerda primeiro.
    assert!(
        read(shapes[0], "fill") < 0.5 && read(shapes[0], "rotation").abs() < 1e-6,
        "a ESQUERDA e' o controle: nem cor propria nem rotacao"
    );
    assert!(
        read(shapes[1], "fill") >= 0.5 && read(shapes[1], "rotation").abs() > 1.0,
        "e a DIREITA autora as duas"
    );
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 8, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}

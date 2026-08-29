//! **O L-SYSTEM EMITE UM ESQUELETO A SÉRIO** — e a prova é o `rig.fk` não lhe tocar.
//!
//! # Por que este gate vive aqui e não no crate do nó
//!
//! O `source.lsystem` afirma, no doc dele, que as colunas que emite são *"exactamente o
//! contrato do `rig.*`"* e que a posição é calculada *"como o `rig.fk` a calcula"*. As duas
//! frases são afirmações sobre **outro módulo**, e afirmações medem-se. O crate do nó não
//! pode medi-las: os dois são drop-crates e não se conhecem (ADR-0075). Esta crate é a que vê
//! os dois.
//!
//! # A régua é a IDENTIDADE, e é a mais forte que existe aqui
//!
//! `rig.fk` recalcula `P` e `wrot` a partir de `(parent, len, rot)`. Se ele mexer numa única
//! posição, uma de três coisas é falsa: ou a árvore que o L-System emite não é uma árvore
//! bem-formada, ou o `len` que ele declara não é a distância que ele desenhou, ou as duas
//! contas de passo divergiram. ⭐ **Ao BIT** e não por tolerância — as duas fazem a mesma
//! sequência de operações sobre os mesmos `f32`, e uma tolerância esconderia exactamente a
//! divergência que este gate existe para apanhar.
//!
//! ⚠️ E o CONTROLO é uma nuvem de pontos: sobre ela o `rig.fk` também é a identidade (é a
//! regra de identidade dele), então um gate só com o lado positivo ficaria verde com um
//! L-System que emitisse pontos soltos — que é precisamente o desenho que esta wave recusou.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::value::CookValue;

fn registry() -> ph2d_node_registry::NodeRegistry {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("o stream tem de trazer P"),
    }
}

fn stream_of(out: &[CookValue]) -> Stream {
    match &out[0] {
        CookValue::Instances(s) => s.clone(),
        other => panic!("esperava um stream de instancias, veio {other:?}"),
    }
}

/// Cozinha `source.lsystem` sozinho e depois com um `rig.fk` a jusante, com a gramática dada.
fn both_sides(axiom: &str, rules: &str, generations: f32) -> (Stream, Stream) {
    let reg = registry();
    let mut g = Graph::new();
    let l = g.add_node("source.lsystem");
    g.set_text_param(l, ph2d_node_source_lsystem::AXIOM_PARAM, axiom);
    g.set_text_param(l, ph2d_node_source_lsystem::RULES_PARAM, rules);
    g.set_param(l, ph2d_node_source_lsystem::param::GENERATIONS, generations);
    let fk = g.add_node("rig.fk");
    g.connect(Edge {
        from: (l, 0),
        to: (fk, 0),
        delayed: false,
    })
    .expect("o L-System sai num stream que o rig.fk aceita");

    let mut cook = Cook::new();
    let raw = stream_of(cook.cook(&g, &reg, l, 0.0).expect("coze"));
    let resolved = stream_of(cook.cook(&g, &reg, fk, 0.0).expect("coze"));
    (raw, resolved)
}

/// ⭐⭐ **`source.lsystem → rig.fk` não move um bit.**
///
/// Sobre quatro gramáticas que exercitam tudo o que pode partir a invariante: ramos
/// aninhados, um SALTO (`f`, que faz nascer raiz nova), espessura, passo variável, e uma
/// planta paramétrica de verdade.
#[test]
fn the_fk_pass_does_not_move_a_single_lsystem_element() {
    for (axiom, rules, g) in [
        ("F", "F -> F[+F]F[-F]F", 3.0),
        ("F", "F -> F[+F[-F]F]F", 3.0),
        ("F", "F -> F f F[+F]", 3.0),
        (
            ph2d_node_source_lsystem::DEFAULT_AXIOM,
            ph2d_node_source_lsystem::DEFAULT_RULES,
            5.0,
        ),
    ] {
        let (raw, resolved) = both_sides(axiom, rules, g);
        assert!(raw.count() > 4, "a fixtura {rules} tem de ter elementos");
        assert_eq!(raw.count(), resolved.count());
        let (a, b) = (positions(&raw), positions(&resolved));
        for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(
                (p[0].to_bits(), p[1].to_bits()),
                (q[0].to_bits(), q[1].to_bits()),
                "{rules}: o rig.fk mexeu no elemento {i}: {p:?} -> {q:?}"
            );
        }
    }
}

/// **E o `wrot` também sobrevive** — a metade que o `P` sozinho não cobre.
///
/// O `rig.fk` reescreve as DUAS colunas derivadas. Um L-System que acertasse nas posições e
/// errasse nos ângulos de mundo passaria o gate acima e entregaria um esqueleto cujas juntas
/// apontam para outro lado — invisível até alguém lhe pendurar uma sprite.
#[test]
fn the_world_angles_survive_the_fk_pass_too() {
    let (raw, resolved) = both_sides("F", "F -> F[+F]F[-F]F", 3.0);
    let w = |s: &Stream| match s.get("wrot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("wrot"),
    };
    let (a, b) = (w(&raw), w(&resolved));
    for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (p - q).abs() < 1e-3,
            "o rig.fk mudou o angulo de mundo da junta {i}: {p} -> {q}"
        );
    }
}

/// ⚠️ **O CONTROLO — a árvore é uma ÁRVORE, não uma corrente nem uma nuvem.**
///
/// Sem isto, um L-System que emitisse `parent[i] = i − 1` (uma corrente) ou `parent = -1`
/// para todos (uma nuvem) passaria os dois gates acima: a identidade do `rig.fk` vale para os
/// dois casos degenerados. O que só uma ÁRVORE produz é **dois elementos pendurados no mesmo
/// pai** — que é o que um ramo é.
#[test]
fn the_emitted_topology_is_a_tree_and_not_a_chain_or_a_cloud() {
    let (raw, _) = both_sides("F", "F -> F[+F]F[-F]F", 3.0);
    let parent = match raw.get("parent") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("o L-System tem de emitir a coluna parent"),
    };
    let roots = parent.iter().filter(|p| **p < 0.0).count();
    assert_eq!(roots, 1, "uma planta sem saltos tem UMA raiz, deu {roots}");

    let mut counts = std::collections::BTreeMap::<i64, usize>::new();
    for p in parent.iter().filter(|p| **p >= 0.0) {
        *counts.entry(*p as i64).or_default() += 1;
    }
    let forks = counts.values().filter(|c| **c > 1).count();
    assert!(
        forks > 4,
        "uma corrente nao tem bifurcacoes e uma nuvem nao tem pais: achei {forks}"
    );
    // E nenhum pai aponta para a frente — a ordem topológica que o `rig.fk` assume.
    for (i, p) in parent.iter().enumerate() {
        assert!(
            *p < i as f32,
            "o elemento {i} pendura no {p}, que ainda nao existe"
        );
    }
}

//! Os gates da cena `=8` — a ramificação morta.
//!
//! ⚠️ **Eles montam o MESMO grafo que a cena monta, e não uma cópia à mão.** A cena vive num
//! `impl crate::App` e precisa de `gfx`, que um teste de unidade não tem; o que se pode partilhar
//! é a FORMA, e uma segunda cópia da forma divergiria no primeiro ajuste. Então o gate constrói a
//! fiação pela mesma receita e prova as duas afirmações que a mensagem faz: **um** buraco marcado,
//! e a cauda livre **não** marcada.

use ph2d_motion_diagnose::{Deficit, diagnose};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// A fiação da cena: `in0`/`in2` ligadas, `in1` VAZIA, `in3` livre.
fn scene() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let sel = g.add_node("value.lfo");
    let sw = g.add_node("value.switch");
    g.connect(Edge {
        from: (sel, 0),
        to: (sw, 0),
        delayed: false,
    })
    .expect("select");
    for port in [1u16, 3] {
        let src = g.add_node("value.pattern");
        g.connect(Edge {
            from: (grid, 0),
            to: (src, 0),
            delayed: false,
        })
        .expect("contagem");
        g.connect(Edge {
            from: (src, 0),
            to: (sw, port),
            delayed: false,
        })
        .expect("entrada");
    }
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");
    for (from, to) in [
        ((grid, 0), (drive, 0)),
        ((sw, 0), (drive, 1)),
        ((drive, 0), (out, 0)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .expect("cadeia");
    }
    (g, sw)
}

/// **A cena marca UM buraco, e é o do meio.** As duas metades são a mensagem que o smoke
/// imprime: se marcasse dois, a cauda livre estaria a ser tratada como defeito; se marcasse
/// zero, o smoke pediria ao Enio para clicar num badge que não existe.
#[test]
fn the_scene_marks_exactly_the_hole_and_not_the_free_tail() {
    let reg = registry();
    let (g, sw) = scene();
    g.validate(&reg).expect("bem-tipado");
    let marks: Vec<Deficit> = diagnose(&g, &reg)
        .into_iter()
        .filter(|d| d.node == sw)
        .map(|d| d.deficit)
        .collect();
    assert_eq!(
        marks,
        vec![Deficit::DeadBranch("in1")],
        "a cena promete UM badge, na `in1`"
    );
}

/// **A cena inteira não tem OUTRO aviso** — um badge a mais e o Enio clica no errado, e a
/// contagem que a mensagem imprime deixa de bater com o que ele vê.
#[test]
fn nothing_else_in_the_scene_is_diagnosed() {
    let reg = registry();
    let (g, _) = scene();
    let all = diagnose(&g, &reg);
    assert_eq!(
        all.len(),
        1,
        "a cena tem de ter exactamente um aviso: {all:?}"
    );
}

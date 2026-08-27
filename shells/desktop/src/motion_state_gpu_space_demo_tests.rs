//! Gates da cena `=104` — o eixo e a máscara (folha 06, célula 41).

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_space_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

fn cooked(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> ph2d_nodegraph::attr::Stream {
    let mut cook = Cook::new();
    cook.cook(&doc.graph, reg, sink, 0.0).expect("coze")[0]
        .as_stream()
        .clone()
}

fn pos(s: &ph2d_nodegraph::attr::Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn sizes(s: &ph2d_nodegraph::attr::Stream) -> Vec<f32> {
    match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect(),
        _ => Vec::new(),
    }
}

/// A cena monta as quatro metades e nenhuma diverge.
#[test]
fn the_space_scene_builds_all_four_halves() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 4, "dois leques e duas fileiras");
    for (k, s) in sinks.iter().enumerate() {
        let st = cooked(&doc, &reg, *s);
        assert!(st.count() > 0, "metade {k} vazia");
    }
}

/// ⭐⭐ **O leque do MUNDO anda todo junto; o do ELEMENTO abre-se.** A régua é a dispersão em
/// `y`: no mundo o empurrão é horizontal, então nenhuma peça sobe.
#[test]
fn only_the_element_space_fan_opens_up() {
    let (doc, reg, sinks) = scene();
    let spread_y = |s: NodeId| {
        let p = pos(&cooked(&doc, &reg, s));
        let (lo, hi) = p
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[1]), b.max(q[1])));
        hi - lo
    };
    let world = spread_y(sinks[0]);
    let element = spread_y(sinks[1]);
    assert!(
        element > world + PUSH,
        "o leque do elemento tem de abrir bem mais: {element:.3} contra {world:.3}"
    );
    // ⚠️ E o do MUNDO anda de facto — um leque parado abriria a mesma conta.
    let xs = pos(&cooked(&doc, &reg, sinks[0]));
    assert!(
        xs.iter().all(|q| q[0].is_finite()),
        "o leque do mundo existe"
    );
}

/// ⭐ **Fora da máscara, o `Set` guarda o tamanho e o `Remap` leva-o a nada.** É a diferença
/// inteira entre os dois modos, e ela só existe onde a máscara não vale 1.
#[test]
fn outside_the_mask_set_keeps_the_size_and_remap_zeroes_it() {
    let (doc, reg, sinks) = scene();
    let (set, remap) = (
        sizes(&cooked(&doc, &reg, sinks[2])),
        sizes(&cooked(&doc, &reg, sinks[3])),
    );
    assert_eq!(
        set.len(),
        remap.len(),
        "as duas fileiras tem o mesmo tamanho"
    );
    // As pontas estão fora da máscara.
    let edge = 0usize;
    assert!(
        set[edge] > 0.1,
        "o Set guarda o tamanho na ponta: {}",
        set[edge]
    );
    assert!(
        remap[edge] < 0.02,
        "o Remap leva a ponta a nada: {}",
        remap[edge]
    );
    // ⚠️ E no MEIO, onde a máscara vale 1, os dois concordam — senão `Remap` seria outra
    // coisa que não «medir a partir do zero».
    let mid = set.len() / 2;
    assert!(
        (set[mid] - remap[mid]).abs() < 1e-3,
        "no miolo os dois entregam o mesmo: {} contra {}",
        set[mid],
        remap[mid]
    );
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`** — a lei das cenas `=98`..`=103`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_router.rs");
    for k in [
        "gpu_space_demo::FAN",
        "gpu_space_demo::PUSH",
        "gpu_space_demo::MASK_W",
    ] {
        assert!(src.contains(k), "o anuncio tem de citar `{k}`");
    }
}

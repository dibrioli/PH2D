//! Gates da cena `=103` — o relógio da simulação (folha 13, célula 60).
//!
//! ⚠️ Medem o que a cena DESENHA ao longo do TEMPO. As três metades são idênticas no instante
//! zero por construção (a mesma fileira, à mesma altura), então um gate de um cozimento só não
//! distinguiria nada — é o relógio que as separa.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Quantos quadros correr, a 60 fps — o bastante para a metade do `Loop` dar duas voltas.
const TICKS: usize = 400;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_lifecycle_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// `(contagem, y medio)` de cada metade, tique a tique.
fn run(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<(usize, f32)>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::with_capacity(TICKS); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            let st = v[0].as_stream();
            out[s].push(match st.get("P") {
                Some(Column::Vec2(p)) if !p.is_empty() => (
                    p.len(),
                    p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32,
                ),
                _ => (0, 0.0),
            });
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    out
}

/// A cena monta as três metades e nenhuma diverge.
#[test]
fn the_lifecycle_scene_builds_all_three_halves() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 3, "Forever, Once e Loop");
    for (k, rows) in run(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(rows.len(), TICKS, "metade {k}");
        assert!(
            rows.iter().all(|(_, y)| y.is_finite()),
            "metade {k} divergiu"
        );
    }
}

/// ⭐ **A do MEIO fica parada no ar até ao `Start`** — e a da esquerda, não. O controle é o que
/// mede o item: sem ele, «não caiu ainda» seria indistinguível de «não existe».
#[test]
fn only_the_delayed_half_waits_before_it_falls() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    // Meio segundo é antes do `START = 1,0`.
    let k = 30usize;
    assert_eq!(r[1][k].0, 0, "a do meio ainda nao existe no tique {k}");
    assert_eq!(r[0][k].0, COLS as usize, "a da esquerda ja' existe");
    assert!(r[0][k].1 < DROP_Y, "e ja' caiu: y = {}", r[0][k].1);
    // Depois do `START` ela aparece INTEIRA e no alto.
    let k = 65usize;
    assert_eq!(r[1][k].0, COLS as usize, "no start ela aparece inteira");
    assert!(
        r[1][k].1 > r[0][k].1,
        "e ela esta' ACIMA da que ja' caia: {} contra {}",
        r[1][k].1,
        r[0][k].1
    );
}

/// ⭐⭐ **A da DIREITA volta ao alto, e some antes de voltar.** As duas metades importam: um
/// ciclo que reaparece sem ter sumido não é um ciclo, é um salto.
#[test]
fn the_looping_half_disappears_and_then_starts_over() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    let loop_half = &r[2];
    // Achar o primeiro tique vazio, e o primeiro tique povoado DEPOIS dele.
    let gone = loop_half
        .iter()
        .position(|(n, _)| *n == 0)
        .expect("a metade do Loop tem de SUMIR");
    let back = gone
        + loop_half[gone..]
            .iter()
            .position(|(n, _)| *n > 0)
            .expect("e tem de VOLTAR");
    assert_eq!(loop_half[back].0, COLS as usize, "volta inteira");
    assert!(
        loop_half[back].1 > loop_half[gone - 1].1 + 1.0,
        "volta bem ACIMA de onde estava: {} contra {}",
        loop_half[back].1,
        loop_half[gone - 1].1
    );
    // ⚠️ O CONTROLE: a da ESQUERDA nunca some nem volta — senão isto mediria a gravidade.
    assert!(
        r[0].iter().all(|(n, _)| *n == COLS as usize),
        "a metade Forever nunca some"
    );
}

/// **A do meio ACABA e não volta** — a diferença entre `Once` e `Loop`, no que a cena desenha.
#[test]
fn the_once_half_ends_for_good() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    let ended = (60.0 * f64::from(START + DURATION)) as usize + 4;
    assert!(
        r[1][..ended - 8].iter().any(|(n, _)| *n > 0),
        "ela existiu antes de acabar"
    );
    for (k, (n, _)) in r[1].iter().enumerate().skip(ended) {
        assert_eq!(*n, 0, "tique {k} e' depois do fim");
    }
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`** — a lei das cenas `=98`..`=102`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_announce.rs");
    for k in [
        "gpu_lifecycle_demo::COLS",
        "gpu_lifecycle_demo::START",
        "gpu_lifecycle_demo::DURATION",
        "gpu_lifecycle_demo::REST",
    ] {
        assert!(src.contains(k), "o anuncio tem de citar `{k}`");
    }
}

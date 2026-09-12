//! Gates da cena `=101` — um obstáculo com quinas (folha 13).
//!
//! ⚠️ Medem o que a cena DESENHA ao longo do TEMPO: as duas metades só divergem depois de a
//! chuva CHEGAR ao obstáculo, e no instante zero elas são a mesma fila no ar.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Quantos quadros correr, a 60 fps.
const TICKS: usize = 300;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_box_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As posições finais de cada metade.
fn settled(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            if k == TICKS - 1 {
                out[s] = match v[0].as_stream().get("P") {
                    Some(Column::Vec2(p)) => p.clone(),
                    _ => Vec::new(),
                };
            }
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    out
}

/// A cena monta as duas metades, e as duas cospem as `COLS` peças, todas finitas.
#[test]
fn the_box_scene_builds_both_halves() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 2, "duas metades");
    for (k, p) in settled(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(p.len(), COLS as usize, "metade {k}: a fila inteira");
        assert!(
            p.iter().all(|q| q.iter().all(|x| x.is_finite())),
            "metade {k}: alguma peca divergiu"
        );
    }
}

/// ⭐⭐ **AS DUAS METADES ACABAM DIFERENTES.** Um par que saísse igual seria um obstáculo com
/// nome novo — e é a primeira coisa que o Enio olha.
#[test]
fn the_disc_and_the_box_do_not_settle_the_same_way() {
    let (doc, reg, sinks) = scene();
    let s = settled(&doc, &reg, &sinks);
    // Comparo a FORMA de cada monte (a posição relativa ao centro da metade), porque as duas
    // metades vivem em `x` diferentes por construção.
    let shape_of = |p: &[[f32; 2]], cx: f32| -> Vec<[f32; 2]> {
        p.iter().map(|q| [q[0] - cx, q[1]]).collect()
    };
    let a = shape_of(&s[0], -2.6);
    let b = shape_of(&s[1], 2.6);
    let differ = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| (x[0] - y[0]).abs() > 0.05 || (x[1] - y[1]).abs() > 0.05)
        .count();
    assert!(
        differ >= COLS as usize / 3,
        "so' {differ} de {} pecas acabaram em sitios diferentes -- as duas formas mal se \
         distinguem",
        COLS as usize
    );
}

/// ⭐ **NENHUMA PEÇA FICA DENTRO DA CAIXA.** É a afirmação que separa um obstáculo de um
/// contentor, e é a que a sonda mostrou que o encadeamento de planos NÃO faz.
#[test]
fn no_piece_ends_up_inside_the_box() {
    let (doc, reg, sinks) = scene();
    let right = &settled(&doc, &reg, &sinks)[1];
    // Para o referencial da caixa (centro `(2,6, OBSTACLE_Y)`, rodada de `TILT`).
    let rad = TILT.to_radians();
    let (cos, sin) = (rad.cos(), rad.sin());
    let (hw, hh) = (DISC_R, 0.275); // BOX_W/2 e BOX_H/2
    for (i, q) in right.iter().enumerate() {
        let (dx, dy) = (q[0] - 2.6, q[1] + 0.4);
        let (lx, ly) = (dx * cos + dy * sin, -dx * sin + dy * cos);
        assert!(
            lx.abs() > hw - 0.05 || ly.abs() > hh - 0.05,
            "peca {i} acabou DENTRO da caixa: local ({lx:.3}, {ly:.3})"
        );
    }
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`** — a mesma lei das cenas `=98`..`=100`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_router.rs");
    assert!(
        src.contains("gpu_box_demo::COLS")
            && src.contains("gpu_box_demo::TILT")
            && src.contains("gpu_box_demo::DISC_R"),
        "a contagem, a inclinacao e o raio saem dos `const`"
    );
}

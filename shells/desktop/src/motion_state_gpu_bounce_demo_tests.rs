//! Gates da cena `=99` — nem toda batida devolve o mesmo (folha 13).
//!
//! ⚠️ **Eles medem o que a cena DESENHA ao longo do TEMPO**, e não o que ela monta: a cena é
//! uma simulação, e a afirmação inteira («cada um volta à sua altura») só existe depois de
//! haver uma batida. Um gate que cozesse o instante zero veria duas fileiras idênticas no ar.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Quantos quadros correr, a 60 fps. `240` = 4 s: tempo para cair, bater e voltar.
const TICKS: usize = 240;
/// A partir de que quadro a altura conta. Antes disto os discos ainda estão na PRIMEIRA
/// queda, e a altura de partida é igual nos dois lados por construção.
const AFTER: usize = 90;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_bounce_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// A altura MÁXIMA que cada disco alcança depois do quadro [`AFTER`] — a régua da quicada.
fn peak_heights(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<f32>> {
    let mut cook = Cook::new();
    let mut peaks: Vec<Vec<f32>> = vec![vec![f32::MIN; COLS as usize]; sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            if k < AFTER {
                continue;
            }
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                for (i, q) in p.iter().enumerate().take(COLS as usize) {
                    peaks[s][i] = peaks[s][i].max(q[1]);
                }
            }
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    peaks
}

/// A cena monta as duas fileiras, e as duas cospem os `COLS` discos.
#[test]
fn the_bounce_scene_builds_both_rows() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 2, "duas fileiras");
    let mut cook = Cook::new();
    for (k, s) in sinks.iter().enumerate() {
        let out = cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze");
        assert_eq!(
            out[0].as_stream().count(),
            COLS as usize,
            "fileira {k}: a fila inteira"
        );
    }
}

/// ⭐⭐ **O PAR.** Em cima todos voltam à mesma altura; em baixo cada um à sua. Se as duas
/// fileiras saíssem iguais, o param seria um rótulo — e é exactamente isso que o Enio vê.
#[test]
fn the_top_row_bounces_as_one_and_the_bottom_row_spreads() {
    let (doc, reg, sinks) = scene();
    let peaks = peak_heights(&doc, &reg, &sinks);
    let spread = |v: &[f32]| -> f32 {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
        hi - lo
    };
    let (top, bottom) = (&peaks[0], &peaks[1]);
    assert!(
        spread(top) < 1e-4,
        "sem acaso as nove alturas tem de ser a MESMA (espalhamento {}): {top:?}",
        spread(top)
    );
    assert!(
        spread(bottom) > 0.05,
        "com acaso elas tem de separar-se (espalhamento {}): {bottom:?}",
        spread(bottom)
    );
}

/// ⚠️ **A lei só TIRA** — nenhum disco da fileira de baixo pode voltar mais alto que os de
/// cima. É a metade que impede a «máquina de fazer energia», e é a que um `±` centrado no
/// valor autorado teria quebrado em metade dos elementos.
#[test]
fn no_disc_in_the_random_row_bounces_higher_than_the_authored_one() {
    let (doc, reg, sinks) = scene();
    let peaks = peak_heights(&doc, &reg, &sinks);
    let ceiling = peaks[0].iter().copied().fold(f32::MIN, f32::max);
    for (i, h) in peaks[1].iter().enumerate() {
        assert!(
            *h <= ceiling + 1e-4,
            "disco {i} voltou a {h} contra o tecto autorado {ceiling}"
        );
    }
}

/// Nenhum disco atravessa o chão — a régua mais barata de que a simulação não explodiu, e a
/// que o anúncio promete ao Enio.
#[test]
fn no_disc_falls_through_the_floor() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, &reg, *sink, t).expect("coze");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                for (i, q) in p.iter().enumerate() {
                    assert!(
                        q[1] >= FLOOR - 0.5 && q[1].is_finite(),
                        "fileira {s}, disco {i}, quadro {k}: y = {} (chao {FLOOR})",
                        q[1]
                    );
                }
            }
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avanca");
    }
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`**, e este gate lê o fonte da narração —
/// a mesma lei da cena `=98`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_router.rs");
    assert!(
        src.contains("gpu_bounce_demo::COLS"),
        "a contagem sai do `const`"
    );
    assert!(
        src.contains("gpu_bounce_demo::FLOOR") && src.contains("gpu_bounce_demo::RESTITUTION"),
        "o chao e a quicada autorada tambem"
    );
}

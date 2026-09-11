//! Gates da cena `=100` — as peças giram, e o giro pode parar (folha 13).
//!
//! ⚠️ Medem o que a cena DESENHA ao longo do TEMPO: a afirmação inteira é sobre um ângulo
//! que cresce, e no instante zero as duas fileiras são idênticas por construção.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Quantos quadros correr, a 60 fps. `240` = 4 s.
const TICKS: usize = 240;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_spin_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// Os ângulos de cada fileira no quadro `at`.
fn angles_at(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId], at: usize) -> Vec<Vec<f32>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..=at {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            if k == at {
                out[s] = match v[0].as_stream().get("rot") {
                    Some(Column::Scalar(c)) => c.clone(),
                    _ => Vec::new(),
                };
            }
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    out
}

/// A cena monta as duas fileiras, e as duas cospem as `COLS` peças **com a coluna do ângulo**.
/// ⚠️ Sem essa segunda metade o gate passaria com o `spin` a nunca ser escrito.
#[test]
fn the_spin_scene_builds_both_rows_and_authors_the_angle() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 2, "duas fileiras");
    let a = angles_at(&doc, &reg, &sinks, 30);
    for (k, row) in a.iter().enumerate() {
        assert_eq!(row.len(), COLS as usize, "fileira {k}: a fila inteira");
    }
}

/// ⭐⭐ **CADA PEÇA GIRA À SUA TAXA.** Uma rampa de giro que saísse chapada daria uma fileira
/// a rodar em bloco — legível, e uma afirmação diferente da que a cena faz.
#[test]
fn every_piece_turns_at_its_own_rate() {
    let (doc, reg, sinks) = scene();
    let top = &angles_at(&doc, &reg, &sinks, 120)[0];
    // Monotónica e estritamente crescente: a rampa vai de `0` a `TOP_SPIN`.
    for w in top.windows(2) {
        assert!(
            w[1] > w[0] + 1e-3,
            "as taxas tem de ser distintas e crescentes: {top:?}"
        );
    }
    assert!(
        top[0].abs() < 1e-4,
        "a primeira peca fica PARADA (a base da rampa e' zero): {top:?}"
    );
}

/// ⭐⭐ **O PAR: o arrasto angular PARA o giro.** Depois de 4 s a fileira de baixo tem de estar
/// muito atrás da de cima — e é isso que o Enio vê como «elas travam».
#[test]
fn the_drag_row_falls_behind_and_settles() {
    let (doc, reg, sinks) = scene();
    let a = angles_at(&doc, &reg, &sinks, TICKS);
    let (top, bottom) = (&a[0], &a[1]);
    let fastest = COLS as usize - 1;
    assert!(
        bottom[fastest] < top[fastest] * 0.6,
        "com arrasto a peca mais rapida girou {} contra os {} sem arrasto",
        bottom[fastest],
        top[fastest]
    );
    // E ela está QUASE parada. ⚠️ **«Parada de vez» não é o que esta lei dá, e a 1.ª versão
    // deste gate pedia isso:** o amortecimento é de primeira ordem (`1 − (1−d)·dt`), então o
    // giro decai geometricamente e **nunca chega a zero** — a 4 s ele está em `0,198°` por
    // quadro, que é `3%` do inicial. *Uma expectativa de «para» sobre uma lei exponencial mede
    // a paciência de quem a escreveu.* A régua honesta é a FRACÇÃO do que sobrou.
    let near_end = angles_at(&doc, &reg, &sinks, TICKS - 1);
    let rate_drag = (bottom[fastest] - near_end[1][fastest]).abs();
    let rate_free = (top[fastest] - near_end[0][fastest]).abs();
    assert!(
        rate_drag < rate_free * 0.08,
        "no fim a peca com arrasto ainda anda {rate_drag} por quadro contra {rate_free} sem \
         arrasto -- ela mal travou"
    );
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`** — a mesma lei das cenas `=98`/`=99`.
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_router.rs");
    assert!(
        src.contains("gpu_spin_demo::COLS")
            && src.contains("gpu_spin_demo::TOP_SPIN")
            && src.contains("gpu_spin_demo::DRAG"),
        "a contagem, a taxa do topo e o arrasto saem dos `const`"
    );
}

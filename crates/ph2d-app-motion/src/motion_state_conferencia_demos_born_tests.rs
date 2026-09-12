//! Gates da cena `=93` — **onde as coisas nascem** (doc 89, folha 01).
//!
//! ⚠️ **As quatro primeiras fileiras NÃO simulam**, então um `advance_tick` é
//! desnecessário para elas; a última (o emissor) precisa de um playhead, e é o único
//! sítio onde este harness pede um instante que não seja zero.

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
    let sinks = build_born_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As poses de cada sink pedido, num instante.
fn poses(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId], t: f64) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    sinks
        .iter()
        .map(|s| {
            let out = cook.cook(&doc.graph, reg, *s, t).expect("coze");
            match out[0].as_stream().get("P") {
                Some(Column::Vec2(p)) => p.clone(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// A maior distância entre dois pontos — o quanto a figura ocupa.
fn extent(p: &[[f32; 2]]) -> f32 {
    let mut d = 0.0_f32;
    for a in p {
        for b in p {
            d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    d
}

/// **A CENA MONTA AS DEZ BANDAS**, e as dez cospem.
#[test]
fn the_born_scene_builds_all_ten_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 10, "cinco pares");
    assert_eq!(band_labels().count(), 10, "um rotulo por banda");
    let p = poses(&doc, &reg, &sinks, 1.2);
    for (k, band) in p.iter().enumerate() {
        assert!(!band.is_empty(), "banda {k} vazia");
        for q in band {
            assert!(q[0].is_finite() && q[1].is_finite(), "banda {k} explodiu");
        }
    }
}

/// ⭐⭐ **O PAR 1 e o PAR 2 são a mesma pergunta com respostas OPOSTAS** — e é a razão de
/// as duas fileiras existirem lado a lado.
///
/// A grade **perde** pontos ao virar círculo (um reticulado não se dobra); o
/// espalhamento **guarda todos** ao virar anel (um amostrador só muda onde o dardo cai).
#[test]
fn the_lattice_loses_points_where_the_sampler_keeps_them_all() {
    let (doc, reg, sinks) = scene();
    let p = poses(&doc, &reg, &sinks, 0.0);
    let (grid_rect, grid_circle) = (p[0].len(), p[1].len());
    let (sc_rect, sc_ring) = (p[2].len(), p[3].len());
    assert!(
        grid_circle < grid_rect,
        "a grade em circulo tinha de PERDER pontos: {grid_rect} -> {grid_circle}"
    );
    assert!(
        grid_circle > grid_rect / 2,
        "e nao de os apagar: {grid_rect} -> {grid_circle}"
    );
    assert_eq!(
        sc_ring, sc_rect,
        "o espalhamento em anel tinha de guardar TODOS: {sc_rect} -> {sc_ring}"
    );
    // E o anel de facto tem buraco — senão a contagem intacta não provava nada.
    let centre = p[3]
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
        .map(|v| v / sc_ring as f32);
    let closest = p[3]
        .iter()
        .map(|q| (q[0] - centre[0]).hypot(q[1] - centre[1]))
        .fold(f32::MAX, f32::min);
    assert!(
        closest > extent(&p[3]) * 0.15,
        "o anel nao tem buraco: o mais perto do centro esta' a {closest:.3}"
    );
}

/// ⭐ **O PAR 3: a borda fica mais RALA, e não esburacada.** A régua é o vão mediano por
/// banda — a contagem não distingue as duas leituras.
#[test]
fn the_graded_poisson_thins_the_edge_instead_of_holing_it() {
    let (doc, reg, sinks) = scene();
    let p = poses(&doc, &reg, &sinks, 0.0);
    let gaps = |pts: &[[f32; 2]], outer: bool| -> f32 {
        let c = pts
            .iter()
            .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]])
            .map(|v| v / pts.len() as f32);
        let r = extent(pts) * 0.5;
        let mut v: Vec<f32> = pts
            .iter()
            .filter(|q| {
                let d = (q[0] - c[0]).hypot(q[1] - c[1]) / r;
                if outer { d > 0.7 } else { d < 0.35 }
            })
            .map(|q| {
                pts.iter()
                    .filter(|o| !std::ptr::eq(*o, q))
                    .map(|o| (q[0] - o[0]).hypot(q[1] - o[1]))
                    .fold(f32::MAX, f32::min)
            })
            .collect();
        v.sort_by(f32::total_cmp);
        if v.is_empty() { 0.0 } else { v[v.len() / 2] }
    };
    let (flat, graded) = (&p[4], &p[5]);
    assert!(flat.len() > 60 && graded.len() > 30, "ha' amostra");
    let flat_ratio = gaps(flat, true) / gaps(flat, false);
    let graded_ratio = gaps(graded, true) / gaps(graded, false);
    assert!(
        (flat_ratio - 1.0).abs() < 0.25,
        "CONTROLE: o uniforme tinha de dar bandas iguais ({flat_ratio:.3})"
    );
    assert!(
        graded_ratio > flat_ratio * 1.4,
        "a borda graduada tinha de ficar mais rala: {flat_ratio:.3} -> {graded_ratio:.3}"
    );
}

/// ⭐ **O PAR 4: a métrica muda o arranjo** — a mesma semente, outro CVT.
#[test]
fn the_metric_moves_the_voronoi_cells() {
    let (doc, reg, sinks) = scene();
    let p = poses(&doc, &reg, &sinks, 0.0);
    let (e, c) = (&p[6], &p[7]);
    assert_eq!(e.len(), c.len(), "a mesma contagem");
    let moved = e
        .iter()
        .zip(c)
        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
        .fold(0.0_f32, f32::max);
    assert!(moved > 0.02, "as duas reguas coincidiram ({moved:.5})");
}

/// ⭐ **O PAR 5: a vida variada desmancha a BORDA do penacho.**
///
/// ⚠️ A régua não é a contagem (que também cai): é o quanto a partícula MAIS VELHA viva
/// se afastou. Com vida única todas morrem na mesma frente; com variância a frente
/// esfarrapa-se, e o alcance máximo desce.
#[test]
fn the_varied_life_frays_the_plumes_edge() {
    let (doc, reg, sinks) = scene();
    let p = poses(&doc, &reg, &sinks, 2.4);
    let (uniform, varied) = (&p[8], &p[9]);
    assert!(uniform.len() > 40, "o penacho enche-se: {}", uniform.len());
    assert!(
        varied.len() < uniform.len(),
        "a variancia tinha de ralar: {} -> {}",
        uniform.len(),
        varied.len()
    );
    assert!(
        varied.len() > uniform.len() / 4,
        "e nao de apagar: {} -> {}",
        uniform.len(),
        varied.len()
    );
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 10, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}

//! Os gates da cena `=61` — o sub-passo do integrador.
//!
//! ⚠️ **Estas fixtures marcham pelo caminho do PUMP** (`substep_islands` + `Cook::substep`), e não
//! é conforto: sem o bracket a cena corre a UM sub-passo qualquer que seja o slider, e todo gate
//! sobre o número mediria uma cena que o app nunca mostra. O `advance_tick` é a outra metade —
//! sem ele a aresta `pre` nunca carrega estado e o corpo fica no repouso, verde por vácuo.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::{Cook, graph_substeps, substep_islands};

const DT: f64 = 1.0 / 60.0;
/// Três segundos — o mesmo horizonte da sonda que mediu a tabela do `STRENGTH`.
const TICKS: u64 = 180;
/// A folga do anel. ⚠️ **Ela é a precisão do `f32` do distribuidor radial, não slack:** o ponto
/// mais fora mede `4,0010` contra `4,0000`, e a fonte é o seno por ciclos do
/// `motion.distribute_radial`. Uma barra mais apertada reprovaria a cena por aritmética correta.
const RING_TOL: f32 = 5e-3;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Monta a cena e troca o `substeps` do integrador por `sub` — a única coisa que o artista mexe.
fn scene(sub: Option<f32>) -> (MotionDoc, NodeRegistry, NodeId) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_substep_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 1, "uma banda, de propósito");
    if let Some(s) = sub {
        let want = ph2d_nodegraph::node::NodeTypeId::of(INTEGRATOR);
        let integ = doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_id() == want)
            .expect("a cena tem um integrador")
            .id;
        doc.graph.set_param(integ, "substeps", s);
    }
    (doc, reg, sinks[0])
}

/// Marcha a cena pelo caminho do pump e devolve o maior RAIO que o corpo alcançou.
///
/// ⚠️ O corpo é o ÚLTIMO ponto do fluxo: o `motion.combine` concatena `in0` (o anel) e depois
/// `in1` (a bolinha). O gate `the_ring_is_the_oracle_at_the_start_radius` é quem defende essa
/// leitura — se a ordem virar, ele fica vermelho antes deste.
fn worst_radius(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> f32 {
    let mut cook = Cook::new();
    let mut worst = 0.0f32;
    for k in 0..TICKS {
        let t = (k + 1) as f64 * DT;
        if let Some(frame_start) = cook.prev_playhead() {
            for island in substep_islands(&doc.graph, reg) {
                cook.substep(
                    &doc.graph,
                    reg,
                    island.root,
                    frame_start,
                    t,
                    island.substeps,
                )
                .expect("substep");
            }
        }
        let v = cook.cook(&doc.graph, reg, sink, t).expect("a cena coze");
        if let Some(Column::Vec2(p)) = v[0].as_stream().get("P")
            && let Some(q) = p.last()
        {
            worst = worst.max((q[0] * q[0] + q[1] * q[1]).sqrt());
        }
        cook.advance_tick(&doc.graph, reg, t).expect("tique");
    }
    worst
}

/// **O ANEL É O ORÁCULO, e ele tem de estar onde a prosa diz.** Um alvo desenhado noutro raio
/// tornaria toda leitura desta cena uma comparação com nada.
///
/// ⚠️ Ele também fixa a ORDEM do `motion.combine` — os primeiros pontos são o anel e o último é o
/// corpo, que é a leitura de que `worst_radius` depende.
#[test]
fn the_ring_is_the_oracle_at_the_start_radius() {
    let (doc, reg, sink) = scene(None);
    let mut cook = Cook::new();
    let v = cook.cook(&doc.graph, &reg, sink, 0.0).expect("coza");
    let Some(Column::Vec2(p)) = v[0].as_stream().get("P") else {
        panic!("a cena emite posições")
    };
    let (ring, _, _) = numbers();
    assert_eq!(
        p.len(),
        RING_POINTS as usize + 1,
        "o anel mais a bolinha: {} pontos",
        p.len()
    );
    for (i, q) in p.iter().take(RING_POINTS as usize).enumerate() {
        let r = (q[0] * q[0] + q[1] * q[1]).sqrt();
        assert!(
            (r - ring).abs() < RING_TOL,
            "o ponto {i} do anel está a {r:.4}, e o alvo é {ring:.4}"
        );
    }
    // E a bolinha NASCE em cima do anel — é isso que faz "voltar ao anel" ser a pergunta.
    let start = p.last().expect("a bolinha");
    let r = (start[0] * start[0] + start[1] * start[1]).sqrt();
    assert!(
        (r - ring).abs() < RING_TOL,
        "a bolinha nasce a {r:.4} e o anel está a {ring:.4}"
    );
}

/// **A CENA AUTORA O RITMO QUE A PROSA CITA.** Um `substeps` que a cena escreve e o achador não vê
/// seria um número no painel sem consequência — o defeito exacto que a folha 17 existia para
/// fechar.
#[test]
fn the_scene_declares_the_rhythm_its_prose_claims() {
    let (doc, reg, _) = scene(None);
    let (_, subs, _) = numbers();
    assert_eq!(
        graph_substeps(&doc.graph, &reg),
        subs as u32,
        "o ritmo do grafo tem de ser o que a cena escreveu"
    );
    assert_eq!(
        substep_islands(&doc.graph, &reg).len(),
        1,
        "uma simulação, uma ilha"
    );
}

/// **A LEITURA DA CENA: a UM sub-passo o corpo foge do anel; no topo da faixa ele volta perto.**
///
/// As barras são folgadas de propósito — o que a cena promete é a SEPARAÇÃO, e prender o gate aos
/// dois números medidos ao centésimo faria dele um espelho da build em vez de uma afirmação sobre
/// o produto.
#[test]
fn one_substep_leaves_the_ring_and_the_top_of_the_range_comes_back() {
    let (ring, subs, _) = numbers();
    let (doc_top, reg, sink_top) = scene(None);
    let top = worst_radius(&doc_top, &reg, sink_top);
    let (doc_one, reg_one, sink_one) = scene(Some(1.0));
    let one = worst_radius(&doc_one, &reg_one, sink_one);

    assert!(
        one > ring * 2.5,
        "a UM sub-passo o corpo tem de sair MUITO do anel ({ring:.1}); foi a {one:.3}"
    );
    assert!(
        top < ring * 1.4,
        "com {subs:.0} sub-passos ele tem de voltar perto do anel ({ring:.1}); foi a {top:.3}"
    );
    // O CONTROLE de que a fixture contém o fenômeno: os dois números são a MESMA cena, e a única
    // diferença é o slider.
    assert!(
        one > top * 2.0,
        "o slider tem de separar as duas leituras: 1 -> {one:.3}, {subs:.0} -> {top:.3}"
    );
}

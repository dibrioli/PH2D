//! Gates da cena `=86` — **a fronteira curva** (doc 89, folha 04).
//!
//! ⚠️ **O oráculo é a RECTIDÃO de uma fileira interior, e não «a figura mudou».** As
//! duas metades da linha 1 têm os mesmos quatro cantos: uma régua de excursão, de
//! contagem ou de caixa envolvente daria verde sobre as duas e não diria nada. O que
//! as separa é o que uma projectividade garante e um bilinear não — rectas.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn cell(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Stream {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    match out.first() {
        Some(CookValue::Instances(s)) => s.clone(),
        _ => panic!("a celula emite instancias"),
    }
}

fn points(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(p)) => p.clone(),
        _ => panic!("P"),
    }
}

fn row_of(case: Case) -> usize {
    ROWS_TABLE
        .iter()
        .position(|r| r.case == case)
        .expect("a linha existe")
}

/// O quanto uma fileira do bloco se afasta da recta que une as duas pontas dela —
/// zero numa projectividade, positivo num bilinear.
fn bow(p: &[[f32; 2]], row: usize) -> f32 {
    let n = SIDE as usize;
    let (a, c) = (p[row * n], p[row * n + n - 1]);
    let d = [c[0] - a[0], c[1] - a[1]];
    let len = d[0].hypot(d[1]).max(1e-6);
    (1..n - 1)
        .map(|i| {
            let m = p[row * n + i];
            ((m[0] - a[0]) * d[1] - (m[1] - a[1]) * d[0]).abs() / len
        })
        .fold(0.0, f32::max)
}

/// **A CENA CONSTRÓI TODAS AS CÉLULAS.**
#[test]
fn the_bezier_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_bezier_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, side) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(side >= 5.0, "o bloco precisa de MIOLO");
    for sink in &sinks {
        assert!(cell(&doc, &reg, *sink).count() > 0, "toda celula desenha");
    }
}

/// **OS MESMOS QUATRO CANTOS, E SÓ UM DOS DOIS ENTORTA O MIOLO.**
///
/// ⚠️ **É o gate que justifica a cena inteira.** As duas metades recebem o MESMO offset
/// de canto; o gate mede (a) que as quatro pontas de facto coincidem — senão estaríamos
/// a comparar dois ajustes diferentes —, e (b) que as fileiras interiores ficam rectas
/// à esquerda e arqueadas à direita. Sem (a) o gate passaria numa cena mal montada; sem
/// (b) ele passaria com os dois nós a fazerem a mesma coisa.
#[test]
fn the_same_corners_give_a_straight_interior_on_one_side_and_a_bowed_one_on_the_other() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_bezier_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::SameCorners);
    let pin = points(&cell(&doc, &reg, sinks[k * 2]));
    let bez = points(&cell(&doc, &reg, sinks[k * 2 + 1]));
    let n = SIDE as usize;
    assert_eq!(pin.len(), bez.len(), "as duas metades têm as mesmas peças");

    // (a) As QUATRO PONTAS coincidem — a cena compara o mesmo quadrilátero.
    //
    // ⚠️ **A primeira versão comparava relativamente ao CENTROIDE de cada metade, e
    // falhou por 0,16 sobre código correcto:** os dois mapas distribuem o INTERIOR de
    // forma diferente (é a feature), então os centroides deles não coincidem — usar um
    // como referência mede a diferença que a cena existe para mostrar. A referência
    // certa é o sítio onde o `motion.transform` PÔS cada metade, que é um número que a
    // cena autora e não uma estatística do resultado.
    let corners = [0, n - 1, (n - 1) * n, n * n - 1];
    let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
    for c in corners {
        let a = [pin[c][0] + COL_X, pin[c][1] - cy];
        let b = [bez[c][0] - COL_X, bez[c][1] - cy];
        assert!(
            (a[0] - b[0]).abs() < 0.02 && (a[1] - b[1]).abs() < 0.02,
            "o canto {c} tem de coincidir nos dois: {a:?} vs {b:?}"
        );
    }

    // (b) A fileira do MEIO: recta à esquerda, arqueada à direita.
    let mid_row = n / 2;
    let (bp, bb) = (bow(&pin, mid_row), bow(&bez, mid_row));
    assert!(
        bp < 0.01,
        "a homografia mantém a fileira RECTA (desvio {bp:.4})"
    );
    assert!(
        bb > 0.05,
        "e o Coons ARQUEIA-a (desvio {bb:.4}) — se os dois fossem rectos, esta cena não \
         teria o que mostrar e o nó novo seria o irmão com mais knobs"
    );
}

/// **A ARESTA CURVA SÓ EXISTE DE UM LADO.**
///
/// ⚠️ A metade da esquerda está no NEUTRO, e é isso que a linha mostra: o Corner Pin
/// não tem param de aresta nenhum. O gate mede a fileira do TOPO (a que a barriga
/// empurra) e exige que a esquerda continue a ser a grelha de origem.
#[test]
fn only_the_bezier_half_bends_the_top_edge() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_bezier_demo_document(&mut doc, &reg).expect("constroi");
    let k = row_of(Case::CurvedEdge);
    let n = SIDE as usize;
    let pin = points(&cell(&doc, &reg, sinks[k * 2]));
    let bez = points(&cell(&doc, &reg, sinks[k * 2 + 1]));
    // A fileira do topo é a de índice mais alto (o `motion.grid` cresce em y).
    let top = n - 1;
    let (bp, bb) = (bow(&pin, top), bow(&bez, top));
    assert!(
        bp < 0.01,
        "a metade de sempre deixa a borda RECTA ({bp:.4})"
    );
    assert!(bb > 0.3, "e a curada empurra-a para fora ({bb:.4})",);
    // E o CONTROLE: a fileira de BAIXO não se mexe — a barriga é da aresta de cima, e
    // um patch que escalasse tudo passaria na metade acima.
    assert!(
        bow(&bez, 0) < 0.02,
        "a borda oposta fica recta: {:.4}",
        bow(&bez, 0)
    );
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_bezier_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let p = points(&cell(&doc, &reg, *sink));
        let row = k / 2;
        let half = k % 2;
        let cx = if half == 0 { -COL_X } else { COL_X };
        let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in &p {
            mx = mx.max((q[0] - cx).abs());
            my = my.max((q[1] - cy).abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}

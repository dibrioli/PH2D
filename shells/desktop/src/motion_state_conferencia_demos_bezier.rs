//! **A FRONTEIRA CURVA** (`PH2D_GPU_COOK_DEMO=86`) — a cena do `motion.bezier_warp`
//! (doc 89, folha 04, o P1).
//!
//! ## Duas linhas que se julgam PARADAS, e a primeira é a que decide
//!
//! - **MESMOS CANTOS** — os DOIS nós com os **mesmos quatro cantos**. À esquerda o
//!   `motion.four_point_warp` (a homografia): as fileiras interiores da grelha ficam
//!   **RECTAS**, porque uma projectividade preserva rectas por definição. À direita o
//!   `motion.bezier_warp`: as mesmas quatro pontas, e as fileiras **ARQUEIAM** — é o
//!   mapa bilinear. ⚠️ *É esta linha que mostra por que são dois nós e não um param.*
//! - **ARESTA CURVA** — o que o Corner Pin **não sabe fazer de todo**: à esquerda ele
//!   com os cantos no lugar (nada acontece — uma aresta não é um canto), à direita a
//!   borda de cima empurrada pelas duas tangentes.
//!
//! ⚠️ **A fixture é um BLOCO e não uma fileira**, e isso é a cena inteira: a diferença
//! entre os dois mapas vive no INTERIOR. Uma fileira só tem borda, e as duas metades
//! sairiam iguais — o gate `no_cell_climbs_into_its_neighbour` continuaria verde sobre
//! uma cena que não prova nada.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Lado do bloco — ímpar, para haver uma fileira EXACTAMENTE no meio (é a que mais
/// arqueia, e a que o olho segue).
pub(crate) const SIDE: f32 = 9.0;
/// O tamanho de uma peça.
const DOT: f32 = 0.17;
/// A distância vertical entre linhas.
///
/// ⚠️ **Ela é DERIVADA do alcance da célula, não escolhida:** o bloco tem meia-altura
/// `R` e o canto é puxado por `CORNER_DY`, então uma peça pode chegar a `R + CORNER_DY`
/// do centro — e meia linha tem de passar disso. A primeira versão punha `3,4` e o gate
/// de layout apanhou a célula 0 a subir `2,05` contra uma meia-linha de `1,7`.
pub(crate) const ROW_GAP: f32 = 4.8;
/// A que distância do centro cada coluna vive.
pub(crate) const COL_X: f32 = 3.0;
/// A meia-largura do bloco.
const R: f32 = 1.15;

/// **O canto puxado, o MESMO nos dois nós da linha 1.**
///
/// ⚠️ Só um canto se move, e é de propósito: com dois cantos opostos puxados o
/// bilinear e o projectivo aproximam-se, e a linha deixaria de decidir nada. Um canto
/// só é o caso em que os dois mapas mais divergem.
const CORNER_DX: f32 = 1.5;
const CORNER_DY: f32 = 0.9;
/// A barriga da aresta de cima, na linha 2.
const BULGE: f32 = 1.1;

/// Qual das duas coisas esta linha encena.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Case {
    /// Os mesmos cantos nos dois nós — a divergência do interior.
    SameCorners,
    /// Uma aresta curva — o que o Corner Pin não exprime.
    CurvedEdge,
}

pub(crate) struct Row {
    pub(crate) label: &'static str,
    pub(crate) left: &'static str,
    pub(crate) right: &'static str,
    pub(crate) case: Case,
}

pub(crate) static ROWS_TABLE: &[Row] = &[
    Row {
        label: "MESMOS CANTOS — as mesmas 4 pontas; à direita o MIOLO arqueia",
        left: "1 CANTOS · antes: miolo reto",
        right: "1 CANTOS · agora: miolo curvo",
        case: Case::SameCorners,
    },
    Row {
        label: "ARESTA — o de antes só sabe mover CANTOS; o novo entorta a BORDA",
        left: "2 ARESTA · antes: nao da'",
        right: "2 ARESTA · agora: borda curva",
        case: Case::CurvedEdge,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32) {
    (ROWS_TABLE.len(), SIDE)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let mut out = Vec::with_capacity(ROWS_TABLE.len() * 2);
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y =
            (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP + ROW_GAP * 0.38;
        out.push(crate::motion_demo_legend::Caption::new(
            [-COL_X, y],
            row.left,
        ));
        out.push(crate::motion_demo_legend::Caption::new(
            [COL_X, y],
            row.right,
        ));
    }
    out
}

fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    from: NodeId,
    fp: u16,
    to: NodeId,
    tp: u16,
) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O documento da cena `=86` — uma sink por célula (duas por linha).
pub(crate) fn build_bezier_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
        for (half, cured) in [(0usize, false), (1, true)] {
            let lane = 100.0 + (k * 2 + half) as f32 * 320.0;
            let cell = build_cell(g, row.case, cured, lane)?;
            let place = g.add_node("motion.transform");
            g.set_param(place, "offset_x", if half == 0 { -COL_X } else { COL_X });
            g.set_param(place, "offset_y", y);
            let out = g.add_node("motion.output");
            g.set_pos(place, Pos { x: 1500.0, y: lane });
            g.set_pos(out, Pos { x: 1700.0, y: lane });
            wire(g, cell, 0, place, 0)?;
            wire(g, place, 0, out, 0)?;
            sinks.push(out);
        }
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

/// Um BLOCO `SIDE × SIDE` — a diferença entre os dois mapas vive no INTERIOR.
fn block(g: &mut ph2d_nodegraph::graph::Graph, lane: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    let gap = 2.0 * R / (SIDE - 1.0);
    g.set_param(grid, "gap_x", gap);
    g.set_param(grid, "gap_y", gap);
    let size = g.add_node("motion.scale");
    g.set_param(size, "amount", DOT);
    wire(g, grid, 0, size, 0)?;
    g.set_pos(grid, Pos { x: 80.0, y: lane });
    g.set_pos(size, Pos { x: 260.0, y: lane });
    Some(size)
}

fn build_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    case: Case,
    cured: bool,
    lane: f32,
) -> Option<NodeId> {
    let src = block(g, lane)?;
    // ⚠️ Os DOIS nós têm os mesmos nomes de param de canto (`tl_dx`…`bl_dy`), e é isso
    // que faz esta comparação ser justa: o mesmo número, escrito da mesma forma, nos
    // dois. Se os nomes divergissem, a cena estaria a comparar dois ajustes.
    let node = g.add_node(if cured {
        "motion.bezier_warp"
    } else {
        "motion.four_point_warp"
    });
    match case {
        Case::SameCorners => {
            g.set_param(node, "tr_dx", CORNER_DX);
            g.set_param(node, "tr_dy", CORNER_DY);
        }
        Case::CurvedEdge => {
            // ⚠️ A metade da ESQUERDA fica no NEUTRO de propósito: o Corner Pin não tem
            // param de aresta nenhum, e mostrar isso é a linha inteira. Uma metade
            // "equivalente" inventada seria a cena a responder por ele.
            if cured {
                g.set_param(node, "top_a_dy", BULGE);
                g.set_param(node, "top_b_dy", BULGE);
            }
        }
    }
    wire(g, src, 0, node, 0)?;
    g.set_pos(node, Pos { x: 700.0, y: lane });
    Some(node)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_bezier_tests.rs"]
mod tests;

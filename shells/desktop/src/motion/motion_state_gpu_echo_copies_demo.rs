//! **AS CÓPIAS ATRASADAS** (`PH2D_GPU_COOK_DEMO=106`) — o *Shape Time Offset* do Cavalry
//! Duplicator (doc 89, folha 08).
//!
//! ⚠️ **Esta cena PRECISA de Play.**
//!
//! ```text
//!   EM CIMA   as copias andam TODAS JUNTAS -- o de sempre
//!   EM BAIXO  cada copia mostra onde a peca ESTAVA ha' um instante
//! ```
//!
//! ⚠️ **A recusa da célula dissolveu porque o SUBSTRATO mudou**, e quem o mudou foi esta
//! mesma linha: as `TimeFans` do ADR-0163 deixam um nó cozinhar a própria entrada em N
//! instantes, que é literalmente *retimar cada cópia* — e elas não existiam quando
//! *"as cópias são LINHAS, não sub-cooks"* foi escrito.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas cópias cada fileira faz.
pub(super) const COPIES: f32 = 6.0;
/// O atraso entre cópias, em segundos.
pub(super) const OFFSET: f32 = 0.12;
/// O período da volta que a peça dá.
const PERIOD: f32 = 2.4;
/// O raio da volta.
const RADIUS: f32 = 1.5;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma peça que dá voltas, clonada — com ou sem atraso entre as cópias.
fn ring(g: &mut Graph, offset: f32, row: f32, y: f32, label: &str) -> Option<NodeId> {
    let one = g.add_node("motion.grid");
    g.set_pos(one, Pos { x: 80.0, y: row });
    g.set_param(one, "rows", 1.0);
    g.set_param(one, "cols", 1.0);

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 240.0, y: row });
    g.set_param(size, "amount", 0.26);
    wire(g, one, 0, size, 0)?;

    // ⚠️ **A peça TEM de se mover**, senão as cópias atrasadas lêem o mesmo sítio e as duas
    // fileiras saem idênticas sobre produto correcto. Uma volta é o movimento em que o
    // atraso se lê de relance.
    let osc_x = g.add_node("motion.oscillator");
    g.set_pos(osc_x, Pos { x: 400.0, y: row });
    g.set_param(osc_x, "channel", 0.0); // X
    g.set_param(osc_x, "amplitude", RADIUS);
    g.set_param(osc_x, "frequency", 1.0 / PERIOD);
    wire(g, size, 0, osc_x, 0)?;

    let osc_y = g.add_node("motion.oscillator");
    g.set_pos(osc_y, Pos { x: 560.0, y: row });
    g.set_param(osc_y, "channel", 1.0); // Y
    g.set_param(osc_y, "amplitude", RADIUS);
    g.set_param(osc_y, "frequency", 1.0 / PERIOD);
    g.set_param(osc_y, "phase", 0.25); // um quarto de volta ⇒ círculo
    wire(g, osc_x, 0, osc_y, 0)?;

    let clone = g.add_node("motion.clone");
    g.set_pos(clone, Pos { x: 720.0, y: row });
    g.set_param(clone, "count", COPIES);
    // ⚠️ **Distância ZERO**: as cópias ficam empilhadas de propósito, para o que as separa
    // ser SÓ o relógio. Com um passo espacial o olho leria a fila e não o atraso.
    g.set_param(clone, "distance", 0.0);
    g.set_param(clone, "time_offset", offset);
    // O taper faz a cauda encolher, que é o que dá o sentido da leitura.
    g.set_param(clone, "scale_taper", 0.35);
    g.set_label(clone, label);
    wire(g, osc_y, 0, clone, 0)?;

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 880.0, y: row });
    g.set_param(place, "offset_y", y);
    wire(g, clone, 0, place, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1040.0, y: row });
    wire(g, place, 0, out, 0)?;
    Some(out)
}

/// **AS CÓPIAS ATRASADAS** (`PH2D_GPU_COOK_DEMO=106`).
pub(super) fn build_gpu_echo_copies_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![
        ring(g, 0.0, -240.0, 1.9, "Sem atraso (o de sempre)")?,
        ring(g, OFFSET, 120.0, -1.9, "Com atraso por copia")?,
    ])
}

#[cfg(test)]
#[path = "motion_state_gpu_echo_copies_demo_tests.rs"]
mod tests;

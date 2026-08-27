//! **A FORMA DA ONDA E O DESLIZE DA RAMPA** — a cena `=55`, o grupo O da
//! conferência (folha 06, as linhas 26 e 30).
//!
//! **Dois pares, cada um com o seu CONTROLE ao lado:**
//! - **1-2** o **Pulse Width** numa Square: a de cima passa metade do ciclo em
//!   cima, a de baixo um quinto — o ciclo de trabalho.
//! - **3-4** o **Offset** do `motion.stagger`: a mesma rampa, **ROLADA** — a
//!   peça mais alta deixa de ser a da ponta.
//!
//! ⚠️ **O primeiro par só se julga com PLAY** (é uma onda no tempo); o segundo
//! **julga-se PARADO** (é uma rampa ao longo do índice, e o `motion.stagger` não
//! lê o relógio). Uma cena com as duas naturezas é de propósito: elas dividem a
//! folha, e ver as duas leituras lado a lado é o que ensina a diferença.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 7.0;
/// Peças por fileira.
const COUNT: f32 = 24.0;
/// A amplitude, a MESMA nos dois pares.
const AMPLITUDE: f32 = 2.0;
/// O ciclo de trabalho da banda 2 (a de cima usa o neutro, `0.5`).
const PULSE_WIDTH: f32 = 0.2;
/// O deslize da banda 4.
const OFFSET: f32 = 0.35;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Termina a banda num `motion.output`, deslocada para o seu lugar.
fn place(g: &mut Graph, head: NodeId, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

fn grid(g: &mut Graph, x: f32, y: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x, y });
    g.set_param(n, "cols", COUNT);
    g.set_param(n, "rows", 1.0);
    // ⚠️ Era `spacing`, que o `motion.grid` NAO declara — um `set_param` NO-OP
    // silencioso, e a grade shipava com o `gap` de omissao (1,0), DOBRO do autorado.
    // Achado pela `validate` no varredor das 107 (auditoria de 2026-08-27).
    g.set_param(n, "gap_x", 0.5);
    g.set_param(n, "gap_y", 0.5);
    n
}

/// Uma fileira cujo Y é uma onda quadrada, com o ciclo de trabalho pedido.
fn wave_row(g: &mut Graph, pulse_width: f32, x: f32, y: f32) -> Option<NodeId> {
    let src = grid(g, x, y);
    let osc = g.add_node("motion.oscillator");
    g.set_pos(osc, Pos { x: x + 220.0, y });
    g.set_param(osc, "channel", 1.0); // Y
    g.set_param(osc, "wave", 2.0); // Square
    g.set_param(osc, "amplitude", AMPLITUDE);
    g.set_param(osc, "frequency", 0.6);
    g.set_param(osc, "pulse_width", pulse_width);
    wire(g, src, 0, osc, 0)?;
    Some(osc)
}

/// Uma fileira cujo Y é uma rampa ao longo do índice, deslizada.
fn ramp_row(g: &mut Graph, offset: f32, x: f32, y: f32) -> Option<NodeId> {
    let src = grid(g, x, y);
    let st = g.add_node("motion.stagger");
    g.set_pos(st, Pos { x: x + 220.0, y });
    g.set_param(st, "channel", 1.0); // Y
    g.set_param(st, "min", 0.0);
    g.set_param(st, "max", 2.6);
    g.set_param(st, "offset", offset);
    wire(g, src, 0, st, 0)?;
    Some(st)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_shape_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);
    for (row_i, head) in [
        wave_row(g, 0.5, 0.0, 0.0)?,
        wave_row(g, PULSE_WIDTH, 0.0, 300.0)?,
        ramp_row(g, 0.0, 0.0, 600.0)?,
        ramp_row(g, OFFSET, 0.0, 900.0)?,
    ]
    .into_iter()
    .enumerate()
    {
        let gy = row_i as f32 * 300.0;
        sinks.push(place(g, head, (1.5 - row_i as f32) * BAND_DY, 440.0, gy)?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "PULSE WIDTH, controle -- metade do ciclo em cima, metade em baixo",
        "PULSE WIDTH = 0,2 -- ela salta e volta DEPRESSA: um quinto em cima",
        "OFFSET, controle -- a rampa sobe da esquerda para a direita",
        "OFFSET = 0,35 -- a MESMA rampa, ROLADA: o topo mudou de lugar",
    ]
    .into_iter()
    .enumerate()
}

/// Os dois números que a cena autora.
pub(crate) fn knobs() -> (f32, f32) {
    (PULSE_WIDTH, OFFSET)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_shape_tests.rs"]
mod tests;

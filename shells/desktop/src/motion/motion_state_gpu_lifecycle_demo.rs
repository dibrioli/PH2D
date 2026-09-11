//! **O RELÓGIO DA SIMULAÇÃO** (`PH2D_GPU_COOK_DEMO=103`) — o ciclo de vida da zona
//! (doc 89, folha 13, célula 60 · o *Emitter State* do Niagara).
//!
//! ⚠️ **Esta cena PRECISA de Play.**
//!
//! ```text
//!   ESQUERDA  Forever  a zona de sempre -- cai e nunca mais volta
//!   MEIO      Once     comeca depois de um atraso, corre um tempo, acaba
//!   DIREITA   Loop     corre, some, RECOMECA do princípio
//! ```
//!
//! ⚠️ **A do meio tem `Start`, e a razão é a medição:** a sonda `measure_zone_life_cycle`
//! mostrou que uma zona já está a cair no tique 0 e **nada a montante a adia** — o atraso é
//! metade do buraco que esta célula nomeia, e sem ele a cena mostraria só a outra metade.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças em cada fileira.
pub(super) const COLS: f32 = 9.0;
const GAP_X: f32 = 0.3;
const SIZE: f32 = 0.16;
/// De que altura elas caem.
const DROP_Y: f32 = 2.6;
/// A separação horizontal entre as três metades.
const SPAN_X: f32 = 3.2;

/// O atraso da metade do meio, em segundos.
pub(super) const START: f32 = 1.0;
/// Quanto tempo cada janela corre, e quanto descansa entre elas.
pub(super) const DURATION: f32 = 1.6;
pub(super) const REST: f32 = 0.6;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// Uma fileira que cai dentro de uma zona com o ciclo pedido.
fn half(g: &mut Graph, x: f32, row: f32, life: &[(&str, f32)], label: &str) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 80.0, y: row });
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", GAP_X);

    let lift = g.add_node("motion.transform");
    g.set_pos(lift, Pos { x: 240.0, y: row });
    g.set_param(lift, "offset_x", x);
    g.set_param(lift, "offset_y", DROP_Y);
    wire(g, grid, 0, lift, 0, false)?;

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 400.0, y: row });
    g.set_param(size, "amount", SIZE);
    wire(g, lift, 0, size, 0, false)?;

    let zone = g.add_node("sim.zone");
    g.set_pos(zone, Pos { x: 560.0, y: row });
    for (k, v) in life {
        g.set_param(zone, *k, *v);
    }
    g.set_label(zone, label);
    wire(g, size, 0, zone, 0, false)?;

    // O interior: gravidade + passo. ⚠️ Não há `force.gravity` neste catálogo — a gravidade é
    // o `force.wind` apontado para baixo, que é o que as cenas de chuva já fazem.
    let grav = g.add_node("force.wind");
    g.set_pos(
        grav,
        Pos {
            x: 720.0,
            y: row + 120.0,
        },
    );
    g.set_param(grav, "angle", 270.0);
    g.set_param(grav, "strength", 3.0);
    g.set_param(grav, "gust", 0.0);
    wire(g, zone, 0, grav, 0, true)?;

    let step = g.add_node("sim.step");
    g.set_pos(
        step,
        Pos {
            x: 880.0,
            y: row + 120.0,
        },
    );
    g.set_param(step, "damping", 1.0);
    wire(g, grav, 0, step, 0, false)?;
    wire(g, step, 0, zone, 1, false)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1040.0, y: row });
    wire(g, zone, 0, out, 0, false)?;
    Some(out)
}

/// **O RELÓGIO DA SIMULAÇÃO** (`PH2D_GPU_COOK_DEMO=103`).
pub(super) fn build_gpu_lifecycle_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![
        half(g, -SPAN_X, -260.0, &[], "Forever (o de sempre)")?,
        half(
            g,
            0.0,
            40.0,
            &[("mode", 1.0), ("start", START), ("duration", DURATION)],
            "Once (atrasada, com fim)",
        )?,
        half(
            g,
            SPAN_X,
            340.0,
            &[("mode", 2.0), ("duration", DURATION), ("loop_delay", REST)],
            "Loop (recomeca)",
        )?,
    ])
}

#[cfg(test)]
#[path = "motion_state_gpu_lifecycle_demo_tests.rs"]
mod tests;

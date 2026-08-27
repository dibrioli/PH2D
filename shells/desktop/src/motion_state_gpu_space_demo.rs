//! **O EIXO E A MÁSCARA** (`PH2D_GPU_COOK_DEMO=104`) — as duas metades da célula 41 da
//! folha 06 (o *Transform Space* e o *Transform Mode* do C4D).
//!
//! ⚠️ **Esta cena é ESTÁTICA** — não precisa de Play.
//!
//! ```text
//!   EM CIMA   World x Element   empurrar "para a direita" contra "para a FRENTE de cada um"
//!   EM BAIXO  Set   x Remap     o que a mascara faz onde ela vale ZERO
//! ```
//!
//! ⚠️ **A fileira de cima só diz alguma coisa porque as peças estão VIRADAS para lados
//! diferentes** — num leque todo alinhado os dois espaços dão o mesmo desenho, e a cena
//! mostraria duas metades idênticas sobre produto correcto.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças em cada leque.
pub(super) const FAN: f32 = 12.0;
/// Quanto cada peça é empurrada, em unidades de mundo.
pub(super) const PUSH: f32 = 1.1;
/// A separação horizontal entre as metades.
const SPAN_X: f32 = 3.0;
/// A meia-largura da máscara da fileira de baixo.
pub(super) const MASK_W: f32 = 2.2;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Uma constante — o `value.lfo` com amplitude zero é a mais barata do catálogo.
fn constant(g: &mut Graph, v: f32, at: Pos) -> NodeId {
    let k = g.add_node("value.lfo");
    g.set_pos(k, at);
    g.set_param(k, "amplitude", 0.0);
    g.set_param(k, "offset", v);
    k
}

/// **A metade de cima** — um leque de peças viradas, empurradas num espaço ou no outro.
fn fan(g: &mut Graph, x: f32, space: f32, row: f32, label: &str) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 80.0, y: row });
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", FAN);
    g.set_param(grid, "gap_x", 0.34);

    // ⚠️ **As peças TÊM de estar viradas**, senão os dois espaços coincidem — ver o cabeçalho.
    let turn = g.add_node("motion.stagger");
    g.set_pos(turn, Pos { x: 240.0, y: row });
    g.set_param(turn, "channel", 2.0); // Rotation
    g.set_param(turn, "min", 0.0);
    g.set_param(turn, "max", 330.0);
    wire(g, grid, 0, turn, 0)?;

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 400.0, y: row });
    g.set_param(size, "uniform", 0.0);
    g.set_param(size, "amount", 0.3);
    g.set_param(size, "amount_y", 0.08);
    wire(g, turn, 0, size, 0)?;

    let push = g.add_node("motion.drive");
    g.set_pos(push, Pos { x: 560.0, y: row });
    g.set_param(push, "channel", 0.0); // X
    g.set_param(push, "mode", 0.0); // Add
    g.set_param(push, "space", space);
    g.set_label(push, label);
    wire(g, size, 0, push, 0)?;
    let k = constant(
        g,
        PUSH,
        Pos {
            x: 400.0,
            y: row + 110.0,
        },
    );
    wire(g, k, 0, push, 1)?;

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 720.0, y: row });
    g.set_param(place, "offset_x", x);
    g.set_param(place, "offset_y", 1.6);
    wire(g, push, 0, place, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 880.0, y: row });
    wire(g, place, 0, out, 0)?;
    Some(out)
}

/// **A metade de baixo** — uma fileira sob máscara, com o tamanho conduzido por `Set` ou
/// `Remap`. Onde a máscara vale zero, o `Set` protege o tamanho de origem e o `Remap` leva-o
/// a nada.
fn masked(g: &mut Graph, x: f32, mode: f32, row: f32, label: &str) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 80.0, y: row });
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 17.0);
    g.set_param(grid, "gap_x", 0.3);

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 240.0, y: row });
    g.set_param(size, "amount", 0.2);
    wire(g, grid, 0, size, 0)?;

    // A máscara: uma caixa larga no meio da fileira.
    let mask = g.add_node("field.box");
    g.set_pos(mask, Pos { x: 400.0, y: row });
    g.set_param(mask, "width", MASK_W);
    g.set_param(mask, "height", 4.0);
    g.set_param(mask, "soft", 0.4);
    wire(g, size, 0, mask, 0)?;

    let drive = g.add_node("motion.drive");
    g.set_pos(drive, Pos { x: 560.0, y: row });
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "mode", mode);
    g.set_label(drive, label);
    wire(g, mask, 0, drive, 0)?;
    let k = constant(
        g,
        0.34,
        Pos {
            x: 400.0,
            y: row + 110.0,
        },
    );
    wire(g, k, 0, drive, 1)?;

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 720.0, y: row });
    g.set_param(place, "offset_x", x);
    g.set_param(place, "offset_y", -1.8);
    wire(g, drive, 0, place, 0)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 880.0, y: row });
    wire(g, place, 0, out, 0)?;
    Some(out)
}

/// **O EIXO E A MÁSCARA** (`PH2D_GPU_COOK_DEMO=104`).
pub(super) fn build_gpu_space_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![
        fan(g, -SPAN_X, 0.0, -300.0, "World (o de sempre)")?,
        fan(g, SPAN_X, 1.0, -80.0, "Element (agora)")?,
        masked(g, -SPAN_X, 1.0, 160.0, "Set (o de sempre)")?,
        masked(g, SPAN_X, 7.0, 380.0, "Remap (agora)")?,
    ])
}

#[cfg(test)]
#[path = "motion_state_gpu_space_demo_tests.rs"]
mod tests;

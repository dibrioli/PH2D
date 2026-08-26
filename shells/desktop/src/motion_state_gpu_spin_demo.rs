//! **AS PEÇAS GIRAM, E O GIRO PODE PARAR** (`PH2D_GPU_COOK_DEMO=100`) — a célula do estado
//! angular (doc 89, folha 13: *POP Spin* / *POP Drag Spin*).
//!
//! ⚠️ **Esta cena PRECISA de Play** — o que ela mostra é um movimento.
//!
//! ```text
//!   EM CIMA    sem arrasto angular   cada peca gira a' SUA taxa, para sempre
//!   EM BAIXO   com arrasto angular   as mesmas taxas, e todas param
//! ```
//!
//! ⚠️ **Quem escreve o giro é o `motion.drive` no canal `Custom…`**, apontado à coluna
//! `spin` — não foi preciso canal novo. Medido (`measure_spin_authoring`): fora do laço de
//! estado o `Set` é *POP Spin* (uma taxa por peça, escrita ao nascer); DENTRO do laço o `Add`
//! é *POP Torque* e acumula — `21 417°` contra `354°` nos mesmos 2 s. *O modo não é a lei; o
//! modo mais o SÍTIO é.*
//!
//! ⚠️ **As taxas vêm de uma rampa por elemento** e não de um número só: com todas as peças a
//! girar igual, «cada uma à sua taxa» seria indistinguível de «todas à mesma», e o arrasto —
//! que é proporcional ao giro — não teria nada para separar.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por fileira.
pub(super) const COLS: f32 = 8.0;
/// O espaçamento e o tamanho de cada peça.
const GAP_X: f32 = 0.8;
const SIZE: f32 = 0.4;
/// A altura de cada fileira.
const ROW_Y: [f32; 2] = [1.1, -1.1];
/// **A TAXA DE GIRO no topo da rampa**, em graus por segundo. `360` = uma volta por segundo
/// na peça mais rápida, e a mais lenta fica parada — a faixa inteira num relance.
pub(super) const TOP_SPIN: f32 = 360.0;
/// **O ARRASTO ANGULAR da fileira de baixo.** `0,15` foi escolhido para as peças pararem
/// **dentro dos primeiros segundos** — um valor mais alto e o olho não vê a travagem, mais
/// baixo e ela acaba antes de o Enio olhar.
pub(super) const DRAG: f32 = 0.15;
/// O índice do canal `Custom…` do `motion.drive` (o último dos rótulos dele).
const CH_CUSTOM: f32 = 9.0;
/// O modo `Set`.
const SET: f32 = 1.0;

/// **AS PEÇAS GIRAM** (`PH2D_GPU_COOK_DEMO=100`).
pub(super) fn build_gpu_spin_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // Uma fileira: grade -> alto -> tamanho -> a rampa de giro -> zona -> passo.
    // `angular_damping` é o ÚNICO param que difere entre as duas chamadas.
    let mut row = |y: f32, drag: f32, at: f32| -> Option<NodeId> {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let size = g.add_node("motion.scale");
        g.set_param(size, "amount", SIZE);
        // Uma taxa por peça: `0 … TOP_SPIN`.
        let ramp = g.add_node("value.instance_field");
        g.set_param(ramp, "mode", 1.0); // Ramp
        let spin = g.add_node("motion.drive");
        g.set_param(spin, "channel", CH_CUSTOM);
        g.set_param(spin, "mode", SET);
        g.set_param(spin, "scale", TOP_SPIN);
        g.set_text_param(spin, "column", "spin");

        let zone = g.add_node("sim.zone");
        let step = g.add_node("sim.step");
        g.set_param(step, "angular_damping", drag);

        for (i, n) in [grid, place, size, ramp, spin, zone]
            .into_iter()
            .enumerate()
        {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 150.0,
                    y: at,
                },
            );
        }
        g.set_pos(
            step,
            Pos {
                x: 1030.0,
                y: at + 120.0,
            },
        );

        for (a, ap, b, bp, delayed) in [
            (grid, 0u16, place, 0u16, false),
            (place, 0, size, 0, false),
            (size, 0, ramp, 0, false),
            (size, 0, spin, 0, false),
            (ramp, 0, spin, 1, false),
            (spin, 0, zone, 0, false),
            // O laço de estado: a aresta que SAI da zona é atrasada.
            (zone, 0, step, 0, true),
            (step, 0, zone, 1, false),
        ] {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed,
            })
            .ok()?;
        }
        let out = g.add_node("motion.output");
        g.set_pos(
            out,
            Pos {
                x: 1180.0,
                y: at + 120.0,
            },
        );
        g.connect(Edge {
            from: (zone, 0),
            to: (out, 0),
            delayed: false,
        })
        .ok()?;
        Some(out)
    };

    // ⚠️ `1,0` é «sem arrasto», não «arrasto máximo» — a mesma escada do `damping` linear.
    let top = row(ROW_Y[0], 1.0, 0.0)?;
    let bottom = row(ROW_Y[1], DRAG, 420.0)?;
    g.validate(reg).ok()?;
    Some(vec![top, bottom])
}

#[cfg(test)]
#[path = "motion_state_gpu_spin_demo_tests.rs"]
mod tests;

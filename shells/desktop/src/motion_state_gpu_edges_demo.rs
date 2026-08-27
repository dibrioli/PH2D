//! **A PAREDE E O SEED** (`PH2D_GPU_COOK_DEMO=102`) — duas células da folha 06 numa cena.
//!
//! ⚠️ **Esta cena PRECISA de Play.**
//!
//! ```text
//!   EM CIMA   dois tanques   Reflect (o de sempre) x Absorb (o novo)
//!   EM BAIXO  dois cachos    um seed para a cena   x um seed por PECA
//! ```
//!
//! ## Porque as duas metades cabem no mesmo ecrã
//!
//! Elas respondem à mesma pergunta em dois sítios: *o que acontece quando duas coisas
//! partilham um número que devia ser delas?* Na onda é a energia que não tem por onde
//! sair da caixa; no ruído é o campo que duas peças no mesmo ponto lêem igual. Nos dois
//! casos o modo antigo é a metade da ESQUERDA, e ele continua a ser o default.
//!
//! ⚠️ **`damping = 0` nos dois tanques, de propósito:** com amortecimento global a caixa
//! reflectora também se cala, e a cena deixaria de mostrar o item. A zero, o que sobra é
//! só a fronteira — que é exactamente a variável.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O lado de cada tanque, em células.
pub(super) const SIDE: f32 = 21.0;
const SPACING: f32 = 0.16;
/// A velocidade da onda — abaixo do limite CFL que o nó impõe.
const WAVE_SPEED: f32 = 0.35;
/// O período da fonte, em segundos.
const PULSE: f32 = 0.5;
/// Onde os dois tanques ficam.
const POND_X: f32 = 2.1;
const POND_Y: f32 = 1.5;

/// Quantas peças tem cada cacho.
pub(super) const CLUMP: f32 = 8.0;
/// O vão entre elas — APERTADO de propósito: peças quase no mesmo ponto lêem quase o
/// mesmo número, que é a condição em que o defeito aparece.
const CLUMP_GAP: f32 = 0.04;
/// Onde os dois cachos ficam. ⚠️ **Fora dos pontos de rede**: o ruído de gradiente vale
/// zero em todo ponto inteiro, por construção, e um cacho pousado num deles não se
/// mexeria em modo nenhum — a cena mostraria duas metades paradas e iguais.
const CLUMP_X: f32 = 2.13;
const CLUMP_Y: f32 = -1.87;
const CLUMP_AMP: f32 = 1.2;
/// ⚠️ **A FEIÇÃO tem de ser MUITO maior que o cacho.** A `scale` do `motion.noise` é
/// frequência: `scale` maior ⇒ feição MENOR. A 1,0 as feições medem ~1 unidade, e o
/// cacho tem 0,42 de largura — o campo varia ao longo dele e a metade «partilhada»
/// **deforma-se** (medido: a envergadura caía de `0,42` para `0,187`), o que mata a
/// leitura *"este anda como um bloco"*. A 0,25 a feição é ~4× o cacho.
const CLUMP_SCALE: f32 = 0.25;
const CLUMP_SPEED: f32 = 0.6;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// Um tanque com a fonte no centro dele e a fronteira escolhida.
fn pond(g: &mut Graph, x: f32, edges: f32, row: f32) -> Option<NodeId> {
    let w = g.add_node("motion.wave");
    g.set_pos(w, Pos { x: 300.0, y: row });
    for (k, v) in [
        ("rows", SIDE),
        ("cols", SIDE),
        ("spacing", SPACING),
        ("speed", WAVE_SPEED),
        // Ver o doc do módulo: a zero, a fronteira é a única variável.
        ("damping", 0.0),
        ("center_x", x),
        ("edges", edges),
    ] {
        g.set_param(w, k, v);
    }
    g.set_label(
        w,
        if edges >= 0.5 {
            "Absorb (agora)"
        } else {
            "Reflect (o de sempre)"
        },
    );

    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x: 100.0, y: row });
    g.set_param(lfo, "period", PULSE);
    g.set_param(lfo, "amplitude", 1.0);
    wire(g, lfo, 0, w, 0, false)?;
    // O laço de estado — sem ele o campo é re-semeado plano a cada tique.
    wire(g, w, 0, w, 1, true)?;

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 500.0, y: row });
    g.set_param(place, "offset_y", POND_Y);
    wire(g, w, 0, place, 0, false)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 700.0, y: row });
    wire(g, place, 0, out, 0, false)?;
    Some(out)
}

/// Um cacho de peças quase coincidentes, com o seed partilhado ou por peça.
fn clump(g: &mut Graph, x: f32, own_field: f32, row: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    g.set_pos(grid, Pos { x: 100.0, y: row });
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", CLUMP);
    g.set_param(grid, "gap_x", CLUMP_GAP);
    g.set_param(grid, "gap_y", CLUMP_GAP);

    let place = g.add_node("motion.transform");
    g.set_pos(place, Pos { x: 280.0, y: row });
    g.set_param(place, "offset_x", x);
    g.set_param(place, "offset_y", CLUMP_Y);
    wire(g, grid, 0, place, 0, false)?;

    let size = g.add_node("motion.scale");
    g.set_pos(size, Pos { x: 440.0, y: row });
    g.set_param(size, "amount", 0.22);
    wire(g, place, 0, size, 0, false)?;

    let ns = g.add_node("motion.noise");
    g.set_pos(ns, Pos { x: 600.0, y: row });
    g.set_param(ns, "channel", 4.0); // Position XY
    g.set_param(ns, "amplitude", CLUMP_AMP);
    g.set_param(ns, "scale", CLUMP_SCALE);
    g.set_param(ns, "speed", CLUMP_SPEED);
    g.set_param(ns, "own_field", own_field);
    g.set_label(
        ns,
        if own_field >= 0.5 {
            "Seed por PECA (agora)"
        } else {
            "Seed da cena (o de sempre)"
        },
    );
    wire(g, size, 0, ns, 0, false)?;

    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 780.0, y: row });
    wire(g, ns, 0, out, 0, false)?;
    Some(out)
}

/// **A PAREDE E O SEED** (`PH2D_GPU_COOK_DEMO=102`).
pub(super) fn build_gpu_edges_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    Some(vec![
        pond(g, -POND_X, 0.0, -260.0)?,
        pond(g, POND_X, 1.0, -60.0)?,
        clump(g, -CLUMP_X, 0.0, 160.0)?,
        clump(g, CLUMP_X, 1.0, 360.0)?,
    ])
}

#[cfg(test)]
#[path = "motion_state_gpu_edges_demo_tests.rs"]
mod tests;

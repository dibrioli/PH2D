//! The **panel-on-the-device** scene (`PH2D_GPU_COOK_DEMO=6`), a sibling of
//! `motion_state_gpu_demos.rs`.
//!
//! It lives in its own module because it answers a different question. Every
//! scene next door asks *"does it draw?"* — throughput, simulation, the id
//! gather. This one asks ***"can you still see what it is doing?"***, which is
//! the question that kept the device path opt-in, and it is the smoke for the
//! default flipping.
//!
//! (Split at the HR-18 cap: the demos file reached 612 LOC. The seam is the
//! question, not the line count — that is just what forced the choice.)
//!
//! ⛔⛔⛔ **OS LADOS DE GRELHA DESTE FICHEIRO SÃO DERIVADOS DO TECTO DESDE 2026-09-22** — ordem do
//! dono (*«vamos efetivar o limite de 16 384»*, reafirmando a de 21/09). O `motion.grid` clampa
//! cada LADO em [`LADO_MAX_DE_GRELHA`](ph2d_nodegraph::node::LADO_MAX_DE_GRELHA), logo um literal
//! maior aqui entregaria `128` na mesma **e a cena anunciaria uma população que ela não produz**
//! (`CLAUDE.md` §5.0).
//!
//! ⚠️ **O clamp é por LADO e não pelo PRODUTO, e a diferença era visível:** o `build_grid` trunca
//! em ordem row-major, logo clampar o produto entregava as primeiras `16 384` células — um
//! `512 × 512` saía como **`32` linhas de `512`**, uma FAIXA. Nenhum gate desta casa mede a FORMA
//! de uma grelha, então isso passaria em silêncio.
//!
//! ⚠️ **E as medições que os números antigos carregavam FICAM**: elas continuam verdadeiras sobre
//! o relógio e sobre a placa; o que mudou foi o que um nó pode pedir.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// **The panel-on-the-device scene** (`PH2D_GPU_COOK_DEMO=6`) — the smoke for
/// the GPU path becoming the DEFAULT.
///
/// Every other demo here answers *"does it draw?"*. This one answers *"can you
/// still see what it is doing?"*, which is the question that kept the device path
/// opt-in: a GPU cook does not feed the CPU memo, so the graph panel's readouts,
/// postage stamps, wire march and probe all went blank exactly on the documents
/// worth watching.
///
/// So the chain is chosen to put every kind of reading on one screen at a size
/// the CPU could not hold:
///
/// ```text
///   grid 512x512 ──> oscillator(Y) ─────> drive(Size) ──> output
///        │                                    ^
///        └──> instance_field ──> math(x) ──────┘
///                       lfo ──────┘
/// ```
///
/// - **262.144 instances**, so the readout reads `262144 inst` — the number that
///   would say `48 inst` if anything downstream counted the tap's rows instead of
///   asking `CookShape`. It is the one assertion an artist can make by LOOKING.
/// - **Two kinds of readout on screen at once**: the instance nodes quote a
///   count, the `value.*` nodes quote a scalar. `value.lfo` is unconnected, so it
///   is one global oscillation — its readout **changes every frame**, which is
///   also the easiest way to see the wire march working.
/// - **`value.math` is the count law in the picture**: a length-262144 field
///   times a length-1 one, output 262.144 — the law that says *"as wide as the
///   widest input"* rather than *"as wide as port 0"*.
/// - **`motion.drive` on Size** exercises the per-param kernel variants: Size was
///   one of the channels that used to recede to the CPU, and a single receding
///   node in this chain would drop the whole thing off the device.
///
/// Fully GPU by construction — gated, not assumed (`the_panel_demo_is_fully_gpu`).
pub(super) fn build_gpu_panel_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    // 512 × 512 = 262.144 — six figures on the card, and a size the CPU pump
    // visibly labours at, so `PH2D_GPU_COOK=0` is a real A/B rather than a flag.
    g.set_param(
        grid,
        "rows",
        ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32,
    );
    g.set_param(
        grid,
        "cols",
        ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32,
    );
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0); // Y
    g.set_param(osc, "amplitude", 5.0);
    g.set_param(osc, "frequency", 0.5);
    g.set_param(osc, "phase_stagger", 0.002);

    // The VALUE branch: a per-element spatial field, modulated by one global
    // oscillation. This is the doc-12 headline (`instance_field x lfo`) and the
    // reason the count law exists.
    let field = g.add_node("value.instance_field");
    let lfo = g.add_node("value.lfo"); // unconnected: ONE global value
    g.set_param(lfo, "period", 2.0);
    g.set_param(lfo, "amplitude", 0.45);
    g.set_param(lfo, "offset", 1.0);
    let math = g.add_node("value.math");
    g.set_param(math, "op", 2.0); // Multiply

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size — a variant-covered channel
    g.set_param(drive, "scale", 1.0);

    let out = g.add_node("motion.output");
    for (n, (x, y)) in [
        (grid, (60.0, 200.0)),
        (osc, (260.0, 120.0)),
        (field, (260.0, 320.0)),
        (lfo, (260.0, 460.0)),
        (math, (460.0, 380.0)),
        (drive, (660.0, 200.0)),
        (out, (860.0, 200.0)),
    ] {
        g.set_pos(n, Pos { x, y });
    }
    for (from, to, port) in [
        (grid, osc, 0u16),
        (grid, field, 0),
        (field, math, 0),
        (lfo, math, 1),
        (osc, drive, 0),
        (math, drive, 1), // the `value` port
        (drive, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

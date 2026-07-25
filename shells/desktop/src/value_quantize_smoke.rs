//! **A cena pronta para o smoke do `value.quantize`** (`PH2D_VALUE_QUANTIZE_SMOKE=1`, doc 71).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige**
//! outra coisa. Então a cena mostra a assinatura do `value.quantize` — a
//! **escada** — do jeito mais direto: um `value.lfo` faz uma onda limpa que
//! percorre a fileira, e o `quantize` a colapsa em degraus.
//!
//! Duas fileiras de 24 instâncias, o MESMO LFO viajante, só o quantize difere:
//!
//! - **De cima (SMOOTH):** `lfo → drive(Y)`. Uma senoide contínua que ondula.
//! - **De baixo (STEPPED):** `lfo → value.quantize(step=1) → drive(Y)`. A MESMA
//!   onda snapada numa grade de 1 unidade — os pontos pousam em Y discretos: uma
//!   **senoide em degraus** que salta em vez de deslizar.
//!
//! Selecione o `value.quantize` de baixo → o painel mostra **Step** (o espaçamento
//! da grade — suba para degraus mais grossos, zere para passthrough) e **Mode**
//! (Round/Floor/Ceil). O nó cozinha **100% na GPU**, sem cair pra CPU.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A amplitude da onda em Y — o LFO oscila em `[-AMP, AMP]`, e o `step = 1` do
/// quantize corta isso em ~`2·AMP + 1` degraus visíveis.
const AMP: f32 = 3.0;
/// O espaçamento da grade da fileira STEPPED.
const STEP: f32 = 1.0;

/// Monta uma fileira `grid → move → drive(Y)` cujo valor vem de um `value.lfo`
/// viajante, opcionalmente passado por `value.quantize`. Devolve o sink.
fn row(g: &mut Graph, stepped: bool, y_off: f32, tag: &str) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let lfo = g.add_node("value.lfo");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", y_off);
    g.set_param(lfo, "wave", 0.0); // sine
    g.set_param(lfo, "period", 2.5);
    g.set_param(lfo, "amplitude", AMP);
    g.set_param(lfo, "phase_stagger", 0.25); // a travelling wave across the row
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional staircase: snap the wave to a `STEP` grid.
    let quant = stepped.then(|| {
        let q = g.add_node("value.quantize");
        g.set_param(q, "step", STEP);
        g.set_pos(q, Pos { x: 440.0, y: 300.0 });
        q
    });
    // The value the drive reads: the quantize when stepped, else the raw LFO.
    let value_src = quant.unwrap_or(lfo);

    for (n, (x, y)) in [
        (grid, (60.0, 200.0)),
        (mv, (240.0, 120.0)),
        (lfo, (240.0, 300.0)),
        (drive, (640.0, 200.0)),
        (out, (840.0, 200.0)),
    ] {
        g.set_pos(n, Pos { x, y });
    }

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),  // geometry into drive's `in`
        (grid, lfo, 0),  // lfo reads the grid for count
        (value_src, drive, 1), // the (maybe quantized) value into drive's `value`
        (drive, out, 0),
    ];
    if let Some(q) = quant {
        edges.push((lfo, q, 0)); // the wave into the quantize
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    let label_node = quant.unwrap_or(lfo);
    g.set_label(label_node, tag);
    g.set_label(out, tag);
    Some(out)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_QUANTIZE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_quantize_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a onda SUAVE; de baixo a MESMA onda em degraus.
        let smooth = row(g, false, 2.4, "SMOOTH");
        let stepped = row(g, true, -2.4, "STEPPED");
        gfx.motion.sinks.extend(smooth.into_iter().chain(stepped));
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

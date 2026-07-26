//! **A cena pronta para o smoke do `value.reduce`** (`PH2D_VALUE_REDUCE_SMOKE=1`, doc 76).
//!
//! O `value.reduce` dobra um campo inteiro num número (Sum/Mean/Min/Max) e o
//! **transmite de volta como campo constante**. A cena mostra isso do jeito mais
//! direto: um driver variado (uma rampa `[1, 5]`) e a média DELE, lado a lado.
//!
//! Duas fileiras de 24 instâncias:
//!
//! - **De baixo (RAW):** o driver `[1, 5]` direto em Y — uma **rampa** que sobe de
//!   `1` a `5` (a referência, sem marca).
//! - **De cima (REDUCE, Mean):** o MESMO driver `→ value.reduce(Mean) → drive(Y)`.
//!   O `Mean` é `3` (a média da rampa) transmitido a TODAS as instâncias, então a
//!   fileira é uma **linha PLANA** — a média da fileira de baixo. É a única forma
//!   de tornar um valor RELATIVO ao campo inteiro.
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.reduce` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Mode** (Sum · Mean · Min · Max). Troque para
//! **Min** (a linha plana desce a `1`), **Max** (sobe a `5`) ou **Sum** (dispara
//! para o total). ⚠️ Compondo com um `value.math(Subtract)` — o driver menos a sua
//! média — você **centra** o campo em zero; com `Divide` pela `Sum`, vira
//! distribuição. O nó cozinha **100% na GPU** (reduce → broadcast; Min/Max
//! bit-exatos, paridade de dispositivo).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O range do driver: a rampa vai de `BASE_LO` a `BASE_HI`, então o `Mean` é o meio.
const BASE_LO: f32 = 1.0;
const BASE_HI: f32 = 5.0;

/// Monta uma fileira `grid → move → drive(Y)`. O driver é um `instance_field(Ramp)`
/// esticado para `[BASE_LO, BASE_HI]` por um `value.map_range`. Se `reduce_it`, ele
/// passa por `value.reduce(Mean)` (a fileira vira plana na média) antes do drive;
/// senão vai a rampa crua em Y. `canvas_dy` desloca a fileira na tela. Devolve
/// `(sink, hero)`: o sink e o `value.reduce` a avaliar (só a fileira de cima tem um).
fn row(g: &mut Graph, reduce_it: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let base = g.add_node("value.map_range");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(base, "out_lo", BASE_LO); // the driver: a ramp [1, 5]
    g.set_param(base, "out_hi", BASE_HI);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional reduce: fold the driver to its aggregate before it drives Y.
    let reduce = reduce_it.then(|| {
        let vr = g.add_node("value.reduce");
        g.set_param(vr, "mode", 1.0); // Mean (the average, broadcast flat)
        vr
    });
    // What feeds the drive: the reduce when present, else the raw ramp.
    let value_src = reduce.unwrap_or(base);

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, field, 0),      // instance_field reads the grid for count
        (field, base, 0),      // the ramp into the base map
        (value_src, drive, 1), // the (maybe reduced) value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(vr) = reduce {
        edges.push((base, vr, 0)); // the ramp into the reduce
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.reduce is the node under evaluation (only the top row has one).
    Some((out, reduce))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_REDUCE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_reduce_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a linha plana da média (value.reduce, marcado); de baixo a rampa.
        let reduced = row(g, true, 2.4);
        let raw = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [reduced, raw].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

//! **A cena pronta para o smoke do `value.normalize`** (`PH2D_VALUE_NORMALIZE_SMOKE=1`, doc 74).
//!
//! O `value.normalize` **descobre** o range de um campo (o próprio min/max) e o
//! ajusta a `[0,1]` — sem você digitar min/max, o que um `value.map_range` exige.
//! A cena mostra exatamente isso: um driver num range ARBITRÁRIO `[2, 10]` (o que
//! um `value.noise` / `instance_field` Random te dá — range desconhecido), e o
//! normalize o encaixa.
//!
//! Duas fileiras de 24 instâncias, o MESMO driver `[2, 10]`, só o ajuste difere:
//!
//! - **De cima (NORMALIZE):** `driver → value.normalize(Range) → map_range([0,1] → [0, ARCH]) → drive(Y)`.
//!   O normalize acha `min = 2`, `max = 10` e mapeia para `[0,1]`; depois a escala
//!   leva à altura da fileira. A rampa **ancora no chão e sobe até `ARCH`** —
//!   encaixada, qualquer que fosse o range cru.
//! - **De baixo (RAW):** o MESMO driver **direto** em Y, sem normalize — as
//!   instâncias ficam em `[2, 10]`: **deslocadas para cima** (offset de 2) e **duas
//!   vezes mais altas** que a fileira de cima. É o range cru, não encaixado (a
//!   referência, sem marca).
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.normalize` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Mode** (Range = `[0,1]` pelo min/max · MaxAbs =
//! `[-1,1]` preservando sinal e zero, para um sinal bipolar). Troque para MaxAbs e
//! veja a fileira re-centrar. O nó cozinha **100% na GPU** (`reduce → broadcast →
//! map`: as reduções Min/Max são bit-exatas, paridade de dispositivo).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A altura da fileira NORMALIZE — o `value.normalize` produz `[0,1]`, e o
/// `value.map_range` o leva para `[0, ARCH]`.
const ARCH: f32 = 4.0;
/// O range ARBITRÁRIO do driver (o "range desconhecido" que o normalize descobre).
const RAW_LO: f32 = 2.0;
const RAW_HI: f32 = 10.0;

/// Monta uma fileira `grid → move → drive(Y)`. O driver é um `instance_field(Ramp)`
/// esticado para `[RAW_LO, RAW_HI]` por um `value.map_range`. Se `normalized`, ele
/// passa por `value.normalize(Range)` → `map_range([0,1] → [0, ARCH])` antes do
/// drive; senão vai cru em Y. `canvas_dy` desloca a fileira na tela. Devolve
/// `(sink, hero)`: o sink e o `value.normalize` a avaliar (só a fileira de cima tem um).
fn row(g: &mut Graph, normalized: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let raw = g.add_node("value.map_range");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(raw, "out_lo", RAW_LO); // the raw driver: an arbitrary range [2, 10]
    g.set_param(raw, "out_hi", RAW_HI);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The normalize + rescale, or nothing (the raw driver straight into Y).
    let (value_src, hero) = if normalized {
        let norm = g.add_node("value.normalize");
        g.set_param(norm, "mode", 0.0); // Range -> [0,1] by the field's own min/max
        let scale = g.add_node("value.map_range");
        g.set_param(scale, "out_lo", 0.0); // the fitted [0,1] -> [0, ARCH] for Y
        g.set_param(scale, "out_hi", ARCH);
        // raw -> normalize -> scale
        g.connect(Edge { from: (raw, 0), to: (norm, 0), delayed: false }).ok()?;
        g.connect(Edge { from: (norm, 0), to: (scale, 0), delayed: false }).ok()?;
        (scale, Some(norm))
    } else {
        (raw, None)
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, field, 0),      // instance_field reads the grid for count
        (field, raw, 0),       // the ramp into the raw-range map
        (value_src, drive, 1), // the (maybe normalized) value into drive's `value` port
        (drive, out, 0),
    ];
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.normalize is the node under evaluation (only the top row has one).
    Some((out, hero))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_NORMALIZE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_normalize_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o driver encaixado (value.normalize, marcado); de baixo o cru.
        let fitted = row(g, true, 2.4);
        let raw = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [fitted, raw].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

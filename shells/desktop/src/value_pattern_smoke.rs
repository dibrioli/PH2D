//! **A cena pronta para o smoke do `value.pattern`** (`PH2D_VALUE_PATTERN_SMOKE=1`, doc 78).
//!
//! O `value.pattern` deixa você AUTORAR uma lista explícita de valores que se
//! repete pelas instâncias — o step sequencer. Enquanto o `instance_field` e o
//! `value.noise` GERAM um campo por fórmula, este você DIGITA. A cena mostra uma
//! batida autorada contra uma rampa lisa.
//!
//! Duas fileiras de 24 instâncias:
//!
//! - **De cima (PATTERN):** `grid → value.pattern(Steps 4) → drive(Y)`, com os 4
//!   valores `[0.15, 0.6, 0.35, 1.0]`. Eles se repetem 6× pelas 24 instâncias — uma
//!   **batida autorada** de 4 tempos, um ritmo que nenhum produtor procedural dá.
//! - **De baixo (RAMP):** `instance_field(Ramp)` — uma **rampa lisa** de `0` a `1`
//!   (a referência procedural, sem marca).
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.pattern` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Steps** (quantos slots ciclam, `1..8`) e os
//! oito valores **V0..V7**. Mude **V2** e veja o 3º tempo saltar; suba **Steps**
//! para `8` e edite V4..V7 para uma batida mais longa. O nó cozinha **100% na GPU**
//! (os valores SÃO o uniforme; paridade de dispositivo bit-exata).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Os quatro valores da batida autorada (um ritmo irregular, claramente à mão).
const BEAT: [f32; 4] = [0.15, 0.6, 0.35, 1.0];
/// A escala de altura no drive — os valores `[0,1]` sobem até `HEIGHT`.
const HEIGHT: f32 = 3.0;

/// Monta uma fileira `grid → move → drive(Y)`. Se `use_pattern`, o valor vem de um
/// `value.pattern` (a batida autorada); senão de um `instance_field(Ramp)` (a rampa
/// lisa). Os dois leem o grid para a contagem. `canvas_dy` desloca a fileira.
/// Devolve `(sink, hero)`: o sink e o `value.pattern` a avaliar (só a fileira de
/// cima tem um).
fn row(g: &mut Graph, use_pattern: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT); // [0,1] value -> [0, HEIGHT]

    // The value source: an authored pattern, or a procedural ramp. Both read the
    // grid for their count.
    let (value_src, hero) = if use_pattern {
        let vp = g.add_node("value.pattern");
        g.set_param(vp, "steps", 4.0);
        g.set_param(vp, "v0", BEAT[0]);
        g.set_param(vp, "v1", BEAT[1]);
        g.set_param(vp, "v2", BEAT[2]);
        g.set_param(vp, "v3", BEAT[3]);
        (vp, Some(vp))
    } else {
        let field = g.add_node("value.instance_field");
        g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
        (field, None)
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, value_src, 0),  // the producer reads the grid for its count
        (value_src, drive, 1), // the authored/procedural value into drive's `value` port
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

    // The value.pattern is the node under evaluation (only the top row has one).
    Some((out, hero))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_PATTERN_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_pattern_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a batida autorada (value.pattern, marcado); de baixo a rampa.
        let patterned = row(g, true, 2.4);
        let ramp = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [patterned, ramp].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

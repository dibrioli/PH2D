//! **A cena pronta para o smoke do `value.wrap`** (`PH2D_VALUE_WRAP_SMOKE=1`, doc 79).
//!
//! O `value.wrap` decide o que acontece quando um campo passa das bordas de uma
//! faixa `[min, max]` — o modo de endereçamento de textura (Clamp/Repeat/Mirror),
//! o loopOut do After Effects (Continue/Cycle/PingPong). A cena mostra os TRÊS
//! modos lado a lado, alimentados pela MESMA rampa esticada.
//!
//! Três fileiras de 24 instâncias. Cada uma é
//! `grid -> instance_field(Ramp) -> map_range(estica p/ [0, 3]) -> wrap(<modo>) ->
//! drive(Y)`, então a rampa cobre TRÊS larguras da faixa `[0, 1]` e o modo decide
//! a silhueta que aparece na altura:
//!
//! - **De cima (REPEAT):** um **dente de serra** — a rampa sobe, salta de `max`
//!   de volta a `min`, sobe de novo: três dentes. (marcada `>> EVALUATE <<`.)
//! - **Do meio (MIRROR):** um **triângulo** — sobe até `max`, desce até `min` e
//!   volta, período `2w`: a zig-zag.
//! - **De baixo (CLAMP):** um **platô** — sobe uma vez e trava em `max`; o resto
//!   da rampa fica preso na borda.
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.wrap` marcado é o de cima (Repeat). Selecione-o →
//! o painel mostra **Min** / **Max** (a faixa em que a rampa se dobra) e **Mode**
//! (Clamp/Repeat/Mirror). Aperte a faixa (**Max** menor) e veja MAIS dentes; troque
//! o **Mode** e veja a mesma rampa virar serra, triângulo ou platô. O nó cozinha
//! **100% na GPU** (a dobra é `floor`/`clamp`; paridade de dispositivo em ε).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A escala de altura no drive — os valores `[0,1]` (a faixa em que a rampa se
/// dobra) sobem até `HEIGHT`.
const HEIGHT: f32 = 2.6;
/// Quantas larguras da faixa a rampa cobre (o `out_hi` do `map_range`): 3 larguras
/// = três dentes no Repeat, uma volta e meia no Mirror.
const SPAN: f32 = 3.0;

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de
/// `instance_field(Ramp) -> map_range([0, SPAN]) -> wrap(mode)`. `canvas_dy`
/// desloca a fileira; `mark` diz se o `value.wrap` é o nó a avaliar. Devolve
/// `(sink, hero)`.
fn row(g: &mut Graph, mode: f32, canvas_dy: f32, mark: bool) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let map = g.add_node("value.map_range");
    let wrap = g.add_node("value.wrap");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    // Stretch the [0,1] ramp across SPAN widths of the wrap range so the fold does
    // real work (Repeat tiles it SPAN times, Clamp plateaus past the first).
    g.set_param(map, "out_lo", 0.0);
    g.set_param(map, "out_hi", SPAN);
    g.set_param(wrap, "lo", 0.0);
    g.set_param(wrap, "hi", 1.0);
    g.set_param(wrap, "mode", mode);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT); // [0,1] fold -> [0, HEIGHT]

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),   // geometry into drive's `in`
        (grid, field, 0), // the ramp reads the grid for its count
        (field, map, 0),  // stretch the ramp past the range
        (map, wrap, 0),   // fold it back in by `mode`
        (wrap, drive, 1), // the wrapped value into drive's `value` port
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

    Some((out, mark.then_some(wrap)))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_WRAP_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_wrap_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima Repeat (marcado), meio Mirror, de baixo Clamp — os três modos de
        // endereçamento sobre a MESMA rampa esticada.
        let repeat = row(g, 1.0, 2.4, true);
        let mirror = row(g, 2.0, 0.0, false);
        let clamp = row(g, 0.0, -2.4, false);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [repeat, mirror, clamp].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

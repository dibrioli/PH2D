//! **A cena pronta para o smoke do `value.time`** (`PH2D_VALUE_TIME_SMOKE=1`, doc 80).
//!
//! O `value.time` leva o RELÓGIO animado (o playhead) para o grafo de valor como
//! um número simples -- o `$T`/`@Time` do Houdini, o Timer CHOP do TD. Ele é
//! **MONOTÔNICO** (sobe pra sempre), ao contrário do `value.lfo(Saw)` que dobra a
//! cada período -- e é isso que o torna o par natural do `value.wrap`.
//!
//! ⚠️ **APERTE PLAY.** value.time é temporal: parado (t=0) mostra só uma rampa
//! espacial (o `stagger`); tocando, o relógio SOBE e a cena anima. A lição está no
//! contraste das duas fileiras.
//!
//! Duas fileiras de 24 instâncias:
//!
//! - **De cima (RAW / cru):** `grid -> value.time(stagger) -> drive(Y)`. O relógio
//!   cru dirige Y direto -- tocando, os pontos SOBEM juntos (uma diagonal por causa
//!   do stagger) e **saem de quadro**: o relógio nunca volta. (marcada
//!   `>> EVALUATE <<`.)
//! - **De baixo (WRAPPED / dobrado):** o MESMO relógio dobrado por
//!   `value.wrap(Repeat, [0,1])` antes do drive -- tocando, os pontos sobem e
//!   **saltam de volta** num laço de dente de serra, presos na tela pra sempre. É
//!   `time -> wrap`: o relógio domado num loop controlável.
//!
//! O grafo inteiro é arrumado pelo auto-layout ciente de subgrupos
//! (`smoke_layout`); o `value.time` marcado é o de cima. Selecione-o -> o painel
//! mostra **Rate** (velocidade do relógio; negativo = anda pra trás), **Offset**
//! (onde começa) e **Stagger** (deslocamento por-instância = a diagonal). O nó
//! cozinha **100% na GPU** (uma multiplicação-soma; paridade de dispositivo em ε).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A escala de altura no drive.
const HEIGHT: f32 = 2.0;
/// Velocidade do relógio (unidades de valor por segundo) -- devagar o bastante
/// para a fileira crua levar alguns segundos para sair de quadro.
const RATE: f32 = 0.5;
/// Deslocamento por-instância: a diagonal que revela que cada ponto lê o mesmo
/// relógio deslocado.
const STAGGER: f32 = 0.12;

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de
/// `value.time(RATE, STAGGER)` -- dobrado por `value.wrap(Repeat)` se `wrap_it`.
/// `canvas_dy` desloca a fileira; `mark` diz se o `value.time` é o nó a avaliar.
fn row(g: &mut Graph, wrap_it: bool, canvas_dy: f32, mark: bool) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let time = g.add_node("value.time");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(time, "rate", RATE);
    g.set_param(time, "stagger", STAGGER);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT);

    // The value source: the raw clock, optionally folded into a loop by `wrap`.
    let value_src = if wrap_it {
        let wrap = g.add_node("value.wrap");
        g.set_param(wrap, "lo", 0.0);
        g.set_param(wrap, "hi", 1.0);
        g.set_param(wrap, "mode", 1.0); // Repeat — the sawtooth loop
        g.connect(Edge {
            from: (time, 0),
            to: (wrap, 0),
            delayed: false,
        })
        .ok()?;
        wrap
    } else {
        time
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),         // geometry into drive's `in`
        (grid, time, 0),        // the clock reads the grid for its count
        (value_src, drive, 1),  // the clock (raw or wrapped) into drive's `value` port
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

    Some((out, mark.then_some(time)))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_TIME_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_time_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o relógio cru (marcado, sobe e sai); de baixo o mesmo dobrado
        // por wrap (dente de serra em loop).
        let raw = row(g, false, 2.4, true);
        let wrapped = row(g, true, -2.4, false);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [raw, wrapped].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

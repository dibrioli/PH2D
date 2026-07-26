//! **A cena pronta para o smoke do `value.wave`** (`PH2D_VALUE_WAVE_SMOKE=1`, doc 84).
//!
//! O `value.wave` MOLDA qualquer campo, lido como FASE, numa forma de onda -- o dual
//! SHAPER do `value.lfo` (o lfo PRODUZ do playhead; este molda a entrada). Alimente
//! uma rampa e ele desenha uma ONDA ESTACIONÁRIA espacial pela grade. A cena mostra
//! as quatro formas clássicas sobre a MESMA rampa.
//!
//! Quatro fileiras de 24 instâncias. Cada uma é
//! `grid -> instance_field(Ramp) -> wave(<forma>, freq 2) -> drive(Y)`, então a
//! rampa `[0,1]` vira DUAS ondulações da forma escolhida (bipolar, `[-1,1]`):
//!
//! - **De cima (SINE):** a senoide lisa. (marcada `>> EVALUATE <<`.)
//! - **TRIANGLE:** o zigue-zague.
//! - **SQUARE:** a onda quadrada em degraus.
//! - **SAW:** o dente-de-serra.
//!
//! O grafo é arrumado pelo auto-layout ciente de subgrupos (`smoke_layout`); o
//! `value.wave` marcado é o de cima (Sine). Selecione-o -> o painel mostra **Wave**
//! (troque para Tri/Square/Saw/Spike), **Frequency** (mais ondulações), **Amplitude**
//! / **Offset** (altura e centro) e **Phase**. ⚠️ NÃO confunda com o `value.wrap`:
//! o wave dá waveforms BIPOLARES de oscilador (para dirigir movimento), o wrap dobra
//! num RANGE (para ladrilhar). O nó cozinha **100% na GPU** (mesmo banco
//! transcendental-free do lfo; paridade de dispositivo em ε).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A escala de altura no drive (a onda bipolar `[-1,1]` -> `[-HEIGHT, HEIGHT]`).
const HEIGHT: f32 = 1.1;
/// Quantas ondulações a rampa `[0,1]` desenha.
const FREQ: f32 = 2.0;

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de
/// `instance_field(Ramp) -> wave(kind, FREQ)`. `canvas_dy` desloca a fileira;
/// `mark` diz se o `value.wave` é o nó a avaliar. Devolve `(sink, hero)`.
fn row(g: &mut Graph, kind: f32, canvas_dy: f32, mark: bool) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let wave = g.add_node("value.wave");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(wave, "wave", kind);
    g.set_param(wave, "frequency", FREQ);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT);

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),      // geometry into drive's `in`
        (grid, field, 0),    // the ramp reads the grid for its count
        (field, wave, 0),    // the ramp is the PHASE
        (wave, drive, 1),    // the waveform value into drive's `value` port
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

    Some((out, mark.then_some(wave)))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_WAVE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_wave_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // Sine (marcado) / Triangle / Square / Saw -- as quatro formas na mesma rampa.
        let sine = row(g, 0.0, 4.5, true);
        let tri = row(g, 1.0, 1.5, false);
        let square = row(g, 2.0, -1.5, false);
        let saw = row(g, 3.0, -4.5, false);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [sine, tri, square, saw].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

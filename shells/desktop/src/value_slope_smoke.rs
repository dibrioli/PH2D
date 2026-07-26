//! **A cena pronta para o smoke do `value.slope`** (`PH2D_VALUE_SLOPE_SMOKE=1`, doc 81).
//!
//! O `value.slope` é a DERIVADA do campo -- a taxa de mudança ao longo da ordem
//! das instâncias, o irmão exato do `value.smooth`: onde o smooth faz a MÉDIA de
//! cada elemento com os vizinhos (passa-baixa, borra), este os SUBTRAI
//! (passa-alta, acha as bordas). A cena mostra os dois lado a lado sobre o MESMO
//! campo escalonado, para o par ficar óbvio.
//!
//! Três fileiras de 24 instâncias, a MESMA `value.pattern` escalonada em cada uma
//! (quatro valores distintos repetidos, degraus com platôs e saltos claros):
//!
//! - **De cima (RAW):** `pattern -> drive(Y)`. O campo cru -- degraus.
//! - **Do meio (SMOOTH):** `pattern -> smooth(radius) -> drive(Y)`. Os degraus
//!   AMACIADOS (o passa-baixa).
//! - **De baixo (SLOPE):** `pattern -> slope(scale) -> drive(Y)`. Zero nos platôs,
//!   um PICO em cada salto -- as bordas, a detecção de arestas. (marcada
//!   `>> EVALUATE <<`.)
//!
//! O grafo inteiro é arrumado pelo auto-layout ciente de subgrupos
//! (`smoke_layout`); o `value.slope` marcado é o de baixo. Selecione-o -> o painel
//! mostra **Scale** (amplifica a derivada; negativo inverte o sinal). Suba o Scale
//! e os picos crescem; o platô fica em zero. O nó cozinha **100% na GPU** (uma
//! subtração de vizinhos; paridade de dispositivo em ε).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Os quatro valores do padrão escalonado (alturas distintas -> platôs e saltos).
const STEPS: [f32; 4] = [0.1, 0.6, 0.3, 0.9];
/// A escala de altura no drive.
const HEIGHT: f32 = 2.4;

/// Qual filtro a fileira aplica ao padrão antes do drive.
#[derive(Clone, Copy)]
enum Kind {
    Raw,
    Smooth,
    Slope,
}

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de uma
/// `value.pattern` escalonada, opcionalmente filtrada por `smooth` ou `slope`.
/// `canvas_dy` desloca a fileira; `mark` diz se o nó filtro é o de avaliar.
/// Devolve `(sink, hero)`; o hero (se marcado) é o nó de slope.
fn row(g: &mut Graph, kind: Kind, canvas_dy: f32, mark: bool) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let pat = g.add_node("value.pattern");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(pat, "steps", 4.0);
    g.set_param(pat, "v0", STEPS[0]);
    g.set_param(pat, "v1", STEPS[1]);
    g.set_param(pat, "v2", STEPS[2]);
    g.set_param(pat, "v3", STEPS[3]);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT);

    // The value source: the raw pattern, or its low-pass (smooth) / high-pass
    // (slope) filter. `hero` is the slope node when marked.
    let (value_src, hero) = match kind {
        Kind::Raw => (pat, None),
        Kind::Smooth => {
            let sm = g.add_node("value.smooth");
            g.set_param(sm, "radius", 2.0);
            g.connect(Edge {
                from: (pat, 0),
                to: (sm, 0),
                delayed: false,
            })
            .ok()?;
            (sm, None)
        }
        Kind::Slope => {
            let sl = g.add_node("value.slope");
            g.set_param(sl, "scale", 1.5); // amplify the edge spikes into view
            g.connect(Edge {
                from: (pat, 0),
                to: (sl, 0),
                delayed: false,
            })
            .ok()?;
            (sl, mark.then_some(sl))
        }
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, pat, 0),        // the pattern reads the grid for its count
        (value_src, drive, 1), // the field (raw/smoothed/slope) into drive's value port
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

    Some((out, hero))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_SLOPE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_slope_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o campo cru, meio o smooth (borra), baixo o slope (bordas, marcado).
        let raw = row(g, Kind::Raw, 2.4, false);
        let smooth = row(g, Kind::Smooth, 0.0, false);
        let slope = row(g, Kind::Slope, -2.4, true);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [raw, smooth, slope].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

//! **A cena pronta para o smoke do `value.median`** (`PH2D_VALUE_MEDIAN_SMOKE=1`, doc 82).
//!
//! O `value.median` é o filtro NÃO-LINEAR -- o irmão do `value.smooth` que o smooth
//! não pode ser. O smooth faz a MÉDIA (linear): um outlier vaza para os vizinhos e
//! toda borda amacia. A mediana escolhe o valor do MEIO (estatística de ordem): um
//! spike é DELETADO e uma borda é MANTIDA. É o removedor de ruído sal-e-pimenta. A
//! cena mostra os dois sobre o MESMO campo com spikes e uma borda.
//!
//! Três fileiras de 24 instâncias, a MESMA `value.pattern` de 8 valores repetida 3×
//! -- um platô baixo com um SPIKE (sal, 0.9), a borda 0.2->0.7, e um POÇO (pimenta,
//! 0.1) no platô alto:
//!
//! - **De cima (RAW):** `pattern -> drive(Y)`. O campo cru -- os spikes e a borda.
//! - **Do meio (SMOOTH):** `pattern -> smooth(radius) -> drive(Y)`. Os spikes viram
//!   corcovas e a borda VIRA RAMPA -- o passa-baixa borra tudo.
//! - **De baixo (MEDIAN):** `pattern -> median(radius) -> drive(Y)`. Os spikes
//!   SOMEM e a borda fica AFIADA -- estatística de ordem, preservando as bordas.
//!   (marcada `>> EVALUATE <<`.)
//!
//! O grafo inteiro é arrumado pelo auto-layout ciente de subgrupos
//! (`smoke_layout`); o `value.median` marcado é o de baixo. Selecione-o -> o painel
//! mostra **Radius** (a meia-janela; 0 = passthrough, 1 = mediana-de-3). Suba o
//! Radius e os spikes largos somem enquanto a borda resiste; compare com o smooth do
//! meio, que amacia a borda junto. O nó cozinha **100% na GPU** (seleção por rank;
//! paridade de dispositivo BIT-exata -- escolhe uma amostra existente, sem média).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O padrão de 8 valores: platô baixo, SPIKE (sal), borda 0.2->0.7, POÇO (pimenta).
const PATTERN: [f32; 8] = [0.2, 0.2, 0.9, 0.2, 0.7, 0.7, 0.1, 0.7];
/// A escala de altura no drive.
const HEIGHT: f32 = 2.4;
/// A meia-janela dos dois filtros (mesma para a comparação ser justa).
const RADIUS: f32 = 2.0;

/// Qual filtro a fileira aplica ao padrão antes do drive.
#[derive(Clone, Copy)]
enum Kind {
    Raw,
    Smooth,
    Median,
}

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de uma
/// `value.pattern` de 8 valores, opcionalmente filtrada por `smooth` ou `median`.
/// `canvas_dy` desloca a fileira; `mark` diz se o nó filtro é o de avaliar.
/// Devolve `(sink, hero)`; o hero (se marcado) é o nó de median.
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
    g.set_param(pat, "steps", 8.0);
    for (k, &v) in PATTERN.iter().enumerate() {
        g.set_param(pat, &format!("v{k}"), v);
    }
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", HEIGHT);

    // The value source: the raw pattern, or its linear (smooth) / order-statistic
    // (median) filter. `hero` is the median node when marked.
    let (value_src, hero) = match kind {
        Kind::Raw => (pat, None),
        Kind::Smooth => {
            let sm = g.add_node("value.smooth");
            g.set_param(sm, "radius", RADIUS);
            g.connect(Edge {
                from: (pat, 0),
                to: (sm, 0),
                delayed: false,
            })
            .ok()?;
            (sm, None)
        }
        Kind::Median => {
            let md = g.add_node("value.median");
            g.set_param(md, "radius", RADIUS);
            g.connect(Edge {
                from: (pat, 0),
                to: (md, 0),
                delayed: false,
            })
            .ok()?;
            (md, mark.then_some(md))
        }
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, pat, 0),        // the pattern reads the grid for its count
        (value_src, drive, 1), // the field (raw/smoothed/median) into drive's value port
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
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_MEDIAN_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_median_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o campo cru, meio o smooth (borra spikes E bordas), baixo o median
        // (remove spikes, mantém bordas -- marcado).
        let raw = row(g, Kind::Raw, 2.4, false);
        let smooth = row(g, Kind::Smooth, 0.0, false);
        let median = row(g, Kind::Median, -2.4, true);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [raw, smooth, median].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

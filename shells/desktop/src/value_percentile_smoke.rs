//! **A cena pronta para o smoke do `value.percentile`** (`PH2D_VALUE_PERCENTILE_SMOKE=1`, doc 83).
//!
//! O `value.percentile` é o filtro MORFOLÓGICO / de rank -- troca cada elemento
//! pela `p`-ésima estatística de ordem da sua janela. As pontas são as operações
//! genuinamente novas (não uma mediana com botão):
//! - **`p = 0` (ERODE / min):** cada elemento vira o MENOR da janela -- um spike
//!   ALTO morre, um POÇO baixo se espalha (erosão; o *Minimum* do Photoshop).
//! - **`p = 0.5` (MEDIAN):** o valor do meio -- spike E poço somem.
//! - **`p = 1` (DILATE / max):** cada elemento vira o MAIOR -- um spike ALTO cresce,
//!   um poço morre (dilatação; o *Maximum* do Photoshop).
//!
//! Quatro fileiras de 24 instâncias, a MESMA `value.pattern` de 8 valores repetida
//! 3× -- um platô médio (0.5) com um SPIKE alto (0.9) e um POÇO baixo (0.1):
//!
//! - **De cima (RAW):** o campo cru -- o spike e o poço.
//! - **ERODE (`p=0`):** o spike some, o poço engorda.
//! - **MEDIAN (`p=0.5`):** spike E poço somem -- platô limpo. (marcada `>> EVALUATE <<`.)
//! - **DILATE (`p=1`):** o spike engorda, o poço some.
//!
//! O grafo é arrumado pelo auto-layout ciente de subgrupos (`smoke_layout`); o
//! `value.percentile` marcado é o do MEIO (median). Selecione-o -> o painel mostra
//! **Radius** e **Percentile**. Deslize o **Percentile** de `0` a `1` e veja a MESMA
//! fileira MORFAR erosão -> mediana -> dilatação ao vivo. O nó cozinha **100% na
//! GPU** (seleção por rank; paridade de dispositivo BIT-exata).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O padrão de 8 valores: platô médio (0.5) com um SPIKE (0.9) e um POÇO (0.1).
const PATTERN: [f32; 8] = [0.5, 0.5, 0.9, 0.5, 0.5, 0.1, 0.5, 0.5];
/// A escala de altura no drive.
const HEIGHT: f32 = 2.2;
/// A meia-janela dos filtros (mediana-de-3 = um passo morfológico limpo).
const RADIUS: f32 = 1.0;

/// Monta uma fileira `grid -> move -> drive(Y)`, com o valor vindo de uma
/// `value.pattern` de 8 valores, opcionalmente filtrada por `percentile` em `p`
/// (`None` = campo cru). `canvas_dy` desloca a fileira; `mark` marca o percentile.
/// Devolve `(sink, hero)`; o hero (se marcado) é o nó de percentile.
fn row(
    g: &mut Graph,
    p: Option<f32>,
    canvas_dy: f32,
    mark: bool,
) -> Option<(NodeId, Option<NodeId>)> {
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

    // The value source: the raw pattern, or a percentile (erode/median/dilate) of it.
    let (value_src, hero) = match p {
        None => (pat, None),
        Some(p) => {
            let pct = g.add_node("value.percentile");
            g.set_param(pct, "radius", RADIUS);
            g.set_param(pct, "percentile", p);
            g.connect(Edge {
                from: (pat, 0),
                to: (pct, 0),
                delayed: false,
            })
            .ok()?;
            (pct, mark.then_some(pct))
        }
    };

    let edges = [
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, pat, 0),        // the pattern reads the grid for its count
        (value_src, drive, 1), // the field (raw or percentile) into drive's value port
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
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_PERCENTILE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_percentile_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // Cru, erode (p=0), median (p=0.5, marcado), dilate (p=1).
        let raw = row(g, None, 3.6, false);
        let erode = row(g, Some(0.0), 1.2, false);
        let median = row(g, Some(0.5), -1.2, true);
        let dilate = row(g, Some(1.0), -3.6, false);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [raw, erode, median, dilate].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

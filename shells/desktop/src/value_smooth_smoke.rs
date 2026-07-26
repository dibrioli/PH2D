//! **A cena pronta para o smoke do `value.smooth`** (`PH2D_VALUE_SMOOTH_SMOKE=1`, doc 77).
//!
//! O `value.smooth` suaviza um campo — a média de cada elemento com os vizinhos de
//! índice (um box blur sobre a ordem das instâncias). A cena mostra isso do jeito
//! mais direto: um campo JAGGED (`instance_field` Random) e a versão suavizada
//! dele, lado a lado.
//!
//! Duas fileiras de 24 instâncias, o MESMO campo aleatório (mesma seed):
//!
//! - **De baixo (RAW):** o campo Random direto em Y — uma fileira **espinhada**,
//!   cada instância numa altura aleatória (a referência, sem marca).
//! - **De cima (SMOOTH):** o MESMO campo `→ value.smooth(Radius 4) → drive(Y)`.
//!   Cada instância vira a média das vizinhas, então a fileira fica **gradual** —
//!   a mesma silhueta, sem os dentes.
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.smooth` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Radius** (`0` = passthrough; maior = mais
//! macio). Arraste o Radius de `0` (a fileira volta a espinhar) até `8` (quase
//! plana na média). O nó cozinha **100% na GPU** (lê os vizinhos do buffer; soma
//! de ordem fixa, paridade de dispositivo bit-exata).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A altura do campo — o Random dá `[0,1)`, e o `value.map_range` o leva a `[0, HEIGHT]`.
const HEIGHT: f32 = 3.0;
/// O raio do box blur na fileira suavizada — uma janela bem visível.
const RADIUS: f32 = 4.0;
/// A seed do campo Random (a MESMA nas duas fileiras, para compararem).
const SEED: f32 = 7.0;

/// Monta uma fileira `grid → move → drive(Y)`. O driver é um `instance_field(Random)`
/// escalado para `[0, HEIGHT]` por um `value.map_range`. Se `smooth_it`, ele passa
/// por `value.smooth(Radius)` antes do drive; senão vai o campo jagged direto em Y.
/// `canvas_dy` desloca a fileira. Devolve `(sink, hero)`: o sink e o `value.smooth`
/// a avaliar (só a fileira de cima tem um).
fn row(g: &mut Graph, smooth_it: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let scale = g.add_node("value.map_range");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 2.0); // Random: a jagged per-instance field
    g.set_param(field, "seed", SEED);
    g.set_param(scale, "out_lo", 0.0); // Random [0,1) -> [0, HEIGHT] for Y
    g.set_param(scale, "out_hi", HEIGHT);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional smooth: soften the jagged field before it drives Y.
    let smooth = smooth_it.then(|| {
        let vs = g.add_node("value.smooth");
        g.set_param(vs, "radius", RADIUS);
        vs
    });
    // What feeds the drive: the smooth when present, else the raw jagged field.
    let value_src = smooth.unwrap_or(scale);

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, field, 0),      // instance_field reads the grid for count
        (field, scale, 0),     // the Random field into the scale
        (value_src, drive, 1), // the (maybe smoothed) value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(vs) = smooth {
        edges.push((scale, vs, 0)); // the scaled field into the smooth
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.smooth is the node under evaluation (only the top row has one).
    Some((out, smooth))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_SMOOTH_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_smooth_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a fileira suavizada (value.smooth, marcado); de baixo a jagged.
        let smoothed = row(g, true, 2.4);
        let raw = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [smoothed, raw].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

//! **A cena pronta para o smoke do `value.noise`** (`PH2D_VALUE_NOISE_SMOKE=1`, doc 69).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige**
//! outra coisa. Então a cena mostra a assinatura do `value.noise` — a diferença
//! entre ruído **COERENTE** e ruído **BRANCO** — do jeito mais direto: o valor
//! vira o deslocamento em Y de uma fileira.
//!
//! Duas fileiras de 24 instâncias, a MESMA geometria, só a FONTE do valor difere:
//!
//! - **De cima (NOISE):** `value.noise → motion.drive(Y)`. O campo é contínuo, então
//!   vizinhos leem pontos próximos do lattice → a fileira ondula como uma **onda
//!   suave** que **escorre com o tempo** (speed > 0). É o driver "dá vida".
//! - **De baixo (WHITE):** `value.instance_field(Random) → value.map_range →
//!   motion.drive(Y)`. Um hash por instância → vizinhos **descorrelacionados**: uma
//!   fileira **serrilhada e estática**. É o contraste que define o `value.noise`.
//!
//! O grafo é arrumado em duas **linhas horizontais retas** (`smoke_layout`), e o nó
//! marcado `>> EVALUATE <<` é o `value.noise` a avaliar. Selecione-o → o painel
//! mostra os knobs (Frequency = detalhe espacial, Speed = evolução no tempo,
//! Octaves/Roughness = o fBm). O nó cozinha 100% na GPU.

use crate::smoke_layout::{lay_horizontal, ROW_GAP};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A amplitude do deslocamento em Y — o `value.noise` mapeia o campo `[-1,1]` para
/// `[-ARCH, ARCH]`, e o `value.map_range` da fileira branca casa a mesma faixa.
const ARCH: f32 = 3.0;

/// Monta a fileira COERENTE: `grid → move → drive(Y)` com o valor vindo de um
/// `value.noise` (que lê o grid só para a contagem). Devolve o sink.
fn noise_row(g: &mut Graph, canvas_dy: f32, panel_y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let vn = g.add_node("value.noise");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(vn, "frequency", 0.28); // a smooth swell across the row
    g.set_param(vn, "speed", 1.0); // drifts over time
    g.set_param(vn, "octaves", 2.0); // a touch of fBm detail
    g.set_param(vn, "amplitude", ARCH);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, drive, 0), // geometry into drive's `in`
        (grid, vn, 0),  // value.noise reads the grid for its count
        (vn, drive, 1), // the noise value into drive's `value` port
        (drive, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    lay_horizontal(g, &[grid, mv, vn, drive, out], panel_y, Some(vn));
    Some(out)
}

/// Monta a fileira BRANCA: `grid → move → drive(Y)` com o valor vindo de
/// `instance_field(Random) → map_range([0,1] → [-ARCH, ARCH])`. Devolve o sink.
fn white_row(g: &mut Graph, canvas_dy: f32, panel_y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let map = g.add_node("value.map_range");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 2.0); // Random: a white per-instance hash in [0,1)
    g.set_param(map, "out_lo", -ARCH); // match the noise row's amplitude
    g.set_param(map, "out_hi", ARCH);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, drive, 0),
        (grid, field, 0),
        (field, map, 0),
        (map, drive, 1),
        (drive, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    // The reference row — no mark (only the noise row is evaluated).
    lay_horizontal(g, &[grid, mv, field, map, drive, out], panel_y, None);
    Some(out)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_NOISE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_noise_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a onda COERENTE (value.noise, linha 1); de baixo a BRANCA (linha 2).
        let coherent = noise_row(g, 2.4, 80.0);
        let white = white_row(g, -2.4, 80.0 + ROW_GAP);
        gfx.motion.sinks.extend(coherent.into_iter().chain(white));
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

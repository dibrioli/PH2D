//! **A cena pronta para o smoke do `value.curve`** (`PH2D_VALUE_CURVE_SMOKE=1`, doc 68).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige**
//! outra coisa. Então a cena mostra a curva do jeito mais direto possível: **a
//! forma da curva vira o perfil ESPACIAL de uma fileira.**
//!
//! Duas fileiras de 24 instâncias, o MESMO grafo, só a curva difere:
//!
//! - `motion.grid → motion.move → motion.drive(Y)` é a geometria;
//! - `value.instance_field(Ramp) → value.curve → motion.drive(value)` é o valor:
//!   o campo de rampa dá `i/(N−1) ∈ [0,1]`, a curva o molda, o drive o soma em Y.
//!
//! **De cima (ARCH):** a curva é um TENT (`0 → 1 → 0`) — o meio da fileira sobe, as
//! pontas ficam no chão: um **arco**. É a forma que nenhum remap linear faz.
//! **De baixo (RAMP):** a MESMA `value.curve` **sem curva desenhada** = identidade =
//! `value.map_range` — uma **rampa** reta.
//!
//! O grafo é arrumado em duas **linhas horizontais retas** (`smoke_layout`), e o
//! `value.curve` marcado `>> EVALUATE <<` é o de cima (ARCH). Selecione-o → o painel
//! de params mostra o **editor de curva arrastável** (A1); arraste um ponto e o arco
//! muda de forma ao vivo. E o nó cozinha **100% na GPU** (o canal de LUT do A1-gpu).

use crate::smoke_layout::{lay_horizontal, ROW_GAP};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A altura do arco em unidades de mundo — `value.curve` mapeia a rampa `[0,1]` para
/// `[0, ARCH]`, então o pico do tent sobe `ARCH` e as pontas ficam em 0.
const ARCH: f32 = 4.0;
/// Um tent `0 → 1 → 0`: o meio da fileira ao pico, as pontas ao chão.
const TENT: &str = "c1 0:0:S 0.5:1:S 1:0:S";

/// Monta uma fileira `grid → move → drive(Y)` com o valor vindo de
/// `instance_field(Ramp) → value.curve[curve]`. `curve = None` deixa a `value.curve`
/// na identidade (uma rampa reta) e é a fileira de referência (sem marca). Devolve
/// o sink (`motion.output`).
fn row(g: &mut Graph, curve: Option<&str>, canvas_dy: f32, panel_y: f32) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let vc = g.add_node("value.curve");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(vc, "out_hi", ARCH); // map the ramp [0,1] -> [0, ARCH]
    if let Some(c) = curve {
        g.set_text_param(vc, "curve", c.to_string());
    }
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, drive, 0),   // geometry into drive's `in`
        (grid, field, 0), // instance_field reads the grid for its count
        (field, vc, 0),   // the ramp -> the curve
        (vc, drive, 1),   // the shaped value into drive's `value` port
        (drive, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    // Arrange as one straight line; mark the value.curve only on the drawn (ARCH) row.
    let hero = curve.is_some().then_some(vc);
    lay_horizontal(g, &[grid, mv, field, vc, drive, out], panel_y, hero);
    Some(out)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_CURVE_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_curve_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima ARQUEIA (tent), marcada (linha 1); de baixo a mesma sem curva (linha 2).
        let arch = row(g, Some(TENT), 2.4, 80.0);
        let ramp = row(g, None, -0.6, 80.0 + ROW_GAP);
        gfx.motion.sinks.extend(arch.into_iter().chain(ramp));
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

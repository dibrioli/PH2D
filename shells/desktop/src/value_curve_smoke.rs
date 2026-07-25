//! **A cena pronta para o smoke do `value.curve`** (`PH2D_VALUE_CURVE_SMOKE=1`, doc 68).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige** outra
//! coisa. Então a cena mostra a curva do jeito mais direto possível: **a forma da curva
//! vira o perfil ESPACIAL de uma fileira.**
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
//! `value.map_range` — uma **rampa** reta. A única diferença entre as duas fileiras é a
//! curva no text param, exatamente o que o editor arrastável do painel escreve.
//!
//! Selecione o nó `value.curve` de cima no grafo → o painel de params mostra o **editor
//! de curva arrastável** (A1); arraste um ponto e o arco muda de forma ao vivo. E o nó
//! cozinha **100% na GPU** (o canal de LUT do A1-gpu), sem cair pra CPU.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// A altura do arco em unidades de mundo — `value.curve` mapeia a rampa `[0,1]` para
/// `[0, ARCH]`, então o pico do tent sobe `ARCH` e as pontas ficam em 0.
const ARCH: f32 = 4.0;
/// Um tent `0 → 1 → 0`: o meio da fileira ao pico, as pontas ao chão.
const TENT: &str = "c1 0:0:S 0.5:1:S 1:0:S";

/// Monta uma fileira `grid → move → drive(Y)` com o valor vindo de
/// `instance_field(Ramp) → value.curve[curve]`. `curve = None` deixa a `value.curve` na
/// identidade (uma rampa reta). Devolve o sink (`motion.output`).
fn row(g: &mut Graph, curve: Option<&str>, y_off: f32, tag: &str) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let vc = g.add_node("value.curve");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", y_off);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(vc, "out_hi", ARCH); // map the ramp [0,1] -> [0, ARCH]
    if let Some(c) = curve {
        g.set_text_param(vc, "curve", c.to_string());
    }
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    for (n, (x, y)) in [
        (grid, (60.0, 200.0)),
        (mv, (240.0, 120.0)),
        (field, (240.0, 300.0)),
        (vc, (440.0, 300.0)),
        (drive, (640.0, 200.0)),
        (out, (840.0, 200.0)),
    ] {
        g.set_pos(n, Pos { x, y });
    }
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
    g.set_label(vc, tag);
    g.set_label(out, tag);
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
        // A de cima ARQUEIA (tent); a de baixo é a mesma coisa sem curva (rampa).
        let arch = row(g, Some(TENT), 2.4, "ARCH");
        let ramp = row(g, None, -0.6, "RAMP");
        gfx.motion.sinks.extend(arch.into_iter().chain(ramp));
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

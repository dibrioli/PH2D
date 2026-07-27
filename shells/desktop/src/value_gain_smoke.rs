//! **A cena pronta para o smoke do `value.gain`** (`PH2D_VALUE_GAIN_SMOKE=1`, doc 72).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige**
//! outra coisa. Então a cena mostra a assinatura do `value.gain` — a **curva-S de
//! contraste** — do jeito mais direto: uma rampa `[0,1]` vira o perfil ESPACIAL
//! de uma fileira, e o gain a curva.
//!
//! Duas fileiras de 24 instâncias, a MESMA rampa, só o gain difere:
//!
//! - **De cima (GAIN):** `instance_field(Ramp) → value.gain(Gain) → map_range → drive(Y)`.
//!   A rampa é empurrada para os extremos: as instâncias do meio se afastam do
//!   centro (mais contraste), formando um **degrau-S** — os cantos sobem/descem
//!   e o miolo estica. É a forma que nenhum remap linear faz.
//! - **De baixo (LINEAR):** a MESMA rampa **sem gain** = `value.map_range` = uma
//!   **rampa reta**. É a referência (sem marca).
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.gain` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Strength** (0 = neutro; positivo = mais efeito)
//! e **Mode** (Gain = contraste em S · Bias = empurra para uma ponta). Arraste o
//! Strength e veja o S ficar mais forte / inverter; troque para Bias e a rampa
//! curva para cima ou para baixo. O nó cozinha **100% na GPU** (Schlick
//! transcendental-free, paridade de dispositivo).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A amplitude do deslocamento em Y — o `value.gain` produz `[0,1]`, e o
/// `value.map_range` o leva para `[0, ARCH]`, então o pico sobe `ARCH`.
const ARCH: f32 = 4.0;
/// A força do gain na fileira marcada — um S bem visível (positivo = mais contraste).
const STRENGTH: f32 = 0.7;

/// Monta uma fileira `grid → move → drive(Y)` cujo valor vem de um
/// `instance_field(Ramp)`, opcionalmente passado por `value.gain`, e sempre
/// escalado por um `value.map_range([0,1] → [0, ARCH])`. `canvas_dy` desloca a
/// fileira na tela. Devolve `(sink, hero)`: o sink e o `value.gain` a avaliar (só
/// a fileira GAIN tem um).
fn row(g: &mut Graph, gained: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
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
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(map, "out_lo", 0.0); // the shaped [0,1] -> [0, ARCH] for Y
    g.set_param(map, "out_hi", ARCH);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional S-curve: shape the ramp before it is scaled.
    let gain = gained.then(|| {
        let vg = g.add_node("value.gain");
        g.set_param(vg, "mode", 0.0); // Gain (the contrast S-curve)
        g.set_param(vg, "strength", STRENGTH);
        vg
    });
    // What feeds the map: the gain when present, else the raw ramp.
    let shaped_src = gain.unwrap_or(field);

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),       // geometry into drive's `in`
        (grid, field, 0),     // instance_field reads the grid for count
        (shaped_src, map, 0), // the (maybe gained) ramp into the map
        (map, drive, 1),      // the scaled value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(vg) = gain {
        edges.push((field, vg, 0)); // the ramp into the gain
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.gain is the node under evaluation (only the gained row has one).
    Some((out, gain))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_GAIN_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_gain_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima a curva-S de contraste (value.gain, marcada); de baixo a rampa reta.
        let gained = row(g, true, 2.4);
        let linear = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [gained, linear].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

//! **A cena pronta para o smoke do `value.step`** (`PH2D_VALUE_STEP_SMOKE=1`, doc 73).
//!
//! Um nó de VALOR não se vê — ele produz um número por instância que **dirige**
//! outra coisa. O `value.step` produz uma **máscara** (compara o campo a um
//! limiar), então a cena a mostra do jeito mais direto: uma rampa `[0,1]` vira o
//! perfil ESPACIAL de uma fileira, e o gate a corta.
//!
//! Duas fileiras de 24 instâncias, a MESMA rampa, só o gate difere:
//!
//! - **De cima (STEP):** `instance_field(Ramp) → value.step(Hard) → map_range → drive(Y)`.
//!   O gate no `0.5` colapsa a rampa em `{0,1}`: as instâncias caem em DOIS
//!   patamares — as da metade baixa no chão, as da metade alta no topo — um
//!   **penhasco** onde a rampa cruza o limiar. É a forma que nenhum remap faz.
//! - **De baixo (LINEAR):** a MESMA rampa **sem gate** = `value.map_range` = uma
//!   **rampa reta**. É a referência (sem marca).
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.step` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Threshold** (onde corta), **Width** (a largura
//! da banda suave) e **Mode** (Hard = penhasco · Smooth = banda smoothstep).
//! Arraste o Threshold e veja o degrau deslizar; troque para Smooth e suba o Width
//! e o penhasco vira uma rampa-S macia. O nó cozinha **100% na GPU** (comparação +
//! Hermite, transcendental-free, paridade de dispositivo).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A amplitude do deslocamento em Y — o `value.step` produz `[0,1]`, e o
/// `value.map_range` o leva para `[0, ARCH]`, então o topo do degrau sobe `ARCH`.
const ARCH: f32 = 4.0;
/// Onde o gate corta a rampa `[0,1]` na fileira marcada — no meio, um penhasco central.
const THRESHOLD: f32 = 0.5;

/// Monta uma fileira `grid → move → drive(Y)` cujo valor vem de um
/// `instance_field(Ramp)`, opcionalmente passado por `value.step`, e sempre
/// escalado por um `value.map_range([0,1] → [0, ARCH])`. `canvas_dy` desloca a
/// fileira na tela. Devolve `(sink, hero)`: o sink e o `value.step` a avaliar (só
/// a fileira STEP tem um).
fn row(g: &mut Graph, gated: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
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
    g.set_param(map, "out_lo", 0.0); // the mask [0,1] -> [0, ARCH] for Y
    g.set_param(map, "out_hi", ARCH);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional gate: threshold the ramp before it is scaled.
    let step = gated.then(|| {
        let vs = g.add_node("value.step");
        g.set_param(vs, "mode", 0.0); // Hard (the cliff)
        g.set_param(vs, "threshold", THRESHOLD);
        vs
    });
    // What feeds the map: the gate when present, else the raw ramp.
    let shaped_src = step.unwrap_or(field);

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),       // geometry into drive's `in`
        (grid, field, 0),     // instance_field reads the grid for count
        (shaped_src, map, 0), // the (maybe gated) ramp into the map
        (map, drive, 1),      // the scaled value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(vs) = step {
        edges.push((field, vs, 0)); // the ramp into the gate
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.step is the node under evaluation (only the gated row has one).
    Some((out, step))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_STEP_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_step_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o penhasco (value.step, marcado); de baixo a rampa reta.
        let gated = row(g, true, 2.4);
        let linear = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [gated, linear].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

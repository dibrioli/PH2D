//! **A cena pronta para o smoke do `value.unary`** (`PH2D_VALUE_UNARY_SMOKE=1`, doc 75).
//!
//! O `value.unary` aplica UMA função a cada elemento (o contraponto de um argumento
//! ao `value.math` binário). A cena mostra a assinatura mais legível — o **Abs**,
//! que dobra a metade negativa de um sinal para cima — do jeito mais direto: uma
//! rampa BIPOLAR `[-2, 2]` vira o perfil ESPACIAL de uma fileira, e o abs a dobra.
//!
//! Duas fileiras de 24 instâncias, o MESMO driver bipolar, só a função difere:
//!
//! - **De cima (UNARY, Abs):** `instance_field(Ramp) → map_range([-2,2]) → value.unary(Abs) → drive(Y)`.
//!   O driver vai de `-2` a `+2`; o `abs` o dobra num **V**: alto nas pontas, caindo
//!   a `0` no meio (onde o driver cruza zero). É a forma que nenhum remap linear faz.
//! - **De baixo (RAW):** o MESMO driver `[-2, 2]` **direto** em Y, sem função — uma
//!   **rampa reta** que atravessa o zero (a referência, sem marca).
//!
//! O grafo inteiro é **arrumado pelo auto-layout ciente de subgrupos**
//! (`smoke_layout`), e o `value.unary` marcado `>> EVALUATE <<` é o de cima.
//! Selecione-o → o painel mostra **Op** (Abs · Negate · Sign · Floor · Fract ·
//! Square · Sqrt · Reciprocal). Troque para **Square** (o V vira uma parábola
//! íngreme), **Sign** (dois patamares planos `±1`), **Floor** (degraus inteiros) ou
//! **Fract** (dentes de serra repetidos). O nó cozinha **100% na GPU** (algébrico,
//! transcendental-free, paridade de dispositivo).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A amplitude bipolar do driver: a rampa vai de `-BIPOLAR` a `+BIPOLAR`, então o
/// `abs` a dobra num V de altura `BIPOLAR`.
const BIPOLAR: f32 = 2.0;

/// Monta uma fileira `grid → move → drive(Y)` cujo valor vem de um
/// `instance_field(Ramp)` esticado para `[-BIPOLAR, BIPOLAR]` por um
/// `value.map_range`, opcionalmente passado por `value.unary(Abs)`. `canvas_dy`
/// desloca a fileira na tela. Devolve `(sink, hero)`: o sink e o `value.unary` a
/// avaliar (só a fileira de cima tem um).
fn row(g: &mut Graph, apply_op: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let field = g.add_node("value.instance_field");
    let bip = g.add_node("value.map_range");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    g.set_param(field, "mode", 1.0); // Ramp: i/(N-1) in [0,1]
    g.set_param(bip, "out_lo", -BIPOLAR); // the bipolar driver [-2, 2]
    g.set_param(bip, "out_hi", BIPOLAR);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional op: fold the bipolar driver before it drives Y.
    let unary = apply_op.then(|| {
        let vu = g.add_node("value.unary");
        g.set_param(vu, "op", 0.0); // Abs (the fold)
        vu
    });
    // What feeds the drive: the op when present, else the raw bipolar driver.
    let value_src = unary.unwrap_or(bip);

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),        // geometry into drive's `in`
        (grid, field, 0),      // instance_field reads the grid for count
        (field, bip, 0),       // the ramp into the bipolar map
        (value_src, drive, 1), // the (maybe folded) value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(vu) = unary {
        edges.push((bip, vu, 0)); // the bipolar driver into the op
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }

    // The value.unary is the node under evaluation (only the top row has one).
    Some((out, unary))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_UNARY_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_unary_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o V dobrado (value.unary Abs, marcado); de baixo a rampa reta.
        let folded = row(g, true, 3.0);
        let raw = row(g, false, -3.0);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [folded, raw].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

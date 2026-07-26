//! **A cena pronta para o smoke do `value.mix`** (`PH2D_VALUE_MIX_SMOKE=1`, doc 70).
//!
//! Um crossfader não se vê sozinho — ele **mistura dois drivers**. Então a cena
//! liga um `value.lfo` (uma onda LIMPA) e um `value.noise` (orgânico) nas entradas
//! `a` e `b` de um `value.mix`, e usa a saída para deslocar Y de uma fileira.
//!
//! Duas fileiras de 24 instâncias, o MESMO par (onda, ruído) em `a`/`b`, só o
//! FATOR difere:
//!
//! - **De cima (DRIVEN):** o fator `t` é um `value.lfo` triangular lento varrendo
//!   `[0,1]` — a fileira **transita** entre a onda limpa (t=0) e o ruído (t=1) e de
//!   volta, ao vivo. É o diferencial: o crossfade **dirigido por um valor**.
//! - **De baixo (FACTOR):** a porta `t` fica DESCONECTADA, e o `factor` PARAM (0,5)
//!   é a mistura constante — uma onda permanentemente meio-ruidosa. É o fallback do
//!   knob (o socket Factor do Blender, que um fio sobrepõe).
//!
//! O grafo inteiro é **arrumado pelo auto-layout em camadas** (`smoke_layout` →
//! `ph2d_nodegraph::layout`), sem sobreposições, e o `value.mix` marcado
//! `>> EVALUATE <<` é o de baixo (FACTOR). Selecione-o → o painel mostra
//! **Factor** (a mistura constante) e **Clamp**; arraste o Factor de 0 (só a
//! onda) a 1 (só o ruído). Cozinha 100% na GPU — a escolha porta-sobrepõe-param
//! sai do `HAS_t_v` do kernel, sem CPU.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A amplitude do deslocamento em Y de ambos os drivers (onda e ruído), para o
/// crossfade misturar duas coisas da MESMA escala.
const AMP: f32 = 3.0;

/// Monta uma fileira `grid → move → drive(Y)` cujo valor vem de
/// `mix(a = lfo, b = noise, t)`. `driven` liga um `value.lfo` triangular lento em
/// `t` (crossfade animado); senão a porta `t` fica solta e o `factor` param manda —
/// e essa é a fileira MARCADA para avaliação. Devolve `(sink, hero)`.
fn row(g: &mut Graph, driven: bool, canvas_dy: f32) -> Option<(NodeId, Option<NodeId>)> {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let lfo = g.add_node("value.lfo");
    let noise = g.add_node("value.noise");
    let mix = g.add_node("value.mix");
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(mv, "dy", canvas_dy);
    // a: a clean travelling sine across the row.
    g.set_param(lfo, "wave", 0.0); // sine
    g.set_param(lfo, "period", 2.0);
    g.set_param(lfo, "amplitude", AMP);
    g.set_param(lfo, "phase_stagger", 0.3);
    // b: organic coherent noise, same amplitude.
    g.set_param(noise, "frequency", 0.28);
    g.set_param(noise, "speed", 1.0);
    g.set_param(noise, "octaves", 2.0);
    g.set_param(noise, "amplitude", AMP);
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add

    // The optional `t` driver: a slow triangle LFO sweeping [0,1] (length-1, so the
    // whole row crossfades together). Unconnected → the `factor` param (0.5) rules.
    let t_lfo = driven.then(|| {
        let t = g.add_node("value.lfo");
        g.set_param(t, "wave", 1.0); // triangle
        g.set_param(t, "period", 6.0); // slow sweep
        g.set_param(t, "amplitude", 0.5);
        g.set_param(t, "offset", 0.5); // -> [0, 1]
        t
    });

    let mut edges = vec![
        (grid, mv, 0u16),
        (mv, drive, 0),   // geometry into drive's `in`
        (grid, lfo, 0),   // lfo reads the grid for count
        (grid, noise, 0), // noise reads the grid for count
        (lfo, mix, 0),    // a = the clean wave
        (noise, mix, 1),  // b = the noise
        (mix, drive, 1),  // the crossfaded value into drive's `value` port
        (drive, out, 0),
    ];
    if let Some(t) = t_lfo {
        edges.push((t, mix, 2)); // t = the slow sweep (only when driven)
    }
    for (from, to, port) in edges {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    if !driven {
        g.set_param(mix, "factor", 0.5); // the constant half-blend fallback
    }

    // The FACTOR row's mix (the one you drag) is the node under evaluation.
    Some((out, (!driven).then_some(mix)))
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_VALUE_MIX_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn value_mix_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let g = &mut gfx.motion.doc.graph;
        // De cima o crossfade DIRIGIDO (referência); de baixo o FACTOR fixo, marcado.
        let driven = row(g, true, 2.4);
        let factor = row(g, false, -2.4);
        let mut heroes = Vec::new();
        let mut sinks = Vec::new();
        for (sink, hero) in [driven, factor].into_iter().flatten() {
            sinks.push(sink);
            heroes.extend(hero);
        }
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

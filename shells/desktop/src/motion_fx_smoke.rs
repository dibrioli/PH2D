//! **The scene ready for the `motion.fx` glow smoke** (`PH2D_MOTION_FX_SMOKE=1`,
//! doc 67).
//!
//! ## What it shows
//!
//! One row of sparks whose colour ramps **white → hot orange** across the set
//! (`motion.tint` in Gradient mode). White is `1.0` — LDR, right at the bloom
//! threshold, so it barely glows. The hot end is `(6, 4, 2)` — genuinely HDR, so
//! it glows hard. The halo **grows with brightness left-to-right**, which is the
//! defining property of an HDR bloom and the whole reason this is Motion's own
//! `Rgba16Float` pass and not the 8-bit compositor (doc 66): an 8-bit
//! round-trip would clip every spark to white and the ramp would vanish.
//!
//! ```text
//!   grid(1×9) → tint(Gradient white→hot) → scale → fx.glow → output
//! ```
//!
//! The `fx.glow` node IS the effect (doc 67): the shell reads its params
//! (`intensity`, `threshold`, `radius`) and runs the pass. Select it in the
//! Motion params panel to drag the glow live; delete it and the sparks stay
//! exactly as bright, just without the halo.
//!
//! The glow is **additive** and computed from the Motion pixels ALONE — the
//! sprites/Flip/Vector in the scene are never touched (blast radius zero). Toggle
//! the effect off in the panel and the sparks stay exactly as bright, just
//! without the halo: the neutral point is byte-identical.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// The hot (HDR) end of the ramp — well above 1.0 so the bright-pass has real
/// excess to bloom. Warm so the glow reads as light, not a colour cast.
const HOT: [f32; 3] = [6.0, 4.0, 2.0];
/// Instances across the row. Enough that the ramp is legible.
const COLS: f32 = 9.0;

/// The spark chain: `grid → tint(Gradient) → scale → fx.glow → output`. Returns
/// the sink.
fn sparks(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let tint = g.add_node("motion.tint");
    let scale = g.add_node("motion.scale");
    let glow = g.add_node("fx.glow");
    let out = g.add_node("motion.output");

    let chain = [grid, tint, scale, glow, out];
    for (i, n) in chain.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * 190.0,
                y: -360.0,
            },
        );
    }
    for w in chain.windows(2) {
        g.connect(Edge {
            from: (w[0], 0),
            to: (w[1], 0),
            delayed: false,
        })
        .ok()?;
    }

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", COLS);
    g.set_param(grid, "gap_x", 1.1);
    // Gradient (mode 1): Start = white (LDR, barely blooms), End = hot (HDR).
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "r", 1.0);
    g.set_param(tint, "g", 1.0);
    g.set_param(tint, "b", 1.0);
    g.set_param(tint, "r2", HOT[0]);
    g.set_param(tint, "g2", HOT[1]);
    g.set_param(tint, "b2", HOT[2]);
    g.set_label(tint, "White \u{2192} Hot");
    // Solid blocks — a small spark has little light to spread, so the halo comes
    // out thin; a fuller block gives the glow real energy.
    g.set_param(scale, "amount", 0.28);
    // The glow node the shell reads for the pass. The COD mip chain accumulates
    // energy across levels, so a moderate intensity already reads as a bright,
    // ROUND halo (no longer the square of a single-scale box blur).
    g.set_param(glow, "threshold", 1.0);
    g.set_param(glow, "knee", 0.6);
    g.set_param(glow, "intensity", 1.0);
    g.set_param(glow, "radius", 1.0);
    g.set_label(out, "GLOW");
    Some(out)
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_MOTION_FX_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    pub(crate) fn motion_fx_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let out = sparks(&mut gfx.motion.doc.graph);
        gfx.motion.sinks.extend(out);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    }
}

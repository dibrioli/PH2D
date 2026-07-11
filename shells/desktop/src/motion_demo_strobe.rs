//! The M1 stream demo — the **default Motion document**, the first with **branch-and-
//! merge** topology (until now every graph was one linear chain): on the LEFT a grid and
//! a ring are **concatenated** into one cloud (`motion.combine`); on the RIGHT a grid is
//! **blended** into a circle (`motion.mixer` in Blend mode, its `blend` swept by a sine
//! `value.lfo` — a square morphing to a ring and back). Each scene is a Y: two sources
//! converging on the new node. Two independent scenes (each its own `motion.output` sink),
//! kept small so each new node reads on its own. A `#[path]` sibling of `motion_state`,
//! kept out for the LOC cap.
//!
//! ```text
//! LEFT  (combine): grid ┐                          RIGHT (mixer): grid ┐
//!         radial(spin) ┴→ combine → tint → move(−6) → out    radial ┴→ mixer(Blend) → tint → move(+6) → out
//!                                                                      lfo(sine) → blend
//! ```
//!
//! - **combine** (`motion.combine`, doc 30): a 10×10 grid and a spinning 40-point ring
//!   stack into one 140-point stream (concatenation — the Merge/Join).
//! - **mixer** (`motion.mixer`, doc 30): a 64-point grid and a 64-point circle, blended
//!   element-wise; the `blend` `value.lfo` morphs the square into the ring and back.
//!
//! See docs/Motion Nodes/30 (combine + mixer). The whole value/pulse vocabulary + the
//! other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const COMBINE_ROW: f32 = 0.0;
const MIXER_ROW: f32 = 340.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the combine
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let combine = build_combine_scene(g)?;
    let mixer = build_mixer_scene(g)?;
    Some(vec![combine, mixer])
}

/// Connect `from` → `to` on the given ports, an immediate (non-delayed) edge.
fn wire(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16)) -> Option<()> {
    g.connect(Edge {
        from,
        to,
        delayed: false,
    })
    .ok()
}

/// LEFT: a grid and a spinning ring concatenated. Returns its Output node.
fn build_combine_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let ring = g.add_node("motion.distribute_radial");
    let combine = g.add_node("motion.combine");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    g.set_pos(
        grid,
        Pos {
            x: 0.0,
            y: COMBINE_ROW,
        },
    );
    g.set_pos(
        ring,
        Pos {
            x: 0.0,
            y: COMBINE_ROW + 120.0,
        },
    );
    for (n, col) in [(combine, 1.0), (tint, 2.0), (mv, 3.0), (output, 4.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: COMBINE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: COMBINE_ROW + 240.0,
        },
    );

    wire(g, (grid, 0), (combine, 0))?;
    wire(g, (ring, 0), (combine, 1))?;
    wire(g, (combine, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (ring, 0))?; // → ring spin

    // A 10×10 grid + a 40-point ring around it → 140 dots, amber, left.
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 10.0);
    g.set_param(grid, "gap_x", 0.45);
    g.set_param(grid, "gap_y", 0.45);
    g.set_param(ring, "count", 40.0);
    g.set_param(ring, "rings", 1.0);
    g.set_param(ring, "radius", 3.2);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    // lfo → ring spin: a slow sine, ±180°.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 180.0);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}

/// RIGHT: a grid blended into a circle. Returns its Output node.
fn build_mixer_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let circle = g.add_node("motion.distribute_radial");
    let mixer = g.add_node("motion.mixer");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    g.set_pos(
        grid,
        Pos {
            x: 0.0,
            y: MIXER_ROW,
        },
    );
    g.set_pos(
        circle,
        Pos {
            x: 0.0,
            y: MIXER_ROW + 120.0,
        },
    );
    for (n, col) in [(mixer, 1.0), (tint, 2.0), (mv, 3.0), (output, 4.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: MIXER_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: MIXER_ROW + 240.0,
        },
    );

    wire(g, (grid, 0), (mixer, 0))?;
    wire(g, (circle, 0), (mixer, 1))?;
    wire(g, (mixer, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (mixer, 4))?; // → blend weight

    // An 8×8 grid (64) blended toward a 64-point circle → a square↔ring morph, cyan, right.
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    g.set_param(grid, "gap_x", 0.55);
    g.set_param(grid, "gap_y", 0.55);
    g.set_param(circle, "count", 64.0);
    g.set_param(circle, "rings", 1.0);
    g.set_param(circle, "radius", 2.4);
    g.set_param(mixer, "mode", 2.0); // Blend (in0 → in1)
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    // lfo → blend: a sine about 0.5, ±0.5 → blend ∈ [0, 1] (grid ↔ circle).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 5.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

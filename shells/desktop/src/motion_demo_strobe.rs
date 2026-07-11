//! The M1 colour demo — the **default Motion document**: on the LEFT a radial sunburst
//! coloured by a **rainbow gradient** (`motion.color_ramp`) that spins; on the RIGHT a
//! grid coloured by a **cycling palette** (`motion.color_array`) whose slots march across
//! it. The two colour nodes that were missing — until now `motion.tint` (a single solid)
//! was the only colour source. Two independent scenes (each its own `motion.output` sink
//! — the bridge composes several into one draw), kept small so each new node reads on its
//! own. A `#[path]` sibling of `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (rainbow):  distribute_radial → color_ramp(Rainbow) → move(−6) → output  lfo(sine) → spin
//! RIGHT (palette):  grid → color_array(4 colours) → move(+6) → output            lfo(saw)  → offset
//! ```
//!
//! - **color_ramp** (`motion.color_ramp`, doc 29): the 60 radial points are coloured by
//!   their normalised index along a rainbow ramp → concentric spectral rings; the sunburst
//!   spins via the `spin` `value.lfo`.
//! - **color_array** (`motion.color_array`, doc 29): a 10×10 grid takes a 4-colour palette
//!   by `index mod 4`; the `offset` saw `value.lfo` marches the palette across the grid.
//!
//! See docs/Motion Nodes/29 (color_ramp + color_array). The whole value/pulse vocabulary +
//! the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 200.0;
const RAINBOW_ROW: f32 = 0.0;
const PALETTE_ROW: f32 = 320.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the rainbow
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let rainbow = build_rainbow_scene(g)?;
    let palette = build_palette_scene(g)?;
    Some(vec![rainbow, palette])
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

/// LEFT: a rainbow radial sunburst that spins. Returns its Output node.
fn build_rainbow_scene(g: &mut Graph) -> Option<NodeId> {
    let radial = g.add_node("motion.distribute_radial");
    let ramp = g.add_node("motion.color_ramp");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(radial, 0.0), (ramp, 1.0), (mv, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: RAINBOW_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: RAINBOW_ROW + 160.0,
        },
    );

    wire(g, (radial, 0), (ramp, 0))?;
    wire(g, (ramp, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (radial, 0))?; // → spin

    // 60 points over 4 rings, coloured by index along the Rainbow ramp, spinning, left.
    g.set_param(radial, "count", 60.0);
    g.set_param(radial, "rings", 4.0);
    g.set_param(radial, "radius", 3.0);
    g.set_param(radial, "inner", 0.6);
    g.set_param(ramp, "preset", 0.0); // Rainbow
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    // lfo → spin: a slow sine, ±180° → the sunburst turns.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 180.0);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}

/// RIGHT: a grid with a marching 4-colour palette. Returns its Output node.
fn build_palette_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let array = g.add_node("motion.color_array");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(grid, 0.0), (array, 1.0), (mv, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: PALETTE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: PALETTE_ROW + 160.0,
        },
    );

    wire(g, (grid, 0), (array, 0))?;
    wire(g, (array, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (array, 1))?; // → offset

    // A 10×10 grid with the default red/green/blue/yellow palette, marching, right.
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 10.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    g.set_param(array, "colors", 4.0);
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    // lfo → offset: a saw ramping 0→4 (amp 2 about 2) so the palette marches a full cycle.
    g.set_param(lfo, "wave", 3.0); // Saw
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 2.0);
    g.set_param(lfo, "offset", 2.0);
    Some(output)
}

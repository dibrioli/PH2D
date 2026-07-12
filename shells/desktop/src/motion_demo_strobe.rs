//! The M1 adapter demo — the **default Motion document**, showing the value↔geometry↔
//! colour bridges: on the LEFT a **Lissajous** plotted from two staggered LFOs
//! (`motion.make_point` turns value fields into positions); on the RIGHT a rainbow grid
//! **recoloured by its own brightness** (`motion.luminance` reads the tint back into a
//! value that drives a second ramp). Two independent scenes (each its own `motion.output`
//! sink), kept small so each new node reads on its own. A `#[path]` sibling of
//! `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (make_point): grid → lfoX(stagger) ┐
//!                     grid → lfoY(stagger) ┴→ make_point → tint → move(−6) → output
//! RIGHT (luminance):  grid → color_ramp(Rainbow) → luminance → color_ramp(t, Heat) → move(+6) → out
//! ```
//!
//! - **make_point** (`motion.make_point`, doc 31): the grid fixes the count (64); two
//!   `value.lfo`s with different `phase_stagger` give per-instance x and y → a Lissajous
//!   that the playhead animates.
//! - **luminance** (`motion.luminance`, doc 31): the rainbow's per-instance brightness
//!   becomes a `v` field that indexes a Heat ramp — colour read back into a value.
//!
//! See docs/Motion Nodes/31 (make_point + luminance). The whole value/pulse vocabulary +
//! the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const LISSAJOUS_ROW: f32 = 0.0;
const LUMA_ROW: f32 = 320.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the make_point
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let lissajous = build_lissajous_scene(g)?;
    let luma = build_luminance_scene(g)?;
    Some(vec![lissajous, luma])
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

/// Configure a `value.lfo`: sine, `amp`, period 6 s, per-instance `stagger` cycles.
fn setup_lfo(g: &mut Graph, lfo: NodeId, amp: f32, phase: f32, stagger: f32) {
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", amp);
    g.set_param(lfo, "phase", phase);
    g.set_param(lfo, "phase_stagger", stagger);
}

/// LEFT: a Lissajous plotted by make_point from two staggered LFOs. Returns its Output.
fn build_lissajous_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let lfo_x = g.add_node("value.lfo");
    let lfo_y = g.add_node("value.lfo");
    let point = g.add_node("motion.make_point");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    g.set_pos(
        grid,
        Pos {
            x: 0.0,
            y: LISSAJOUS_ROW,
        },
    );
    g.set_pos(
        lfo_x,
        Pos {
            x: COL_W,
            y: LISSAJOUS_ROW - 120.0,
        },
    );
    g.set_pos(
        lfo_y,
        Pos {
            x: COL_W,
            y: LISSAJOUS_ROW + 120.0,
        },
    );
    for (n, col) in [(point, 2.0), (tint, 3.0), (mv, 4.0), (output, 5.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: LISSAJOUS_ROW,
            },
        );
    }

    wire(g, (grid, 0), (lfo_x, 0))?; // grid fixes the count for the LFOs
    wire(g, (grid, 0), (lfo_y, 0))?;
    wire(g, (grid, 0), (point, 0))?; // and for make_point
    wire(g, (lfo_x, 0), (point, 1))?; // → x
    wire(g, (lfo_y, 0), (point, 2))?; // → y
    wire(g, (point, 0), (tint, 0))?; // the plotted points → tint → move → output
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    // An 8×8 grid (64 instances) drives the LFO count; x runs 3 cycles across the set,
    // y runs 2 → a 3:2 Lissajous, amplitude 4, on the left.
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    setup_lfo(g, lfo_x, 4.0, 0.0, 3.0 / 64.0);
    setup_lfo(g, lfo_y, 4.0, 0.25, 2.0 / 64.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

/// RIGHT: a rainbow grid recoloured by its own luminance through a Heat ramp. Returns
/// its Output.
fn build_luminance_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let rainbow = g.add_node("motion.color_ramp");
    let luma = g.add_node("motion.luminance");
    let heat = g.add_node("motion.color_ramp");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    for (n, col) in [
        (grid, 0.0),
        (rainbow, 1.0),
        (luma, 2.0),
        (heat, 3.0),
        (mv, 4.0),
        (output, 5.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: LUMA_ROW,
            },
        );
    }

    wire(g, (grid, 0), (rainbow, 0))?;
    wire(g, (rainbow, 0), (luma, 0))?; // luminance reads the rainbow's tint → a `v` field
    wire(g, (rainbow, 0), (heat, 0))?; // the rainbow stream carries the geometry to Heat
    wire(g, (luma, 0), (heat, 1))?; // the luma value drives Heat's `t`
    wire(g, (heat, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    // A 10×10 grid coloured by a rainbow (by index), its brightness read by luminance and
    // fed as the `t` of a Heat ramp → recoloured by luma, on the right.
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 10.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    g.set_param(rainbow, "preset", 0.0); // Rainbow
    g.set_param(heat, "preset", 1.0); // Heat (indexed by the luma `t`)
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

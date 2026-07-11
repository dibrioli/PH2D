//! The M3 symmetry/packing demo — the **default Motion document**: on the LEFT a
//! spinning **mandala** — a small Fibonacci spiral folded into 8-fold symmetry
//! (`motion.kaleidoscope`) — and on the RIGHT a tight grid that **packs apart** into a
//! breathing circle-packing (`motion.collide`, the Push-Apart relaxation, its `radius`
//! animated so the cloud inflates and settles). Two independent scenes (each its own
//! `motion.output` sink — the bridge composes several into one draw), kept small so each
//! new node reads on its own. A `#[path]` sibling of `motion_state`, kept out for the
//! LOC cap.
//!
//! ```text
//! LEFT  (mandala): fibonacci → kaleidoscope → move(−6) → tint(amber) → output   lfo → spin
//! RIGHT (packing): grid → collide → move(+6) → tint(cyan) → output              lfo → spread
//! ```
//!
//! - **kaleidoscope** (`motion.kaleidoscope`, doc 26): the 6-seed spiral is replicated
//!   into 8 mirrored slices about the origin (48 dots); the `spin` `value.lfo` turns the
//!   whole mandala.
//! - **collide** (`motion.collide`, doc 26): an 8×8 grid whose cells start overlapping is
//!   pushed apart into a packing; the `spread` `value.lfo` breathes the disc radius, so
//!   the cloud expands and contracts — the Push-Apart effector in the flesh.
//!
//! See docs/Motion Nodes/26 (kaleidoscope + collide). The whole value/pulse vocabulary +
//! the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
const MANDALA_ROW: f32 = 0.0;
const PACKING_ROW: f32 = 320.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the mandala
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let mandala_out = build_mandala_scene(g)?;
    let packing_out = build_packing_scene(g)?;
    Some(vec![mandala_out, packing_out])
}

/// Connect `from` → `to` (output/input port 0), an immediate (non-delayed) edge.
fn wire(g: &mut Graph, from: NodeId, to: NodeId) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .ok()
}

/// LEFT: a spinning mandala — a Fibonacci spiral folded 8-fold. Returns its Output node.
fn build_mandala_scene(g: &mut Graph) -> Option<NodeId> {
    let fib = g.add_node("motion.fibonacci");
    let kaleido = g.add_node("motion.kaleidoscope");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [
        (fib, 0.0),
        (kaleido, 1.0),
        (mv, 2.0),
        (tint, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: MANDALA_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: COL_W,
            y: MANDALA_ROW + 160.0,
        },
    );

    wire(g, fib, kaleido)?;
    wire(g, kaleido, mv)?;
    wire(g, mv, tint)?;
    wire(g, tint, output)?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (kaleido, 1),
        delayed: false,
    })
    .ok()?; // → spin

    // A small 6-seed spiral (outer radius ≈ 0.9·√5 ≈ 2), folded into 8 mirrored slices
    // about the origin → a 48-dot mandala, placed on the left half.
    g.set_param(fib, "count", 6.0);
    g.set_param(fib, "spacing", 0.9);
    g.set_param(kaleido, "segments", 8.0);
    g.set_param(kaleido, "reflect", 1.0); // Mirrored (Dₙ) — the kaleidoscope look
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // lfo → spin: a slow (6 s) sine, ±180° → the mandala turns one way and back.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 180.0);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}

/// RIGHT: a tight grid that packs apart into a breathing circle-packing. Returns its
/// Output node.
fn build_packing_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let collide = g.add_node("motion.collide");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [
        (grid, 0.0),
        (collide, 1.0),
        (mv, 2.0),
        (tint, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: PACKING_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: COL_W,
            y: PACKING_ROW + 160.0,
        },
    );

    wire(g, grid, collide)?;
    wire(g, collide, mv)?;
    wire(g, mv, tint)?;
    wire(g, tint, output)?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (collide, 1),
        delayed: false,
    })
    .ok()?; // → spread

    // An 8×8 grid whose 0.45 spacing is *tighter* than the 0.6 collision diameter, so
    // every cell overlaps its neighbours → collide packs them apart (64 dots), on the
    // right half. The `spread` lfo breathes the radius so the packing inflates/settles.
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    g.set_param(grid, "gap_x", 0.45);
    g.set_param(grid, "gap_y", 0.45);
    g.set_param(collide, "radius", 0.3);
    g.set_param(collide, "iterations", 10.0);
    g.set_param(collide, "strength", 1.0);
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    // lfo → spread: a 4 s sine about 0.9, ±0.6 → spread ∈ [0.3, 1.5] (radius breathes,
    // the packing expands and contracts).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.6);
    g.set_param(lfo, "offset", 0.9);
    Some(output)
}

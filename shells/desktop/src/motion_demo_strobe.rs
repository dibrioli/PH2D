//! The M3 structure demo — the **default Motion document**: two grids revealed by a
//! shared `value.lfo` sweeping a **cull** fraction, differing only in how a **sort**
//! orders the reveal. On the LEFT `motion.sort` orders **radially**, so the cull wipes
//! the grid in from the centre out; on the RIGHT it orders **randomly**, so the same
//! cull dissolves it in scattered specks. One `value.lfo` fans out to both culls (a
//! value driving two scenes at once). Two independent scenes (each its own
//! `motion.output` sink — the bridge composes several into one draw), kept small so each
//! new node reads on its own. A `#[path]` sibling of `motion_state`, kept out for the
//! LOC cap.
//!
//! ```text
//! LEFT  (radial wipe):     grid → sort(Radial) → cull → tint(amber) → move(−6) → output
//! RIGHT (random dissolve): grid → sort(Random) → cull → tint(cyan)  → move(+6) → output
//! shared value.lfo → both culls' amount (one clock, two reveals)
//! ```
//!
//! - **sort** (`motion.sort`, doc 27): reorders the stream by a key. On its own it looks
//!   like nothing changed — it sets the *order* the reveal happens in (Radial vs Random).
//! - **cull** (`motion.cull`, doc 27): keeps the first `amount·n` elements; the `amount`
//!   `value.lfo` sweeps, so the grid fills and empties — a reveal whose *shape* is the
//!   upstream sort (a centre-out wipe on the left, a dissolve on the right).
//!
//! See docs/Motion Nodes/27 (sort + cull). The whole value/pulse vocabulary + the other
//! M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 200.0;
const LEFT_ROW: f32 = 0.0;
const RIGHT_ROW: f32 = 320.0;

/// Author both scenes into `g`, sharing one `value.lfo` that drives both culls; returns
/// their Output nodes (the sinks), the left scene's first so the sink order is stable.
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    // The shared reveal clock: a 5 s sine about 0.55, ±0.4 → amount ∈ [0.15, 0.95] (the
    // grids fill and empty without ever fully clearing or filling).
    let lfo = g.add_node("value.lfo");
    g.set_pos(
        lfo,
        Pos {
            x: 2.0 * COL_W,
            y: 160.0,
        },
    );
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 5.0);
    g.set_param(lfo, "amplitude", 0.4);
    g.set_param(lfo, "offset", 0.55);

    let left = build_reveal_scene(g, lfo, LEFT_ROW, 0.0, -6.0, [0.95, 0.70, 0.20])?;
    let right = build_reveal_scene(g, lfo, RIGHT_ROW, 3.0, 6.0, [0.25, 0.80, 0.95])?;
    Some(vec![left, right])
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

/// One reveal scene: a grid ordered by `sort_key` (0 Radial / 3 Random), culled by the
/// shared `lfo`, tinted `rgb`, shifted to `dx`. Returns its Output node.
fn build_reveal_scene(
    g: &mut Graph,
    lfo: NodeId,
    row: f32,
    sort_key: f32,
    dx: f32,
    rgb: [f32; 3],
) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let sort = g.add_node("motion.sort");
    let cull = g.add_node("motion.cull");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    for (n, col) in [
        (grid, 0.0),
        (sort, 1.0),
        (cull, 2.0),
        (tint, 3.0),
        (mv, 4.0),
        (output, 5.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: row,
            },
        );
    }

    wire(g, (grid, 0), (sort, 0))?;
    wire(g, (sort, 0), (cull, 0))?;
    wire(g, (cull, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (cull, 1))?; // shared reveal clock → cull amount

    // A 10×10 grid (100 dots, ~4.5 wide) centred on the origin — the sort's Radial
    // centre. `sort_key` picks Radial (centre-out) or Random (dissolve).
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 10.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    g.set_param(sort, "key", sort_key);
    g.set_param(sort, "seed", 7.0); // only used by the Random key
    g.set_param(cull, "mode", 0.0); // Fraction — keep the first amount·n (post-sort)
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

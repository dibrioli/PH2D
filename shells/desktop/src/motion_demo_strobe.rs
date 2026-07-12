//! The M4 FX demo — the **default Motion document**, showing the slice's two new
//! nodes: on the LEFT a bobbing grid that **casts a drop shadow**; on the RIGHT a
//! ring of orbiting elements smeared by **chromatic aberration**, clean at the axis
//! and fringed at the rim. Two independent scenes (each its own `motion.output`
//! sink), kept small so each new node reads on its own. A `#[path]` sibling of
//! `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (drop shadow): grid ─> scale ─> oscillator ─> drop_shadow ─> move(−7) ─> output
//! RIGHT (rgb split):   grid ─> scale ─> orbit ─> tint ─> rgb_split ─> move(+7) ─> output
//! ```
//!
//! The `motion.scale` in each chain is how a document asks for **small quads**: the
//! lowering's fallback for a stream with no `size` column is the IDENTITY (unit scale),
//! never a cosmetic number hidden in the shell (doc 39).
//!
//! - **drop_shadow** (`fx.drop_shadow`, doc 38): each element becomes two rows — its
//!   shadow (behind, in a block) and itself. Spin `Direction` in the params panel and
//!   the whole layout's shadow swings around it; drop `Distance` to 0 and the shadow
//!   hides exactly under the elements. The bob is there to prove the shadow **tracks**
//!   its caster every tick rather than being baked once.
//! - **rgb_split** (`fx.rgb_split`, doc 38) in **Aberration** mode: the fringe is zero
//!   at the layout's centroid and grows with the distance from it — so the middle of
//!   the ring stays clean while the rim smears red one way and cyan the other. Switch
//!   `Mode` to *Split* and the whole ring's channels slide apart uniformly (the glitch
//!   look). The `motion.tint` upstream makes the elements coloured, which is the
//!   falsifiable read: the fringes carry only the channels that colour **actually
//!   contains** (a blue element throws no red).
//!
//! See docs/Motion Nodes/38 (the ghost-copy FX). The whole value/pulse vocabulary +
//! the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const SHADOW_ROW: f32 = 0.0;
const SPLIT_ROW: f32 = 360.0;
/// The quad size both scenes ask for, as a fraction of their grid spacing — small
/// enough that the elements read as distinct dots with clear gaps between them.
const QUAD: f32 = 0.4;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the shadow
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let shadow = build_drop_shadow_scene(g)?;
    let split = build_rgb_split_scene(g)?;
    Some(vec![shadow, split])
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

/// Lay a chain out left-to-right on `row`, one card per column.
fn place(g: &mut Graph, row: f32, chain: &[NodeId]) {
    for (col, n) in chain.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: col as f32 * COL_W,
                y: row,
            },
        );
    }
}

/// LEFT: a bobbing grid casting a hard drop shadow. Returns its Output.
fn build_drop_shadow_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let osc = g.add_node("motion.oscillator");
    let shadow = g.add_node("fx.drop_shadow");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    place(g, SHADOW_ROW, &[grid, scale, osc, shadow, mv, output]);

    wire(g, (grid, 0), (scale, 0))?;
    wire(g, (scale, 0), (osc, 0))?;
    wire(g, (osc, 0), (shadow, 0))?;
    wire(g, (shadow, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    g.set_param(grid, "rows", 4.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    g.set_param(scale, "amount", QUAD);
    // A gentle bob, staggered across the set — the shadow has to follow it.
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 0.5);
    g.set_param(osc, "frequency", 0.5);
    g.set_param(osc, "phase_stagger", 0.08);
    // Down-and-right (the y-up world), far enough to read at the demo's scale.
    g.set_param(shadow, "direction", 315.0);
    g.set_param(shadow, "distance", 0.3);
    g.set_param(shadow, "a", 0.45);
    g.set_param(mv, "dx", -7.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

/// RIGHT: an orbiting, coloured ring smeared by lateral chromatic aberration.
/// Returns its Output.
fn build_rgb_split_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let orbit = g.add_node("motion.orbit");
    let tint = g.add_node("motion.tint");
    let split = g.add_node("fx.rgb_split");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    place(g, SPLIT_ROW, &[grid, scale, orbit, tint, split, mv, output]);

    wire(g, (grid, 0), (scale, 0))?;
    wire(g, (scale, 0), (orbit, 0))?;
    wire(g, (orbit, 0), (tint, 0))?;
    wire(g, (tint, 0), (split, 0))?;
    wire(g, (split, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    g.set_param(grid, "rows", 5.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 0.9);
    g.set_param(grid, "gap_y", 0.9);
    g.set_param(scale, "amount", QUAD);
    g.set_param(orbit, "speed", 0.15);
    // A cyan-ish body, so the fringes can only carry the channels it HAS: the R ghost
    // is nearly black and the G+B ghost carries the colour (doc 38 §2).
    g.set_param(tint, "r", 0.15);
    g.set_param(tint, "g", 0.75);
    g.set_param(tint, "b", 0.95);
    g.set_param(tint, "a", 1.0);
    // Aberration (radial): clean at the centroid, smeared at the rim.
    g.set_param(split, "mode", 1.0);
    g.set_param(split, "strength", 0.14);
    g.set_param(split, "opacity", 1.0);
    g.set_param(mv, "dx", 7.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

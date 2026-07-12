//! The M4 rig demo — the **default Motion document**, showing the slice's two new
//! nodes: on the LEFT a **skeleton at rest** (a chain of joints, curling by its Bend);
//! on the RIGHT the same skeleton **posed and resolved** — an oscillator writes the
//! joints' angles and `rig.fk` turns them into a pose, so the limb waves. Two
//! independent scenes (each its own `motion.output` sink), kept small so each new node
//! reads on its own. A `#[path]` sibling of `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (rest):  skeleton ─> scale ─> move(−7) ─> output
//! RIGHT (wave):  skeleton ─> oscillator (Rotation) ─> rig.fk ─> scale ─> move(+7) ─> output
//! ```
//!
//! **The contrast IS the lesson** (doc 40). The oscillator is a *generic* node: it writes
//! the `rot` column and knows nothing about bones — it leaves every joint exactly where
//! it was. **`rig.fk` is what turns posed angles into a pose.** Cut it out of the right
//! chain and the limb goes as still as the left one, with all its joints secretly bent.
//!
//! That works at all because **a skeleton is an ordinary instance stream** (M4.N3): its
//! joints are elements, `parent`/`len`/`rot` are ordinary columns, so every generic node
//! already works on a rig — no `Domain::Rig`, no contract change.
//!
//! The `motion.scale` in each chain is how a document asks for **small quads**: the
//! lowering's fallback for a stream with no `size` column is the IDENTITY (doc 39).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const REST_ROW: f32 = 0.0;
const WAVE_ROW: f32 = 360.0;
/// The quad size both scenes ask for — small enough that the joints read as distinct
/// beads on the chain rather than a merged blob.
const QUAD: f32 = 0.35;
/// Both limbs: long enough to read as a chain, short enough to stay on screen.
const JOINTS: f32 = 14.0;
const BONE: f32 = 0.55;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the rest
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let rest = build_rest_scene(g)?;
    let wave = build_wave_scene(g)?;
    Some(vec![rest, wave])
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

/// Wire a straight chain left-to-right, and lay it out one card per column.
fn chain(g: &mut Graph, row: f32, nodes: &[NodeId]) -> Option<()> {
    for (col, n) in nodes.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: col as f32 * COL_W,
                y: row,
            },
        );
    }
    for pair in nodes.windows(2) {
        wire(g, (pair[0], 0), (pair[1], 0))?;
    }
    Some(())
}

/// LEFT: the skeleton as authored — a limb curling by its `Bend`. Returns its Output.
fn build_rest_scene(g: &mut Graph) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    chain(g, REST_ROW, &[skel, scale, mv, output])?;

    g.set_param(skel, "joints", JOINTS);
    g.set_param(skel, "length", BONE);
    // A gentle bend per joint — the chain curls into an arc instead of a straight rod.
    g.set_param(skel, "angle", 9.0);
    g.set_param(skel, "root_angle", 100.0);
    g.set_param(scale, "amount", QUAD);
    g.set_param(mv, "dx", -7.0);
    g.set_param(mv, "dy", -3.0);
    Some(output)
}

/// RIGHT: the same limb, posed by a generic oscillator and RESOLVED by `rig.fk` — it
/// waves. Returns its Output.
fn build_wave_scene(g: &mut Graph) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let osc = g.add_node("motion.oscillator");
    let fk = g.add_node("rig.fk");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    chain(g, WAVE_ROW, &[skel, osc, fk, scale, mv, output])?;

    g.set_param(skel, "joints", JOINTS);
    g.set_param(skel, "length", BONE);
    g.set_param(skel, "angle", 0.0); // a straight limb at rest — the wave is the pose
    g.set_param(skel, "root_angle", 90.0);
    // Channel 2 = Rotation: the oscillator writes the joints' LOCAL angles. The stagger
    // gives each joint a later phase, so the bend travels down the limb like a whip.
    g.set_param(osc, "channel", 2.0);
    g.set_param(osc, "amplitude", 14.0); // degrees per joint
    g.set_param(osc, "frequency", 0.5);
    g.set_param(osc, "phase_stagger", 0.06);
    g.set_param(scale, "amount", QUAD);
    g.set_param(mv, "dx", 7.0);
    g.set_param(mv, "dy", -3.0);
    Some(output)
}

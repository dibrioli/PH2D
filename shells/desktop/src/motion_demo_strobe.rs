//! The M4 IK demo — the **default Motion document**: two limbs **chasing the same
//! orbiting goal**. On the LEFT a three-joint arm solved in closed form by the law of
//! cosines (`rig.ik_2bone`); on the RIGHT a ten-joint tentacle solved iteratively by
//! **FABRIK** (`rig.fabrik`). The goal itself is drawn as a fat dot beside each limb, so
//! what the limbs are reaching for is visible and not a matter of faith.
//!
//! ```text
//!  goal:      grid(1) ─> move(2.4, 0) ─> orbit ─┬─────────────────────────────────┐
//!                                               │                                 │
//!  LEFT  arm: skeleton(3) ─> ik_2bone <─────────┤                                 │
//!                              └─> scale ─> move(−7) ─> output                    │
//!  RIGHT tentacle: skeleton(10) ─> fabrik <─────┘                                 │
//!                              └─> scale ─> move(+7) ─> output                    │
//!  goal dots:                             scale(big) ─> move(−7) ─> output  <─────┤
//!                                                    ─> move(+7) ─> output  <─────┘
//! ```
//!
//! **Four sinks, no merge node.** Every `motion.output` in the document lowers onto the
//! same draw buffer, so the goal dot is its own little scene rather than something
//! concatenated into the limb's stream. (`motion.combine` would have worked too — but it
//! zero-fills the columns an input lacks, so merging a tinted goal into an untinted
//! skeleton would paint the whole limb transparent black. Sinks compose without that.)
//!
//! The limbs are solved at the ORIGIN and moved apart afterwards, so both can chase the
//! same goal; each goal dot is moved by the same offset as the limb it belongs to.
//!
//! Both solvers write a **pose** (angles), never positions — so the bones are rigid, the
//! root stays nailed, and an unreachable goal extends the limb instead of stretching it
//! (doc 41). Drag `Iterations` on FABRIK down to 1 and watch it lag behind the goal.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const GOAL_ROW: f32 = -170.0;
const ARM_ROW: f32 = 0.0;
const TENTACLE_ROW: f32 = 200.0;
const DOT_ROW: f32 = 400.0;

/// The quad size the limbs ask for, and the fatter one the goal gets — the lowering's
/// fallback for a stream with no `size` column is the IDENTITY (doc 39), so a document
/// that wants small quads says so.
const JOINT_QUAD: f32 = 0.28;
const GOAL_QUAD: f32 = 0.6;

/// How far the goal orbits from the limbs' shared root, and how fast (`motion.orbit`'s
/// speed is DEGREES PER SECOND — 90 is a quarter turn a second, a lap every four).
const GOAL_RADIUS: f32 = 2.4;
const GOAL_SPEED: f32 = 90.0;

/// The arm: three joints, two bones — reach 3.0, comfortably past the goal's orbit, so
/// the elbow really has to bend rather than sitting at full stretch.
const ARM_BONE: f32 = 1.5;
/// The tentacle: ten joints of 0.34 — reach 3.06, about the same, so the two solvers are
/// answering the same question with very different machinery.
const TENTACLE_JOINTS: f32 = 10.0;
const TENTACLE_BONE: f32 = 0.34;

/// Author every scene into `g`; returns the four Output nodes (the sinks), in the order
/// the tests read them: arm · tentacle · the arm's goal dot · the tentacle's goal dot.
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let goal = build_goal(g)?;
    let arm = build_arm(g, goal)?;
    let tentacle = build_tentacle(g, goal)?;
    let (dot_l, dot_r) = build_goal_dots(g, goal)?;
    Some(vec![arm, tentacle, dot_l, dot_r])
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

/// Wire a straight chain left-to-right (port 0 to port 0), laying one card per column
/// starting at `col`.
fn chain(g: &mut Graph, row: f32, col: usize, nodes: &[NodeId]) -> Option<()> {
    for (i, n) in nodes.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: (col + i) as f32 * COL_W,
                y: row,
            },
        );
    }
    for pair in nodes.windows(2) {
        wire(g, (pair[0], 0), (pair[1], 0))?;
    }
    Some(())
}

/// The shared goal: a single point orbiting the origin. Returns the node both solvers
/// (and both goal dots) read.
fn build_goal(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.move");
    let orbit = g.add_node("motion.orbit");
    chain(g, GOAL_ROW, 0, &[grid, out, orbit])?;

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    // Push the point off the pivot — an orbit around the point itself would stand still.
    g.set_param(out, "dx", GOAL_RADIUS);
    g.set_param(out, "dy", 0.0);
    g.set_param(orbit, "speed", GOAL_SPEED);
    Some(orbit)
}

/// LEFT: the three-joint arm, solved by the law of cosines. Returns its Output.
fn build_arm(g: &mut Graph, goal: NodeId) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let ik = g.add_node("rig.ik_2bone");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    chain(g, ARM_ROW, 1, &[skel, ik, scale, mv, output])?;
    wire(g, (goal, 0), (ik, 1))?; // the goal feeds the solver's `target` port

    g.set_param(skel, "joints", 3.0);
    g.set_param(skel, "length", ARM_BONE);
    g.set_param(skel, "angle", 0.0);
    g.set_param(skel, "root_angle", 90.0);
    g.set_param(scale, "amount", JOINT_QUAD);
    g.set_param(mv, "dx", -7.0);
    Some(output)
}

/// RIGHT: the ten-joint tentacle, solved by FABRIK. Returns its Output.
fn build_tentacle(g: &mut Graph, goal: NodeId) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let solver = g.add_node("rig.fabrik");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    chain(g, TENTACLE_ROW, 1, &[skel, solver, scale, mv, output])?;
    wire(g, (goal, 0), (solver, 1))?;

    g.set_param(skel, "joints", TENTACLE_JOINTS);
    g.set_param(skel, "length", TENTACLE_BONE);
    g.set_param(skel, "angle", 0.0);
    g.set_param(skel, "root_angle", 90.0);
    g.set_param(solver, "iterations", 10.0);
    g.set_param(scale, "amount", JOINT_QUAD);
    g.set_param(mv, "dx", 7.0);
    Some(output)
}

/// The goal, drawn as a fat dot beside each limb — the same point, moved by the same
/// offset as the limb that chases it. Returns both Outputs.
fn build_goal_dots(g: &mut Graph, goal: NodeId) -> Option<(NodeId, NodeId)> {
    let scale = g.add_node("motion.scale");
    let mv_l = g.add_node("motion.move");
    let out_l = g.add_node("motion.output");
    let mv_r = g.add_node("motion.move");
    let out_r = g.add_node("motion.output");

    chain(g, DOT_ROW, 2, &[scale, mv_l, out_l])?;
    chain(g, DOT_ROW + 120.0, 3, &[mv_r, out_r])?;
    wire(g, (goal, 0), (scale, 0))?;
    wire(g, (scale, 0), (mv_r, 0))?;

    g.set_param(scale, "amount", GOAL_QUAD);
    g.set_param(mv_l, "dx", -7.0);
    g.set_param(mv_r, "dx", 7.0);
    Some((out_l, out_r))
}

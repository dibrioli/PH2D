//! The M4 rig-closing demo — the **default Motion document**: two limbs chasing the same
//! breathing goal, showing the slice's two nodes.
//!
//! - **LEFT — `rig.rubber_hose`**: a limb with **no elbow**. Its curvature is constant, so
//!   it curls into one smooth arc instead of hinging. Put it beside yesterday's `ik_2bone`
//!   and the whole point of the style is visible at a glance.
//! - **RIGHT — `rig.skin_deformer`**: a cloud of points **skinned** to a FABRIK tentacle.
//!   The skeleton is never drawn; what you see is the *flesh* following it (Linear Blend
//!   Skinning, envelope weights).
//!
//! ```text
//!  goal:  grid(1) ─> move(2, 0) ─> oscillator(X) ─> orbit ─┬───────────┐
//!  LEFT:  skeleton(9) ─> rubber_hose <─────────────────────┤           │
//!                           └─> scale ─> move(−7) ─> output            │
//!  RIGHT: skeleton(8) ─┬─────────────────────────> skin.rest           │
//!                      └─> fabrik <───────────────┤                    │
//!                            └──────────────────> skin.posed           │
//!         grid(5×24) ─> move ─────────────────────> skin.in            │
//!                            skin └─> scale ─> move(+7) ─> output       │
//!  goal dots:                       scale(big) ─> move(∓7) ─> output <─┘
//! ```
//!
//! **The bind pose is a WIRE.** `rig.skin_deformer` takes the skeleton twice — once as
//! authored (`rest`) and once as solved (`posed`) — because a skin IS the difference
//! between those two. Nothing is snapshotted behind your back at a moment you have to
//! remember: the bind is a thing you can see in the graph, and cut.
//!
//! The goal BREATHES in and out (an oscillator on X upstream of the orbit, where +X is the
//! radial direction). A goal at a fixed radius around a limb's own root would keep the
//! reach constant — and a hose whose reach never changes never changes its curl, the same
//! trap that made the elbow look frozen the day before (doc 41 §6-bis).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const GOAL_ROW: f32 = -170.0;
const HOSE_ROW: f32 = 0.0;
const SKIN_ROW: f32 = 200.0;
const FLESH_ROW: f32 = 340.0;
const DOT_ROW: f32 = 480.0;
const RAIN_ROW: f32 = 640.0;
/// The interior sits one card below the zone: a state loop reads right-to-left, and putting it
/// on the same row would run it straight through the cards it feeds.
const RAIN_INTERIOR_DY: f32 = 140.0;

/// Quad sizes: the hose's joints, the flesh's points, and the fat goal dot. (The lowering's
/// fallback for a stream with no `size` column is the IDENTITY — doc 39 — so a document
/// that wants small quads says so, with a `motion.scale`.)
const JOINT_QUAD: f32 = 0.28;
const FLESH_QUAD: f32 = 0.16;
const GOAL_QUAD: f32 = 0.6;

/// The goal's orbit, and the pulse that makes its distance breathe (`motion.orbit`'s speed
/// is DEGREES PER SECOND).
const GOAL_RADIUS: f32 = 2.0;
const GOAL_SPEED: f32 = 90.0;
const GOAL_PULSE: f32 = 0.9;
const GOAL_PULSE_HZ: f32 = 0.35;

/// The hose: nine joints of 0.36 — a reach of 2.88, so the breathing goal (1.1 .. 2.9) has
/// it curling from a tight coil to nearly straight.
const HOSE_JOINTS: f32 = 9.0;
const HOSE_BONE: f32 = 0.36;

/// The snow (O4): flakes born along a line at the top of the disc, falling under gravity, dying
/// as they leave it. Birth and death balance, so the population settles.
const SNOW_SITES: f32 = 14.0;
const SNOW_SITE_GAP: f32 = 0.36;
const SNOW_SKY_Y: f32 = 2.6;
const SNOW_RATE: f32 = 24.0;
const SNOW_GUST: f32 = 0.35;
const RAIN_QUAD: f32 = 0.18;
const RAIN_GRAVITY: f32 = 4.0;
/// The kill disc, around the seed's own centre: a LINEAR falloff of radius 3.4, culled at 0.05,
/// so a drop dies at 0.95 x 3.4 = 3.2 world units out. The seed's own corners are 1.78 out, so
/// every drop is born alive and dies only by falling — a seed that started outside its own kill
/// mask would cull its corners on frame one, which is what the first numbers here did.
const RAIN_KILL_R: f32 = 3.4;
const RAIN_KILL_KEEP: f32 = 0.05;
/// Where the whole scene sits, so it does not land on the limbs.
const RAIN_X: f32 = 0.0;
const RAIN_Y: f32 = 2.4;

/// The skinned tentacle: eight joints of 0.42 (reach 2.94), with the flesh wrapped around
/// its rest pose — a strip of points over the limb, which stands straight up from the root.
const SKIN_JOINTS: f32 = 8.0;
const SKIN_BONE: f32 = 0.42;
const FLESH_COLS: f32 = 5.0;
const FLESH_ROWS: f32 = 24.0;
const FLESH_GAP_X: f32 = 0.16;
const FLESH_GAP_Y: f32 = 0.13;

/// Author every scene into `g`; returns the Output nodes (the sinks), in the order the tests
/// read them: the hose · the flesh · the hose's goal dot · the tentacle's goal dot · the rain.
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let goal = build_goal(g)?;
    let hose = build_hose(g, goal)?;
    let flesh = build_skinned_tentacle(g, goal)?;
    let (dot_l, dot_r) = build_goal_dots(g, goal)?;
    let rain = build_sim_zone(g)?;
    build_inert_example(g);
    Some(vec![hose, flesh, dot_l, dot_r, rain])
}

/// **The Simulation Zone** (O4, docs 48 + 49) — SNOW: it is born, it falls, it dies.
///
/// ```text
///                    ┌──────────┐ out ──┬──→ scale ──→ move ──→ output
///                    │ sim.zone │       │
///           state ←──┴──────────┘       ⊙  (the state entry: last tick's population)
///             ↑                         │
///   cull ← falloff ← sim.step ← wind ← combine(in0)
///                                          ↑ in1
///                     grid(1x14) → move → sim.spawn        (this tick's NEWBORNS)
/// ```
///
/// **The zone's `init` is deliberately left UNWIRED**: the population starts at nothing and is
/// entirely BORN. Every flake therefore has a unique identity from birth (`sim.spawn` stamps the
/// birth ordinal), and the whole triangle of a particle system is on the canvas:
///
/// - **BIRTH** — `sim.spawn` emits only *this tick's* newborns, and `motion.combine` merges them
///   into the live state. (A `motion.emitter` cannot do this: it is stateless, so merging it
///   would merge the same particles again, every frame, forever.)
/// - **LIFE** — `force.wind` + `sim.step`: velocity ACCUMULATES in the state, so the flakes
///   accelerate instead of drifting at a constant rate.
/// - **DEATH** — `motion.falloff` + `motion.cull`: inside a zone that pair is not a filter but a
///   KILL. What falls out of the circle stays out.
///
/// Birth and death balance, so the snow reaches a **steady state** and stays there. Every node
/// in the interior but the two `sim.*` ones already existed; the zone is what made them a
/// simulation.
fn build_sim_zone(g: &mut Graph) -> Option<NodeId> {
    let zone = g.add_node("sim.zone");
    let combine = g.add_node("motion.combine");
    let wind = g.add_node("force.wind");
    let step = g.add_node("sim.step");
    let falloff = g.add_node("motion.falloff");
    let cull = g.add_node("motion.cull");
    let sky = g.add_node("motion.grid");
    let lift = g.add_node("motion.move");
    let spawn = g.add_node("sim.spawn");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    // Three rows: the zone and its render chain · the interior (which reads right-to-left, the
    // shape a state loop has on a canvas that reads left-to-right) · the birth chain under it.
    chain(g, RAIN_ROW, 1, &[zone])?;
    chain(g, RAIN_ROW, 2, &[scale, mv, output])?;
    wire(g, (zone, 0), (scale, 0))?;
    for (i, n) in [combine, wind, step, falloff, cull].iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * COL_W,
                y: RAIN_ROW + RAIN_INTERIOR_DY,
            },
        );
    }
    wire(g, (combine, 0), (wind, 0))?;
    wire(g, (wind, 0), (step, 0))?;
    wire(g, (step, 0), (falloff, 0))?;
    wire(g, (falloff, 0), (cull, 0))?;
    wire(g, (cull, 0), (zone, 1))?; // the interior's END lands on `state`

    chain(g, RAIN_ROW + 2.0 * RAIN_INTERIOR_DY, 0, &[sky, lift, spawn])?;
    wire(g, (spawn, 0), (combine, 1))?; // …the newborns join the population

    // **The state entry.** The zone's PREVIOUS output enters the interior at its head — here the
    // free `in0` of the merge, which is exactly where the editor's plumbing would put it (it is
    // the one `pre` edge in the scene, and it draws as a portal badge, never as a spline). The
    // demo writes the topology the plumbing would, so the boot document is already reconciled.
    g.connect(Edge {
        from: (zone, 0),
        to: (combine, 0),
        delayed: true,
    })
    .ok()?;

    // The sky: a wide, thin line of birth sites, lifted to the top of the disc.
    g.set_param(sky, "rows", 1.0);
    g.set_param(sky, "cols", SNOW_SITES);
    g.set_param(sky, "gap_x", SNOW_SITE_GAP);
    g.set_param(lift, "dy", SNOW_SKY_Y);
    g.set_param(spawn, "rate", SNOW_RATE);
    // Gravity: `force.wind`'s angle is degrees, 270 = straight down (y-up world). A little gust
    // so the flakes do not fall in lockstep columns.
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", RAIN_GRAVITY);
    g.set_param(wind, "gust", SNOW_GUST);
    // The kill: `motion.falloff` writes a per-element mask from a disc around the origin, and
    // `motion.cull` keeps only what is still inside it (Falloff mode).
    g.set_param(falloff, "radius", RAIN_KILL_R);
    g.set_param(falloff, "curve", 0.0); // 0 = Linear: the mask is 1 - d/radius
    g.set_param(cull, "mode", 1.0); // Falloff
    g.set_param(cull, "amount", RAIN_KILL_KEEP); // …keep what the mask still calls "inside"
    g.set_param(scale, "amount", RAIN_QUAD);
    g.set_param(mv, "dx", RAIN_X);
    g.set_param(mv, "dy", RAIN_Y);
    Some(output)
}

/// **A node wired to nothing** — deliberately, as the demo of the inline readouts (F2) and of
/// the inert veil (F3): it is a real `motion.grid`, it does exactly nothing, no sink consumes
/// it, so the cook never pulls it — and the editor veils it and leaves its reading blank. The
/// whole feature in one card.
fn build_inert_example(g: &mut Graph) {
    let orphan = g.add_node("motion.grid");
    g.set_pos(
        orphan,
        Pos {
            x: 6.0 * COL_W,
            y: GOAL_ROW,
        },
    );
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

/// Wire a straight chain left-to-right (port 0 to port 0), one card per column from `col`.
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

/// The shared goal: one point orbiting the origin, its distance breathing in and out.
fn build_goal(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.move");
    let pulse = g.add_node("motion.oscillator");
    let orbit = g.add_node("motion.orbit");
    chain(g, GOAL_ROW, 0, &[grid, out, pulse, orbit])?;

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    // Push the point off the pivot — an orbit around the point itself would stand still.
    g.set_param(out, "dx", GOAL_RADIUS);
    // Channel 0 = X. The point sits on the +x axis here, so an X wobble IS a change of
    // RADIUS — which is what makes the limbs' reach (and so their curl) change at all.
    g.set_param(pulse, "channel", 0.0);
    g.set_param(pulse, "amplitude", GOAL_PULSE);
    g.set_param(pulse, "frequency", GOAL_PULSE_HZ);
    g.set_param(pulse, "phase_stagger", 0.0);
    g.set_param(orbit, "speed", GOAL_SPEED);
    Some(orbit)
}

/// LEFT: the elbowless limb. Returns its Output.
fn build_hose(g: &mut Graph, goal: NodeId) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let hose = g.add_node("rig.rubber_hose");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    chain(g, HOSE_ROW, 1, &[skel, hose, scale, mv, output])?;
    wire(g, (goal, 0), (hose, 1))?;

    g.set_param(skel, "joints", HOSE_JOINTS);
    g.set_param(skel, "length", HOSE_BONE);
    g.set_param(skel, "root_angle", 90.0);
    g.set_param(scale, "amount", JOINT_QUAD);
    g.set_param(mv, "dx", -7.0);
    Some(output)
}

/// RIGHT: a FABRIK tentacle with FLESH on it — the skeleton itself is never drawn.
fn build_skinned_tentacle(g: &mut Graph, goal: NodeId) -> Option<NodeId> {
    let skel = g.add_node("rig.skeleton");
    let solver = g.add_node("rig.fabrik");
    let flesh = g.add_node("motion.grid");
    let flesh_place = g.add_node("motion.move");
    let skin = g.add_node("rig.skin_deformer");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    chain(g, SKIN_ROW, 1, &[skel, solver])?;
    chain(g, FLESH_ROW, 1, &[flesh, flesh_place])?;
    chain(g, SKIN_ROW, 3, &[skin, scale, mv, output])?;
    wire(g, (goal, 0), (solver, 1))?;
    // The three wires that make a skin: the points, the BIND pose, the SOLVED pose.
    wire(g, (flesh_place, 0), (skin, 0))?;
    wire(g, (skel, 0), (skin, 1))?; // rest — the skeleton as authored
    wire(g, (solver, 0), (skin, 2))?; // posed — the same skeleton, solved

    g.set_param(skel, "joints", SKIN_JOINTS);
    g.set_param(skel, "length", SKIN_BONE);
    g.set_param(skel, "root_angle", 90.0);
    g.set_param(solver, "iterations", 12.0);
    // A strip of points laid over the limb's rest pose, which stands straight up from the
    // origin — so the flesh starts wrapped around the bones it is bound to.
    g.set_param(flesh, "rows", FLESH_ROWS);
    g.set_param(flesh, "cols", FLESH_COLS);
    g.set_param(flesh, "gap_x", FLESH_GAP_X);
    g.set_param(flesh, "gap_y", FLESH_GAP_Y);
    g.set_param(flesh_place, "dy", (SKIN_JOINTS - 1.0) * SKIN_BONE * 0.5);
    g.set_param(scale, "amount", FLESH_QUAD);
    g.set_param(mv, "dx", 7.0);
    Some(output)
}

/// The goal, drawn as a fat dot beside each limb — the same point, moved by the same offset
/// as the limb that chases it. Returns both Outputs.
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

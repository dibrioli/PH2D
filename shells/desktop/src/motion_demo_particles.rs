//! The particle-fountain half of the default Motion document (M2-particles).
//! Declared by `motion_state` as a `#[path]` sibling, to keep both under the
//! file LOC cap.
//!
//! A second, independent scene in the same document — proving the multi-sink
//! render (`MotionCookPump::pump` lowers every `motion.output` node onto one
//! buffer, so two scenes need no stream-merging node) and showing the dynamics
//! engine on a churning set:
//!
//! ```text
//! emitter → integrate.rest → tint → output
//!           ⟲ wind → curl → drag → integrate.forces
//! ```
//!
//! The force branch mirrors the topology the editor plumbing derives when a
//! user wires the chain into `forces` (docs/Motion Nodes/03): the state enters
//! the branch's dangling head through the engine-managed `pre` (drawn as
//! portal badges), never as a hand-authored loop.
//!
//! - **emitter**: a stateless fountain (its alive set is a pure function of the
//!   playhead) firing up out of the lower-left, stamping the `id` column.
//! - **integrate**: seeds each newborn from its muzzle `vel` and matches every
//!   survivor's state **by id** as the set churns — the capability this wave
//!   added. Displacement composes over the emitter's static origin.
//! - **wind** (`gust = 0`, blowing straight down) IS gravity; **curl** is the
//!   divergence-free turbulence that makes the plume swirl instead of arcing
//!   like a ballistic jet; **drag** keeps it from running away.
//! - **tint** Gradient paints by `Index`, which the emitter emits oldest-first
//!   — so a particle cools from warm at birth to cyan as it ages.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of the fountain's card row (below the grid chain).
const ROW_Y: f32 = 560.0;
const COL_W: f32 = 220.0;

/// Author the fountain into `g`; returns its Output node (the second sink).
/// `None` if any wire is rejected — the caller then renders the grid alone.
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let emitter = g.add_node("motion.emitter");
    let integrate = g.add_node("motion.integrate");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let wind = g.add_node("force.wind");
    let curl = g.add_node("force.curl");
    let drag = g.add_node("force.drag");

    // Forward trunk: emitter → integrate.rest → tint → output. The integrate
    // sits one column further right so the force row below flows INTO it
    // left→right (no wire ever doubles back).
    for (n, col) in [(emitter, 0.0), (integrate, 3.0), (tint, 4.0), (output, 5.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y,
            },
        );
    }
    for (from, to) in [(emitter, integrate), (integrate, tint), (tint, output)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Force branch into `forces`: ⟲ gravity → turbulence → damping. The `pre`
    // into the head (wind) is the engine-managed state entry.
    g.connect(Edge {
        from: (integrate, 0),
        to: (wind, 0),
        delayed: true,
    })
    .ok()?;
    for (from, to) in [(wind, curl), (curl, drag)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.connect(Edge {
        from: (drag, 0),
        to: (integrate, 1),
        delayed: false,
    })
    .ok()?;
    for (i, n) in [wind, curl, drag].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: i as f32 * COL_W,
                y: ROW_Y + 260.0,
            },
        );
    }

    // A fountain firing up-and-right from the lower-left of the frame (the
    // camera frames roughly ±5 world units vertically).
    g.set_param(emitter, "rate", 50.0);
    g.set_param(emitter, "life", 3.5);
    g.set_param(emitter, "speed", 5.5);
    g.set_param(emitter, "angle", 78.0); // up, tilted right
    g.set_param(emitter, "spread", 30.0);
    g.set_param(emitter, "x", -5.5);
    g.set_param(emitter, "y", -4.2);
    g.set_param(emitter, "max", 256.0);
    // Grains, not a poured ribbon: at 50/s and speed 5.5 the spacing between
    // consecutive particles is ~0.11 world units, so they must be smaller still.
    g.set_param(emitter, "size", 0.09);
    // Gravity = a gustless wind blowing straight down (Y-up world).
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", 4.0);
    g.set_param(wind, "gust", 0.0);
    // Turbulence: broad, slow eddies so the plume curls rather than shivers.
    g.set_param(curl, "strength", 4.5);
    g.set_param(curl, "scale", 0.30);
    g.set_param(curl, "speed", 0.25);
    g.set_param(curl, "octaves", 2.0);
    // A touch of air resistance so the plume settles instead of escaping.
    g.set_param(drag, "coefficient", 0.30);
    // Age ramp: warm at birth (Index 0 = oldest… so End is the newest) — the
    // emitter emits oldest-first, hence Start = the cooling tail.
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "r", 0.15);
    g.set_param(tint, "g", 0.75);
    g.set_param(tint, "b", 1.0); // old: cyan
    g.set_param(tint, "r2", 1.0);
    g.set_param(tint, "g2", 0.65);
    g.set_param(tint, "b2", 0.1); // new: warm amber
    Some(output)
}

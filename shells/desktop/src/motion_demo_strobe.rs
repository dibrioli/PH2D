//! The M3 demo — the **sole scene of the default Motion document**: a **twisting
//! sunflower**. A deliberately SMALL scene (~8 nodes) opening M3 (distributions +
//! deformers) so the two new nodes read on their own. A `#[path]` sibling of
//! `motion_state`, kept out of it for the LOC cap.
//!
//! ```text
//! fibonacci → twist → tint → drive_size → output
//!             lfo ─────┘ (amount)
//! fibonacci → instance_field → size_range → drive_size.value
//! ```
//!
//! - **fibonacci** (`motion.fibonacci`, doc 18): the Vogel **phyllotaxis** generator
//!   — `count` seeds on a golden-angle spiral (`r = spacing·√i`), the sunflower
//!   packing. The first M3 distribution, a Source node like `motion.grid`.
//! - **twist** (`motion.twist`, doc 18): the first **deformer** — rotates each seed
//!   about the centre by an angle that grows with its radius, so the rim coils and
//!   the centre stays put. Its strength is a VALUE input (`amount`), driven by a
//!   slow `value.lfo`, so the spiral **coils and uncoils in time** — the value
//!   domain (docs 12–17) animating an M3 deformer.
//! - **instance_field(Ramp) → size_range → drive_size**: sizes the seeds by index
//!   (small at the centre, larger at the rim — a real sunflower), reusing the value
//!   nodes on the new distribution.
//!
//! The payoff: a **golden-angle sunflower that coils and uncoils**, its seeds
//! graded small→big — a rich generative distribution (fibonacci) reshaped by an
//! animated deformer (twist), the M3 *generate → deform* pipeline made visible. See
//! docs/Motion Nodes/18 (fibonacci+twist). The whole value/pulse vocabulary of the
//! earlier scenes stays registered (drop it in the editor).
//!
//! This scene is a pure function of the playhead (the `lfo` is Temporal; nothing
//! holds `pre` state), so it needs no self-loops — the checkpoint/restore loop test
//! builds its own sequential doc (`motion_bridge_tests.rs`).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the twisting-sunflower scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let fibonacci = g.add_node("motion.fibonacci");
    let twist = g.add_node("motion.twist");
    let tint = g.add_node("motion.tint");
    let drive_size = g.add_node("motion.drive");
    let output = g.add_node("motion.output");
    let instance_field = g.add_node("value.instance_field");
    let size_range = g.add_node("value.map_range");
    let lfo = g.add_node("value.lfo");

    // Visible trunk: fibonacci → twist → tint → drive_size → output.
    for (n, col) in [
        (fibonacci, 0.0),
        (twist, 1.0),
        (tint, 2.0),
        (drive_size, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y,
            },
        );
    }
    for (from, to) in [
        (fibonacci, twist),
        (twist, tint),
        (tint, drive_size),
        (drive_size, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Value branches. The `lfo` animates the twist's `amount` (coil/uncoil). The
    // `instance_field` Ramp (its count read from the spiral) sizes the seeds by
    // index through `size_range`.
    for (from, to) in [
        ((lfo, 0), (twist, 1)),                // lfo → twist.amount (animate the coil)
        ((fibonacci, 0), (instance_field, 0)), // spiral count → instance_field
        ((instance_field, 0), (size_range, 0)),
        ((size_range, 0), (drive_size, 1)), // graded size → drive_size.value
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    for (n, col, dy) in [
        (instance_field, 0.5, 220.0),
        (size_range, 1.5, 220.0),
        (lfo, 1.0, 360.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // A sunflower of 180 seeds; spacing puts the rim at ~2 world units (in frame).
    g.set_param(fibonacci, "count", 180.0);
    g.set_param(fibonacci, "spacing", 0.15);
    // `angle` defaults to the golden angle — the sunflower packing.
    // twist: coil the rim up to 200° at full amount; centred on the origin.
    g.set_param(twist, "angle", 200.0);
    g.set_param(twist, "pivot_x", 0.0);
    g.set_param(twist, "pivot_y", 0.0);
    // A warm amber base (a sunflower).
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // Seeds sized by index (Ramp) — small at the centre, larger at the rim.
    g.set_param(instance_field, "mode", 1.0); // Ramp
    g.set_param(size_range, "in_lo", 0.0);
    g.set_param(size_range, "in_hi", 1.0);
    g.set_param(size_range, "out_lo", 0.04);
    g.set_param(size_range, "out_hi", 0.13);
    g.set_param(drive_size, "channel", 3.0); // Size
    g.set_param(drive_size, "scale", 1.0);
    g.set_param(drive_size, "mode", 1.0); // Set
    // lfo → amount: a slow (4 s) sine kept in [0, 1] (amplitude 0.5, offset 0.5),
    // so the twist eases 0 → full coil → 0. Unconnected `in` → a length-1 GLOBAL
    // amount (the whole spiral coils together).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

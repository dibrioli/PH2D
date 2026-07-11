//! The M3 morph demo — the **sole scene of the default Motion document**: a
//! **sunflower that dissolves into a blue-noise cloud and reforms**. A small scene
//! (~9 nodes) showing an M3 distribution and the crossfade deformer. A `#[path]`
//! sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! ```text
//! fibonacci ─┐
//! scatter ───┼→ morph → tint → drive_size → output
//! lfo ───────┘ (blend)
//! morph → instance_field → size_range → drive_size.value
//! ```
//!
//! - **fibonacci** (`motion.fibonacci`, doc 18): the ordered golden-angle spiral
//!   (180 seeds) — the morph's `a`.
//! - **scatter** (`motion.scatter`, doc 19): the M3 **blue-noise** distribution
//!   (180 points, best-candidate) — an evenly-random cloud, the exact opposite of
//!   the spiral's order — the morph's `b`.
//! - **morph** (`motion.morph`, doc 19): the **vertex crossfade** — it lerps each
//!   seed from its spiral position toward its scatter position by a `blend` VALUE,
//!   driven by a slow `value.lfo`, so the sunflower **melts into the cloud and
//!   reforms** in time (the value domain animating an M3 deformer).
//! - **instance_field(Ramp) → size_range → drive_size**: sizes the seeds small→big
//!   by index, so each keeps its size as it migrates.
//!
//! The payoff: a **golden-angle sunflower dissolving into an even blue-noise cloud
//! and back** — order ⇄ randomness, two M3 distributions bridged by a crossfade,
//! the *generate → deform (value-driven)* pattern again. See docs/Motion Nodes/18
//! (fibonacci), 19 (scatter+morph). The whole value/pulse vocabulary stays
//! registered (drop it in the editor). Pure function of the playhead (the lfo is
//! Temporal; nothing holds `pre` state).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the morph scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let fibonacci = g.add_node("motion.fibonacci");
    let scatter = g.add_node("motion.scatter");
    let morph = g.add_node("motion.morph");
    let tint = g.add_node("motion.tint");
    let drive_size = g.add_node("motion.drive");
    let output = g.add_node("motion.output");
    let instance_field = g.add_node("value.instance_field");
    let size_range = g.add_node("value.map_range");
    let lfo = g.add_node("value.lfo");

    // Visible trunk: morph → tint → drive_size → output.
    for (n, col) in [(morph, 1.0), (tint, 2.0), (drive_size, 3.0), (output, 4.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y,
            },
        );
    }
    for (from, to) in [(morph, tint), (tint, drive_size), (drive_size, output)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // The two shapes feed the morph; the lfo animates its blend. The morphed
    // stream sizes its seeds by index and drives Size.
    for (from, to) in [
        ((fibonacci, 0), (morph, 0)),      // spiral → morph.a
        ((scatter, 0), (morph, 1)),        // blue-noise → morph.b
        ((lfo, 0), (morph, 2)),            // lfo → morph.blend
        ((morph, 0), (instance_field, 0)), // morphed count → instance_field
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
        (fibonacci, 0.0, 220.0),
        (scatter, 0.0, 340.0),
        (lfo, 0.0, 460.0),
        (instance_field, 2.0, 220.0),
        (size_range, 3.0, 220.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // Shape a: a 180-seed sunflower, rim at ~2 world units.
    g.set_param(fibonacci, "count", 180.0);
    g.set_param(fibonacci, "spacing", 0.15);
    // Shape b: 180 blue-noise points in a 4×4 field (~±2, the sunflower's extent).
    g.set_param(scatter, "count", 180.0);
    g.set_param(scatter, "width", 4.0);
    g.set_param(scatter, "height", 4.0);
    g.set_param(scatter, "seed", 3.0);
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
    // lfo → blend: a slow (4 s) sine kept in [0, 1] (amplitude 0.5, offset 0.5),
    // so the morph eases spiral → cloud → spiral. Unconnected `in` → a length-1
    // GLOBAL blend (the whole set crossfades together).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

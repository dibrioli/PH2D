//! The pulse-loop scene — the **sole scene of the default Motion document** (M2
//! pulse + value families). A `#[path]` sibling of `motion_state`, kept out of
//! it for the LOC cap.
//!
//! It proves the **pulse type AND the value domain** end to end — a source
//! reduces to a value, the value drives a channel, all off one beat
//! (docs/Motion Nodes/06, 08, 09, 12):
//!
//! ```text
//! grid → move → tint → drive → strobe → output
//!        grid → beat ⟲ → { counter.pulse, strobe.pulse }
//!               counter.out --pre--> counter.state ; counter.out → drive.value
//!               strobe.out  --pre--> strobe.state
//! ```
//!
//! - **beat** (`pulse.beat`, doc 09): the metronome — emits the PULSE straight
//!   from the playhead, one beat per `period`. No transform channel anywhere
//!   (the earlier scene faked the clock by oscillating Rotation into a threshold
//!   — doc 09 §1's "clock hack"). Its cycle index rides a `pre` self-loop; the
//!   one pulse fans out to BOTH consumers below.
//! - **counter** (`pulse.counter`, doc 12): the PURE reducer — each beat advances
//!   a persistent count that ZIGZAGS 0..N..0 and emits it as a **value** (never a
//!   channel). Its monotonic tick rides its own `pre` self-loop.
//! - **drive** (`motion.drive`, doc 12): routes the counter's value onto the X
//!   channel (`value · scale`), sliding the whole grid a discrete notch — the
//!   sequencer sweep, now COMPOSED from a value instead of bundled. The same
//!   value could fan out to a second drive (X *and* Rotation) — the value-domain
//!   win that `motion.step` (which bundles reduce+apply) cannot do.
//! - **strobe** (`motion.strobe`): the SAME pulse lights every dot to full glow —
//!   a size boost + white flash — that decays geometrically. Its own `pre` loop.
//!
//! The payoff of both families in one shot: the grid **steps to a new place and
//! flashes on every beat** — a persistent value-driven notch next to a decaying
//! flash, both off one pulse. This is the smallest closed loop that exercises the
//! value domain (reduce → value → drive → visible). Centred on the world origin,
//! sweeping symmetrically about centre.
//!
//! (`motion.step` — the bundled reduce+apply — and `pulse.threshold` stay
//! registered with their own unit tests; the boot scene now composes the
//! primitives instead.)

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the pulse-loop scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let place = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let drive = g.add_node("motion.drive");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let beat = g.add_node("pulse.beat");
    let counter = g.add_node("pulse.counter");

    // Visible trunk: grid → move → tint → drive.in → strobe.in → output.
    for (n, col) in [
        (grid, 0.0),
        (place, 1.0),
        (tint, 2.0),
        (drive, 3.0),
        (strobe, 4.0),
        (output, 5.0),
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
        (grid, place),
        (place, tint),
        (tint, drive),
        (drive, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Value/pulse branch: grid → beat (the stream tells the metronome N). The
    // one pulse feeds counter.pulse (port 0) and strobe.pulse (port 1); the
    // counter's VALUE feeds drive.value (port 1). One beat, one reduced value,
    // two visible responses.
    g.connect(Edge {
        from: (grid, 0),
        to: (beat, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (beat, 0),
        to: (counter, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (beat, 0),
        to: (strobe, 1),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (counter, 0),
        to: (drive, 1),
        delayed: false,
    })
    .ok()?;
    // The three `pre` self-loops (what the editor auto-plumbs on drop): the
    // beat's cycle index, the counter's monotonic tick, and the strobe's glow.
    for (n, port) in [(beat, 1), (counter, 1), (strobe, 2)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    for (n, col, dy) in [(beat, 1.0, 220.0), (counter, 2.0, 220.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // A 4×3 grid of big, well-spaced dots, the grid centred on the origin.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.6);
    g.set_param(grid, "gap_y", 1.1);
    // Centred on the world origin. `dx` is pre-offset by half the sweep's zigzag
    // reach (N-1=4 counts · scale 0.5 = 2.0 → -1.0) so the beat-driven sweep
    // ping-pongs SYMMETRICALLY about centre instead of drifting off to one side.
    g.set_param(place, "dx", -1.0);
    g.set_param(place, "dy", 0.0);
    // A calm base colour so the white flash reads as a brighten, not a hue jump.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.35);
    g.set_param(tint, "b", 0.85);
    // The beat: ~0.7 Hz → a pulse a bit under once a second (and one on start —
    // the scene moves the moment it plays).
    g.set_param(beat, "period", 1.4);
    g.set_param(beat, "offset", 0.0);
    // The counter: each beat advances the count, ZIGZAGGING 0..4..0 (5 distinct
    // counts), emitting it as a value. The value PERSISTS between beats.
    g.set_param(counter, "count_max", 5.0);
    g.set_param(counter, "mode", 2.0); // Zigzag — a ping-pong sweep
    // The drive: value → X, scaled 0.5 (the ±1.0 sweep about the pre-offset
    // centre), added to the channel — the discrete sequencer notch.
    g.set_param(drive, "channel", 0.0); // X
    g.set_param(drive, "scale", 0.5);
    g.set_param(drive, "mode", 0.0); // Add
    // A punchy flash that fades over ~0.3 s. A firm size boost so the pulse is
    // unmistakable (kept below the level that saturates the dots into one solid
    // block).
    g.set_param(strobe, "decay", 0.88);
    g.set_param(strobe, "size_boost", 2.2);
    g.set_param(strobe, "flash_r", 1.0);
    g.set_param(strobe, "flash_g", 1.0);
    g.set_param(strobe, "flash_b", 1.0);
    g.set_param(strobe, "flash_amount", 0.95);
    Some(output)
}

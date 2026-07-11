//! The pulse-loop scene — the **sole scene of the default Motion document** (M2
//! pulse + value families). A `#[path]` sibling of `motion_state`, kept out of
//! it for the LOC cap.
//!
//! It proves the **pulse type AND the value domain** end to end with **TWO
//! independent value chains** driving two channels of the same grid — a discrete
//! one off a beat, a continuous one off an LFO (docs/Motion Nodes/06, 08, 09, 12):
//!
//! ```text
//! grid → move → tint → drive_x → drive_y → strobe → output
//!        grid → beat ⟲ → { counter.pulse, strobe.pulse }
//!               counter.out --pre--> counter.state ; counter.out → drive_x.value
//!        grid → lfo → map_range → drive_y.value
//!               strobe.out  --pre--> strobe.state
//! ```
//!
//! **Discrete chain (X)** — a beat reduced to a stepping value:
//! - **beat** (`pulse.beat`, doc 09): the metronome — emits the PULSE straight
//!   from the playhead, one beat per `period`. No transform channel anywhere
//!   (the earlier scene faked the clock by oscillating Rotation into a threshold
//!   — doc 09 §1's "clock hack"). Its cycle index rides a `pre` self-loop; the
//!   one pulse fans out to counter AND strobe.
//! - **counter** (`pulse.counter`, doc 12): the PURE reducer — each beat advances
//!   a persistent count that ZIGZAGS 0..N..0 and emits it as a **value** (never a
//!   channel). Its monotonic tick rides its own `pre` self-loop.
//! - **drive_x** (`motion.drive`, doc 12): routes the counter's length-1 value
//!   onto X (`value · scale`), BROADCAST to every dot → the whole grid slides a
//!   discrete notch. The sequencer sweep, COMPOSED from a value, not bundled.
//!
//! **Continuous chain (Y)** — a stateless oscillation reshaped and routed:
//! - **lfo** (`value.lfo`, doc 12): reads the grid for its count (12) and emits a
//!   length-N **value** field — a sine with a per-instance `phase_stagger`, so a
//!   travelling wave ripples across the dots (the ELEMENT-WISE path, next to X's
//!   broadcast). Stateless (reads the playhead).
//! - **map_range** (`value.map_range`, doc 12): the glue — remaps the LFO's raw
//!   `[-1,1]` to the `[-0.5,0.5]` world span Y actually wants (the Houdini `fit`).
//! - **drive_y** (`motion.drive`): routes that field onto Y, element-wise → each
//!   dot bobs at its own phase. The SAME node type as drive_x, a different channel.
//!
//! - **strobe** (`motion.strobe`): the SAME beat lights every dot to full glow —
//!   a size boost + white flash — that decays geometrically. Its own `pre` loop.
//!
//! The payoff: the grid **steps sideways in discrete beats WHILE rippling up and
//! down continuously**, and flashes on each beat — a broadcast value-notch and an
//! element-wise travelling wave, two chains through one grid. The value domain's
//! whole thesis (produce → reshape → route, composable) made visible.
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
    let drive_x = g.add_node("motion.drive");
    let drive_y = g.add_node("motion.drive");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let beat = g.add_node("pulse.beat");
    let counter = g.add_node("pulse.counter");
    let lfo = g.add_node("value.lfo");
    let map_range = g.add_node("value.map_range");

    // Visible trunk: grid → move → tint → drive_x → drive_y → strobe → output.
    // drive_x writes X, drive_y writes Y — the two channels compose (each copies
    // the other's column through and rewrites only its own).
    for (n, col) in [
        (grid, 0.0),
        (place, 1.0),
        (tint, 2.0),
        (drive_x, 3.0),
        (drive_y, 4.0),
        (strobe, 5.0),
        (output, 6.0),
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
        (tint, drive_x),
        (drive_x, drive_y),
        (drive_y, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Discrete branch: grid → beat (the stream tells the metronome N). The one
    // pulse feeds counter.pulse (port 0) and strobe.pulse (port 1); the counter's
    // VALUE feeds drive_x.value (port 1) — broadcast to every dot.
    for (from, to) in [
        ((grid, 0), (beat, 0)),
        ((beat, 0), (counter, 0)),
        ((beat, 0), (strobe, 1)),
        ((counter, 0), (drive_x, 1)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // Continuous branch: grid → lfo (count source) → map_range → drive_y.value
    // (port 1) — an element-wise travelling wave onto Y.
    for (from, to) in [
        ((grid, 0), (lfo, 0)),
        ((lfo, 0), (map_range, 0)),
        ((map_range, 0), (drive_y, 1)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // The three `pre` self-loops (what the editor auto-plumbs on drop): the
    // beat's cycle index, the counter's monotonic tick, and the strobe's glow.
    // The LFO and map_range are stateless → no feedback.
    for (n, port) in [(beat, 1), (counter, 1), (strobe, 2)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    for (n, col, dy) in [
        (beat, 1.0, 220.0),
        (counter, 2.0, 220.0),
        (lfo, 1.0, 440.0),
        (map_range, 2.0, 440.0),
    ] {
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
    // drive_x: the counter's length-1 value → X, scaled 0.5 (the ±1.0 sweep about
    // the pre-offset centre), BROADCAST to every dot — the discrete sequencer notch.
    g.set_param(drive_x, "channel", 0.0); // X
    g.set_param(drive_x, "scale", 0.5);
    g.set_param(drive_x, "mode", 0.0); // Add
    // The LFO: a slow (2 s) sine emitting a length-12 value field, staggered
    // 0.12 cycle per instance → a travelling wave rippling across the grid.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 2.0);
    g.set_param(lfo, "amplitude", 1.0); // raw [-1,1]
    g.set_param(lfo, "phase_stagger", 0.12);
    // map_range: reshape the raw [-1,1] into the ±0.5 world span Y wants (a bob
    // just under half the vertical gap of 1.1). Clamp on is harmless here.
    g.set_param(map_range, "in_lo", -1.0);
    g.set_param(map_range, "in_hi", 1.0);
    g.set_param(map_range, "out_lo", -0.5);
    g.set_param(map_range, "out_hi", 0.5);
    // drive_y: that field → Y, element-wise (each dot its own phase), added.
    g.set_param(drive_y, "channel", 1.0); // Y
    g.set_param(drive_y, "scale", 1.0);
    g.set_param(drive_y, "mode", 0.0); // Add
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

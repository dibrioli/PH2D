//! The pulse-loop scene — the **sole scene of the default Motion document** (M2
//! pulse + value families). A `#[path]` sibling of `motion_state`, kept out of
//! it for the LOC cap.
//!
//! It proves the **pulse type AND the value domain** end to end with **FOUR
//! independent value chains** driving four channels of the same grid — a
//! discrete beat-driven X, a sample-and-held Y, a per-element spatial Size, and
//! a value→pulse round-trip that ratchets Rotation
//! (docs/Motion Nodes/06, 08, 09, 12, 13, 14, 16):
//!
//! ```text
//! grid → move → tint → drive_x → drive_y → drive_size → drive_rot → strobe → output
//!        grid → beat ⟲ → { counter.pulse, strobe.pulse, sample_hold.pulse }
//!               counter.out --pre--> counter.state ; counter.out → drive_x.value
//!        grid → lfo → sample_hold ⟲ → map_range → drive_y.value
//!        grid → instance_field → size_range → drive_size.value
//!        instance_field × lfo_g → math → compare ⟲ → counter_r ⟲ → rot_range → drive_rot.value
//!               strobe.out  --pre--> strobe.state
//! ```
//!
//! **Discrete chain (X)** — a beat reduced to a stepping value:
//! - **beat** (`pulse.beat`, doc 09): the metronome — emits the PULSE straight
//!   from the playhead, one beat per `period`. No transform channel anywhere
//!   (the earlier scene faked the clock by oscillating Rotation into a threshold
//!   — doc 09 §1's "clock hack"). Its cycle index rides a `pre` self-loop; the
//!   one pulse fans out to counter, strobe AND sample_hold.
//! - **counter** (`pulse.counter`, doc 12): the PURE reducer — each beat advances
//!   a persistent count that ZIGZAGS 0..N..0 and emits it as a **value** (never a
//!   channel). Its monotonic tick rides its own `pre` self-loop.
//! - **drive_x** (`motion.drive`, doc 12): routes the counter's length-1 value
//!   onto X (`value · scale`), BROADCAST to every dot → the whole grid slides a
//!   discrete notch. The sequencer sweep, COMPOSED from a value, not bundled.
//!
//! **Sample-and-hold chain (Y)** — a continuous wave frozen into steps (doc 14):
//! - **lfo** (`value.lfo`, doc 13): reads the grid for its count (12) and emits a
//!   length-N **value** field — a sine with a per-instance `phase_stagger`, a
//!   travelling wave (the ELEMENT-WISE path, next to X's broadcast). Stateless.
//! - **sample_hold** (`pulse.sample_hold`, doc 14): SAMPLES the LFO on each beat's
//!   rising edge and HOLDS it between (the `sah~`), so the smooth wave becomes a
//!   STAIRCASE. Sequential — its held value rides its own `pre` self-loop. Closes
//!   the doc-09 canonical combo `LFO → SampleHold → drive`.
//! - **map_range** (`value.map_range`, doc 13): the glue — remaps the held raw
//!   `[-1,1]` to the `[-0.5,0.5]` world span Y wants (the Houdini `fit`).
//! - **drive_y** (`motion.drive`): routes that held field onto Y, element-wise →
//!   each dot steps to its own sampled level on the beat and holds between.
//!
//! **Spatial chain (Size)** — per-element variation minted from identity (doc 14):
//! - **instance_field** (`value.instance_field`, doc 14): mints a length-N Ramp
//!   (`0..1` by index) — the ONLY node that makes per-element variation a
//!   first-class value, from the instance's identity rather than a behaviour's
//!   private stagger.
//! - **size_range** (`value.map_range`): maps `0..1` → a modest visible size span.
//! - **drive_size** (`motion.drive`): SETs Size per dot → a static gradient across
//!   the grid (small → big), which the strobe then flashes.
//!
//! **Round-trip chain (Rotation)** — a value graph feeding the pulse graph (doc 16):
//! - **lfo_g** (`value.lfo`): a GLOBAL sine — its `in` is unconnected, so it is a
//!   length-1 field (the degenerate global value, doc 12), the shared modulator.
//! - **math** (`value.math`, doc 16): MULTIPLIES the per-element `instance_field`
//!   Ramp (length-N) by the global `lfo_g` (length-1) — the first two-field
//!   combiner, exercising the doc-12 `1→N` broadcast. The product is a field whose
//!   amplitude is GRADED by index: dot 0 stays at 0, the top dot swings full ±1.
//! - **compare** (`pulse.compare`, doc 16): the genuine value→pulse bridge (the
//!   dual of `sample_hold`) — fires a PULSE when a dot's value rises past `rise`,
//!   with Schmitt hysteresis down to `fall` (no chatter at the peak). Only dots
//!   whose graded amplitude clears the threshold ever fire — the upper half. It is
//!   SEQUENTIAL (its armed state rides a `pre` self-loop).
//! - **counter_r** (`pulse.counter`): each crossing advances a per-dot count
//!   (Wrap 0..8), turning the discrete pulses back into a value.
//! - **rot_range** (`value.map_range`): maps the count `0..8 → 0..90°`.
//! - **drive_rot** (`motion.drive`): ADDs those degrees onto Rotation → the firing
//!   dots RATCHET their tilt forward (and wrap); the still dots stay put.
//!
//! - **strobe** (`motion.strobe`): the SAME beat lights every dot to full glow —
//!   a size boost (multiplying the gradient) + white flash — decaying geometrically.
//!
//! The payoff: the grid **steps sideways in discrete beats, its dots jumping to
//! new held heights on each beat, over a fixed small-to-big size gradient, while
//! its upper dots ratchet their rotation on a faster global wave** — all flashing
//! on the beat. Broadcast, sample-and-hold, per-element AND a value→pulse
//! round-trip at once: the value domain's whole thesis (produce → combine → sample
//! → threshold → reshape → route, composable, continuous↔discrete) made visible.
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
    let drive_size = g.add_node("motion.drive");
    let drive_rot = g.add_node("motion.drive");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let beat = g.add_node("pulse.beat");
    let counter = g.add_node("pulse.counter");
    let lfo = g.add_node("value.lfo");
    let sample_hold = g.add_node("pulse.sample_hold");
    let map_range = g.add_node("value.map_range");
    let instance_field = g.add_node("value.instance_field");
    let size_range = g.add_node("value.map_range");
    // Round-trip (Rotation) chain: a global LFO, a two-field combiner, a
    // value→pulse comparator, its counter and the degree remap (doc 16).
    let lfo_g = g.add_node("value.lfo");
    let math = g.add_node("value.math");
    let compare = g.add_node("pulse.compare");
    let counter_r = g.add_node("pulse.counter");
    let rot_range = g.add_node("value.map_range");

    // Visible trunk: grid → move → tint → drive_x → drive_y → drive_size → strobe
    // → output. Each drive writes ONE channel (X / Y / Size) and copies the other
    // columns through, so the three compose.
    for (n, col) in [
        (grid, 0.0),
        (place, 1.0),
        (tint, 2.0),
        (drive_x, 3.0),
        (drive_y, 4.0),
        (drive_size, 5.0),
        (drive_rot, 6.0),
        (strobe, 7.0),
        (output, 8.0),
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
        (drive_y, drive_size),
        (drive_size, drive_rot),
        (drive_rot, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Discrete branch (X): grid → beat (the stream tells the metronome N). The one
    // pulse fans out to counter.pulse, strobe.pulse AND sample_hold.pulse; the
    // counter's VALUE feeds drive_x.value (port 1) — broadcast to every dot.
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
    // Sample-and-hold branch (Y): grid → lfo (count source); the LFO's continuous
    // travelling wave is SAMPLED on every beat (sample_hold.value ← lfo,
    // .pulse ← beat) and HELD between beats, then reshaped and driven onto Y. So Y
    // steps to a new held level on each beat instead of gliding — the doc-09
    // canonical combo `LFO → SampleHold → drive`.
    for (from, to) in [
        ((grid, 0), (lfo, 0)),
        ((lfo, 0), (sample_hold, 0)),
        ((beat, 0), (sample_hold, 1)),
        ((sample_hold, 0), (map_range, 0)),
        ((map_range, 0), (drive_y, 1)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // Spatial branch (Size): grid → instance_field mints a per-dot Ramp (0..1 by
    // index) — the ONLY node that mints per-element variation from identity —
    // reshaped to a visible size range and driven onto Size. A static size
    // gradient across the grid (small → big), which the strobe then flashes.
    for (from, to) in [
        ((grid, 0), (instance_field, 0)),
        ((instance_field, 0), (size_range, 0)),
        ((size_range, 0), (drive_size, 1)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // Round-trip branch (Rotation): the per-element `instance_field` Ramp is
    // MULTIPLIED by the global `lfo_g` (a length-1 field broadcast over the N dots,
    // doc-12 `1→N`) into a graded wave; `compare` fires a pulse when a dot's value
    // crosses the threshold (Schmitt); `counter_r` accumulates the crossings and
    // `rot_range` maps the count to degrees, driven onto Rotation. The continuous
    // value domain feeding the discrete pulse domain and back — the doc-16 round
    // trip. `instance_field` fans out here as well as to the Size chain.
    for (from, to) in [
        ((instance_field, 0), (math, 0)), // gradient → math.a (length-N)
        ((lfo_g, 0), (math, 1)),          // global wave → math.b (length-1, broadcast)
        ((math, 0), (compare, 0)),        // graded field → compare.value
        ((compare, 0), (counter_r, 0)),   // per-dot crossings → counter_r.pulse
        ((counter_r, 0), (rot_range, 0)), // per-dot count → rot_range.in
        ((rot_range, 0), (drive_rot, 1)), // degrees → drive_rot.value
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // The `pre` self-loops (what the editor auto-plumbs on drop): the beat's cycle
    // index, the two counters' monotonic ticks, the sample_hold's held value, the
    // compare's armed state, and the strobe's glow. The LFOs, map_ranges, math and
    // instance_field are stateless.
    for (n, port) in [
        (beat, 1),
        (counter, 1),
        (sample_hold, 2),
        (strobe, 2),
        (compare, 1),
        (counter_r, 1),
    ] {
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
        (sample_hold, 2.0, 440.0),
        (map_range, 3.0, 440.0),
        (instance_field, 1.0, 660.0),
        (size_range, 2.0, 660.0),
        (lfo_g, 1.0, 880.0),
        (math, 2.0, 880.0),
        (compare, 3.0, 880.0),
        (counter_r, 4.0, 880.0),
        (rot_range, 5.0, 880.0),
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
    // sample_hold has no params — it samples the LFO on each beat's rising edge
    // and holds between (the `sah~`). The Y wave becomes a STAIRCASE.
    // map_range: reshape the (now held) raw [-1,1] into the ±0.5 world span Y
    // wants (a step just under half the vertical gap of 1.1). Clamp on is harmless.
    g.set_param(map_range, "in_lo", -1.0);
    g.set_param(map_range, "in_hi", 1.0);
    g.set_param(map_range, "out_lo", -0.5);
    g.set_param(map_range, "out_hi", 0.5);
    // drive_y: that held field → Y, element-wise (each dot its own sampled level),
    // added. Y holds between beats and steps on them.
    g.set_param(drive_y, "channel", 1.0); // Y
    g.set_param(drive_y, "scale", 1.0);
    g.set_param(drive_y, "mode", 0.0); // Add
    // instance_field: a per-dot Ramp (0..1 across the 12 dots by index) — the
    // sanctioned per-element source. size_range maps 0..1 → a visible size span
    // (kept modest so the strobe's boost never saturates the dots into one block).
    g.set_param(instance_field, "mode", 1.0); // Ramp
    g.set_param(size_range, "in_lo", 0.0);
    g.set_param(size_range, "in_hi", 1.0);
    g.set_param(size_range, "out_lo", 0.3);
    g.set_param(size_range, "out_hi", 0.55);
    // drive_size: SET the size to the gradient (channel 3, mode Set), so each dot
    // has its own base size; the strobe multiplies this on the flash.
    g.set_param(drive_size, "channel", 3.0); // Size
    g.set_param(drive_size, "scale", 1.0);
    g.set_param(drive_size, "mode", 1.0); // Set
    // Round-trip (Rotation) chain params (doc 16). lfo_g: a FAST global sine (its
    // `in` unconnected → a length-1 global field) — a different rhythm from the
    // 1.4 s beat, so the rotation ratchet reads as its own clock.
    g.set_param(lfo_g, "wave", 0.0); // Sine
    g.set_param(lfo_g, "period", 0.5);
    g.set_param(lfo_g, "amplitude", 1.0); // raw [-1,1]
    // math: MULTIPLY the per-dot Ramp (0..1) by the global wave (±1) → a field
    // whose amplitude is graded by index (dot 0 = 0, top dot = full ±1).
    g.set_param(math, "op", 2.0); // Multiply
    // compare: fire on the rising crossing of 0.4, with hysteresis down to 0.2
    // (no chatter at the wave's peak). Only dots whose graded amplitude clears 0.4
    // ever fire — the upper half of the grid.
    g.set_param(compare, "rise", 0.4);
    g.set_param(compare, "fall", 0.2);
    g.set_param(compare, "edge", 0.0); // Rise
    // counter_r: each crossing advances a per-dot count, wrapping 0..8.
    g.set_param(counter_r, "count_max", 8.0);
    g.set_param(counter_r, "mode", 0.0); // Wrap
    // rot_range: map the 0..8 count to 0..90° of rotation.
    g.set_param(rot_range, "in_lo", 0.0);
    g.set_param(rot_range, "in_hi", 8.0);
    g.set_param(rot_range, "out_lo", 0.0);
    g.set_param(rot_range, "out_hi", 90.0);
    // drive_rot: ADD those degrees onto Rotation (channel 2) → the firing dots
    // ratchet their tilt forward and wrap; the still dots stay at 0.
    g.set_param(drive_rot, "channel", 2.0); // Rotation
    g.set_param(drive_rot, "scale", 1.0);
    g.set_param(drive_rot, "mode", 0.0); // Add
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

//! The pulse-loop scene — the **sole scene of the default Motion document** (M2
//! pulse family). A `#[path]` sibling of `motion_state`, kept out of it for the
//! LOC cap.
//!
//! It proves the **pulse type** end to end — the source→consumer pulse loop
//! (docs/Motion Nodes/06, 08, 09):
//!
//! ```text
//! grid → move → tint → step → strobe → output
//!        grid → beat ⟲ → { step.pulse, strobe.pulse }
//!               step.out   --pre--> step.state
//!               strobe.out --pre--> strobe.state
//! ```
//!
//! - **beat** (`pulse.beat`, docs/Motion Nodes/09): the metronome — it emits the
//!   PULSE straight from the playhead, one beat per `period`. No transform
//!   channel anywhere: the earlier scene faked this clock by oscillating the
//!   invisible Rotation channel into a threshold, which coupled two nodes
//!   through a channel selector and broke the moment either side was retuned
//!   (doc 09 §1 — the "clock hack"). There is no `channel` to trip over now.
//!   Its cycle index rides a `pre` self-loop (the family's producer-side edge
//!   discipline). The one pulse fans out to BOTH consumers below.
//! - **step** (`motion.step`, docs/Motion Nodes/08): each beat advances a
//!   persistent count that ZIGZAGS 0..N..0; `count · step` slides the whole
//!   grid a discrete notch in X — a sequencer sweep that STAYS between beats.
//!   Its monotonic tick rides its own `pre` self-loop.
//! - **strobe** (`motion.strobe`): the SAME pulse lights every dot to full glow —
//!   a size boost + a white flash — that then decays geometrically. Its envelope
//!   rides its own `pre` self-loop.
//!
//! The payoff of the whole family in one shot: the grid **steps to a new place
//! and flashes on every beat** — the step's persistent notch (state that lasts)
//! next to the strobe's momentary flash (an envelope that decays), both driven
//! by one pulse from one source. This is the smallest closed pulse loop
//! (source → reduce → consume → visible). As the sole scene it sits **centred on
//! the world origin**, sweeping symmetrically about centre.
//!
//! (`pulse.threshold` is no longer in the boot scene: it is a value→pulse
//! adapter for real signals — a spring settling, an audio level — not a clock.
//! It stays registered with its own unit tests.)

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the pulse-loop scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let place = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let step = g.add_node("motion.step");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let beat = g.add_node("pulse.beat");

    // Visible trunk: grid → move → tint → step.in → strobe.in → output.
    for (n, col) in [
        (grid, 0.0),
        (place, 1.0),
        (tint, 2.0),
        (step, 3.0),
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
        (tint, step),
        (step, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Beat branch: grid → beat (the stream only tells the metronome N). One
    // pulse fans out to BOTH consumers' pulse ports (input port 1 on each): the
    // step (persistent notch) and the strobe (decaying flash). One beat, two
    // responses — and no channel selector anywhere to decouple them.
    g.connect(Edge {
        from: (grid, 0),
        to: (beat, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (beat, 0),
        to: (step, 1),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (beat, 0),
        to: (strobe, 1),
        delayed: false,
    })
    .ok()?;
    // The three `pre` self-loops (what the editor auto-plumbs on drop): the
    // beat's cycle index, the step's monotonic tick, and the strobe's decaying
    // `glow`.
    g.connect(Edge {
        from: (beat, 0),
        to: (beat, 1),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (step, 0),
        to: (step, 2),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (strobe, 0),
        to: (strobe, 2),
        delayed: true,
    })
    .ok()?;
    g.set_pos(
        beat,
        Pos {
            x: COL_W,
            y: ROW_Y + 220.0,
        },
    );

    // A 4×3 grid of big, well-spaced dots, the grid centred on the origin.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.6);
    g.set_param(grid, "gap_y", 1.1);
    // Centred on the world origin. `dx` is pre-offset by half the step's zigzag
    // reach (N-1=4 counts · step 0.5 = 2.0 → -1.0) so the beat-driven sweep
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
    // The step: each beat slides the whole grid one notch in X, ZIGZAGGING
    // 0..4..0 (step 0.5 → a ±1.0 sweep about the pre-offset centre). The notch
    // PERSISTS between beats — the visible counterpart to the strobe's decay.
    g.set_param(step, "channel", 0.0); // X
    g.set_param(step, "step", 0.5);
    g.set_param(step, "count_max", 5.0); // 5 distinct counts (0..4)
    g.set_param(step, "mode", 2.0); // Zigzag (LimitMode index) — a ping-pong sweep
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

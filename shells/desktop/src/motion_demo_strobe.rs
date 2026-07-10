//! The pulse-loop scene of the default Motion document (M2 pulse family).
//! A `#[path]` sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! A third independent scene proving the **pulse type** end to end — the first
//! producer→consumer pulse loop in the engine (docs/Motion Nodes/06, 08):
//!
//! ```text
//! grid → tint → counter → strobe → output
//!               clock(rotation osc) → threshold ⟲ → { counter.pulse, strobe.pulse }
//!               counter.out --pre--> counter.state
//!               strobe.out  --pre--> strobe.state
//! ```
//!
//! - **clock**: an oscillator on the ROTATION channel, uniform (`phase_stagger =
//!   0`) so every dot sees the *same* rising signal — a global beat, not
//!   per-dot noise. Its output feeds only the threshold; the rotation it writes
//!   never reaches the visible path.
//! - **threshold** (`pulse.threshold`, Schmitt): fires a PULSE on the rising
//!   crossing, with hysteresis so the wobble at the trip point does not chatter.
//!   Its `armed` state rides a `pre` self-loop (the sequential-node convention).
//!   The one pulse fans out to BOTH consumers below.
//! - **counter** (`pulse.counter`, the REDUCER, docs/Motion Nodes/08): each pulse
//!   advances a persistent count that ZIGZAGS 0..N..0; the count · step slides the
//!   whole grid a discrete notch in X — a sequencer sweep that STAYS between
//!   beats. Its monotonic tick rides its own `pre` self-loop.
//! - **strobe** (`motion.strobe`): the SAME pulse lights every dot to full glow —
//!   a size boost + a white flash — that then decays geometrically. Its envelope
//!   rides its own `pre` self-loop.
//!
//! The payoff of the whole family in one shot: the grid **steps to a new place and
//! flashes on every beat** — the counter's persistent step (state that lasts) next
//! to the strobe's momentary flash (an envelope that decays), both driven by one
//! pulse. This is the smallest closed pulse loop with a reducer (produce → reduce →
//! consume → visible), at the top of the visible central band (both side panels
//! cover the corners), overlapping the grid rig — the bright flash reads on top.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row.
const ROW_Y: f32 = -520.0;
const COL_W: f32 = 220.0;

/// Author the pulse-loop scene into `g`; returns its Output node (a third sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let place = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let counter = g.add_node("pulse.counter");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let clock = g.add_node("motion.oscillator");
    let threshold = g.add_node("pulse.threshold");

    // Visible trunk: grid → move (up to the top of the visible band, off the
    // focus rig at the origin and the fountain at lower-left) → tint →
    // counter.in → strobe.in → output.
    for (n, col) in [
        (grid, 0.0),
        (place, 1.0),
        (tint, 2.0),
        (counter, 3.0),
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
        (tint, counter),
        (counter, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Clock branch: grid → clock(rotation osc) → threshold → strobe.pulse.
    // A diamond off the grid, so the clock sees the same instances the strobe
    // lights.
    g.connect(Edge {
        from: (grid, 0),
        to: (clock, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (clock, 0),
        to: (threshold, 0),
        delayed: false,
    })
    .ok()?;
    // pulse.threshold.out (PULSE) fans out to BOTH consumers' pulse ports (input
    // port 1 on each): the counter (reducer) and the strobe (envelope). One beat,
    // two responses.
    g.connect(Edge {
        from: (threshold, 0),
        to: (counter, 1),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (threshold, 0),
        to: (strobe, 1),
        delayed: false,
    })
    .ok()?;
    // The three `pre` self-loops (what the editor auto-plumbs on drop): the
    // threshold's latched `armed`, the counter's monotonic tick, and the strobe's
    // decaying `glow`.
    g.connect(Edge {
        from: (threshold, 0),
        to: (threshold, 1),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (counter, 0),
        to: (counter, 2),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (strobe, 0),
        to: (strobe, 2),
        delayed: true,
    })
    .ok()?;
    for (n, col) in [(clock, 1.0), (threshold, 2.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + 220.0,
            },
        );
    }

    // A 4×3 row of big, well-spaced dots. Both side panels (Hierarchy left,
    // Motion right) cover the viewport corners, so this sits in the visible
    // central band along the TOP, overlapping the grid rig — the bright flash
    // dominates on top of it.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.6);
    g.set_param(grid, "gap_y", 1.1);
    // Top-centre of the visible band. `dx` is pre-offset by half the counter's
    // zigzag reach (N-1=4 counts · step 0.5 = 2.0 → -1.0) so the beat-driven sweep
    // ping-pongs SYMMETRICALLY about centre instead of drifting off to one side.
    g.set_param(place, "dx", -1.0);
    g.set_param(place, "dy", 3.6);
    // A calm base colour so the white flash reads as a brighten, not a hue jump.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.35);
    g.set_param(tint, "b", 0.85);
    // The beat: a uniform sine on Rotation (never rendered — feeds the threshold
    // only). ~0.7 Hz → a pulse a bit under once a second.
    g.set_param(clock, "channel", 2.0); // Rotation
    g.set_param(clock, "wave", 0.0); // Sine
    g.set_param(clock, "amplitude", 1.0);
    g.set_param(clock, "frequency", 0.7);
    g.set_param(clock, "phase_stagger", 0.0); // uniform → global beat
    // Fire on the rising edge as the sine climbs past 0.5, with a hysteresis
    // band down to 0.0 so the crest's wobble cannot re-fire.
    g.set_param(threshold, "channel", 2.0); // read Rotation
    g.set_param(threshold, "rise", 0.5);
    g.set_param(threshold, "fall", 0.0);
    g.set_param(threshold, "edge", 0.0); // Rise
    // The counter (reducer): each beat slides the whole grid one notch in X,
    // ZIGZAGGING 0..4..0 (step 0.5 → a ±1.0 sweep about the pre-offset centre). The
    // step PERSISTS between beats — the visible counterpart to the strobe's decay.
    g.set_param(counter, "channel", 0.0); // X
    g.set_param(counter, "step", 0.5);
    g.set_param(counter, "count_max", 5.0); // 5 distinct counts (0..4)
    g.set_param(counter, "mode", 2.0); // Zigzag (LimitMode index) — a ping-pong sweep
    // A punchy flash that fades over ~0.3 s. A firm size boost so the pulse is
    // unmistakable against the grid rig it sits over (kept below the level that
    // saturates the dots into one solid block).
    g.set_param(strobe, "decay", 0.88);
    g.set_param(strobe, "size_boost", 2.2);
    g.set_param(strobe, "flash_r", 1.0);
    g.set_param(strobe, "flash_g", 1.0);
    g.set_param(strobe, "flash_b", 1.0);
    g.set_param(strobe, "flash_amount", 0.95);
    Some(output)
}

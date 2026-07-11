//! The value-domain demo — the **sole scene of the default Motion document**. A
//! deliberately SMALL scene (one grid, ~11 nodes) so each new node reads on its
//! own. A `#[path]` sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! It isolates the two newest value-domain nodes (doc 17) on ONE grid: a
//! `value.switch` ROUTES the grid's Size between two source patterns as a selector
//! cycles, and a `pulse.on_change` FLASHES the grid the instant the pattern flips:
//!
//! ```text
//! grid → tint → drive_size → strobe → output
//!        grid → instance_field(Ramp)   ─┐
//!        grid → instance_field(Random) ─┤
//!        lfo ───────────────────────────┴→ switch → size_range → drive_size.value
//!                                          switch → on_change ⟲ → strobe.pulse
//! ```
//!
//! - **switch** (`value.switch`, doc 17): the multiplexer — its `select` is a
//!   VALUE (here a slow `value.lfo` that cycles `0 ↔ 1`), so it routes `in0` (an
//!   ordered Ramp gradient) or `in1` (a Random scatter) onto the wire. The Size
//!   pattern toggles between a small→big gradient and a random spread every ~1 s.
//! - **on_change** (`pulse.on_change`, doc 17): the change detector — it fires a
//!   PULSE the tick the switched value STEPS to something new (the complement of
//!   `pulse.compare`'s threshold crossing). The `motion.strobe` turns that into a
//!   white flash, so the grid lights up exactly ON each flip. (The strobe flashes
//!   COLOUR only — `size_boost = 0` — so Size stays the pure switched signal.)
//!
//! The payoff: the grid's size pattern **flips between order and randomness** and
//! **flashes on the flip** — routing (switch) and change-detection (on_change), the
//! last two pieces of the value/pulse vocabulary, side by side. See docs/Motion
//! Nodes/12 (value), 14 (instance_field), 17 (switch+on_change). The metronome,
//! counter, sample-hold, math, compare and per-channel drives of the earlier
//! scenes stay registered (drop them in the editor); the boot scene shows the
//! newest pair in isolation.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the value-domain scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let tint = g.add_node("motion.tint");
    let drive_size = g.add_node("motion.drive");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let ramp = g.add_node("value.instance_field");
    let rand = g.add_node("value.instance_field");
    let lfo = g.add_node("value.lfo");
    let switch = g.add_node("value.switch");
    let size_range = g.add_node("value.map_range");
    let on_change = g.add_node("pulse.on_change");

    // Visible trunk: grid → tint → drive_size → strobe → output. `drive_size`
    // writes the Size channel from the switched pattern; `strobe` flashes colour
    // from `on_change`.
    for (n, col) in [
        (grid, 0.0),
        (tint, 1.0),
        (drive_size, 2.0),
        (strobe, 3.0),
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
        (grid, tint),
        (tint, drive_size),
        (drive_size, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Value branches. Two `instance_field` sources (an ordered Ramp, a Random
    // scatter) feed the `switch`; the `lfo` is its animated `select`, cycling the
    // routed input. The switched pattern reshapes onto Size AND feeds `on_change`,
    // which fires the strobe the tick the pattern flips.
    for (from, to) in [
        ((grid, 0), (ramp, 0)),             // grid → instance_field(Ramp) (count)
        ((grid, 0), (rand, 0)),             // grid → instance_field(Random) (count)
        ((lfo, 0), (switch, 0)),            // lfo → switch.select
        ((ramp, 0), (switch, 1)),           // Ramp → switch.in0
        ((rand, 0), (switch, 2)),           // Random → switch.in1
        ((switch, 0), (size_range, 0)),     // switched pattern → size_range.in
        ((size_range, 0), (drive_size, 1)), // → drive_size.value
        ((switch, 0), (on_change, 0)),      // switched pattern → on_change.value
        ((on_change, 0), (strobe, 1)),      // pattern flip → strobe.pulse
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // The `pre` self-loops (what the editor auto-plumbs on drop): the on_change's
    // previous value and the strobe's glow. The lfo, switch, size_range and the two
    // instance_fields are stateless.
    for (n, port) in [(on_change, 1), (strobe, 2)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    for (n, col, dy) in [
        (ramp, 1.0, 220.0),
        (rand, 1.0, 340.0),
        (lfo, 1.0, 460.0),
        (switch, 2.0, 340.0),
        (size_range, 3.0, 220.0),
        (on_change, 3.0, 460.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // A 3×4 grid of well-spaced dots, centred on the origin.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.4);
    g.set_param(grid, "gap_y", 1.1);
    // A calm blue base so the white flash reads as a brighten, not a hue jump.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.35);
    g.set_param(tint, "b", 0.85);
    // The two switched sources: an ordered Ramp (small→big by index) and a Random
    // scatter (both length-N over the 12 dots).
    g.set_param(ramp, "mode", 1.0); // Ramp
    g.set_param(rand, "mode", 2.0); // Random
    g.set_param(rand, "seed", 7.0);
    // lfo as the `select`: a slow (2 s) sine kept in [0, 1] (amplitude 0.5, offset
    // 0.5) so it rounds to 0 or 1 — cycling `in0 ↔ in1` at every zero crossing
    // (~once a second). Unconnected `in` → a length-1 GLOBAL select (the whole
    // grid switches together).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 2.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    // size_range: map the 0..1 pattern to a visible size span (min 0.3 so even the
    // smallest dot stays visible).
    g.set_param(size_range, "in_lo", 0.0);
    g.set_param(size_range, "in_hi", 1.0);
    g.set_param(size_range, "out_lo", 0.3);
    g.set_param(size_range, "out_hi", 0.6);
    // drive_size: SET the size to the switched pattern (channel 3, mode Set).
    g.set_param(drive_size, "channel", 3.0); // Size
    g.set_param(drive_size, "scale", 1.0);
    g.set_param(drive_size, "mode", 1.0); // Set
    // on_change: fire when a dot's switched value moves more than a hair (epsilon
    // ignores float dither; the flip between Ramp and Random is a big step).
    g.set_param(on_change, "epsilon", 0.02);
    // strobe: a COLOUR-ONLY flash (size_boost 0 → Size stays the pure switched
    // signal) fading over ~0.3 s, lit white on the blue base.
    g.set_param(strobe, "decay", 0.88);
    g.set_param(strobe, "size_boost", 0.0);
    g.set_param(strobe, "flash_r", 1.0);
    g.set_param(strobe, "flash_g", 1.0);
    g.set_param(strobe, "flash_b", 1.0);
    g.set_param(strobe, "flash_amount", 0.9);
    Some(output)
}

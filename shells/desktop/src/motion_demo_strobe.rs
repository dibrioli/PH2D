//! The M4.2 simulation demo — the **default Motion document**: two small,
//! side-by-side continuum-media simulations. On the LEFT a **jelly** hangs and
//! wobbles from a sliding pin (`motion.soft_body`, shape-matching); on the RIGHT a
//! **ripple field** radiates concentric waves from a driven centre (`motion.wave`).
//! Two independent scenes (each its own `motion.output` sink — the bridge composes
//! several into one draw), kept deliberately small so each new node reads on its
//! own. A `#[path]` sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! ```text
//! LEFT  (jelly):   soft_body → tint(magenta) → output   lfo_anchor → soft_body.anchor_x
//!                  soft_body --pre--> soft_body.state
//! RIGHT (ripple):  wave      → tint(cyan)    → output    lfo_drive  → wave.drive
//!                  wave --pre--> wave.state
//! ```
//!
//! - **soft-body** (`motion.soft_body`, doc 22): a shape-matching mesh pinned at its
//!   top row; gravity sags it and the `anchor_x` `value.lfo` slides the pin, so the
//!   whole body **wobbles like jelly** and springs back to shape.
//! - **wave** (`motion.wave`, doc 22): the discrete wave equation; the `drive`
//!   `value.lfo` oscillates the centre cell, so ripples **propagate outward** as
//!   expanding rings of larger dots.
//!
//! The payoff: two continuum simulations on the `pre` self-loop — one a deformable
//! body, one a propagating field — each driven by the value domain, on one legible
//! canvas (the discrete-agent counterpart is doc 21's rope + flock). See
//! docs/Motion Nodes/22 (soft-body + wave). The whole value/pulse vocabulary + the
//! other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
/// The two scenes' card rows in graph space (stacked, so the editor reads cleanly).
const JELLY_ROW: f32 = 0.0;
const WAVE_ROW: f32 = 320.0;

/// Author both sim scenes into `g`; returns their Output nodes (the sinks), the
/// jelly's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let jelly_out = build_soft_body_scene(g)?;
    let wave_out = build_wave_scene(g)?;
    Some(vec![jelly_out, wave_out])
}

/// LEFT: a jelly pinned at a sliding anchor. Returns its Output node.
fn build_soft_body_scene(g: &mut Graph) -> Option<NodeId> {
    let body = g.add_node("motion.soft_body");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(body, 0.0), (tint, 1.0), (output, 2.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: JELLY_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: JELLY_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (body, 0),
        to: (tint, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (tint, 0),
        to: (output, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (body, 0),
        delayed: false,
    })
    .ok()?; // → anchor_x
    g.connect(Edge {
        from: (body, 0),
        to: (body, 2),
        delayed: true,
    })
    .ok()?; // pre state loop

    // A 4×4 jelly, gooey (low stiffness) so it wobbles, hung on the left (x≈−6).
    g.set_param(body, "rows", 4.0);
    g.set_param(body, "cols", 4.0);
    g.set_param(body, "spacing", 0.6);
    g.set_param(body, "gravity", 9.0);
    g.set_param(body, "stiffness", 0.35);
    g.set_param(body, "pin", 1.0); // top row pinned to the anchor
    // A warm magenta blob.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.25);
    g.set_param(tint, "b", 0.70);
    // lfo → anchor_x: a slow (3 s) sine about x = −6, ±1.2 → the pin slides and the
    // jelly wobbles. Unconnected `in` → a length-1 GLOBAL value (one anchor).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 3.0);
    g.set_param(lfo, "amplitude", 1.2);
    g.set_param(lfo, "offset", -6.0);
    Some(output)
}

/// RIGHT: a ripple field driven at its centre. Returns its Output node.
fn build_wave_scene(g: &mut Graph) -> Option<NodeId> {
    let wave = g.add_node("motion.wave");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(wave, 0.0), (tint, 1.0), (output, 2.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: WAVE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: WAVE_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (wave, 0),
        to: (tint, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (tint, 0),
        to: (output, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (wave, 0),
        delayed: false,
    })
    .ok()?; // → drive
    g.connect(Edge {
        from: (wave, 0),
        to: (wave, 1),
        delayed: true,
    })
    .ok()?; // pre state loop

    // A 13×13 ripple grid, centred on the right (x≈+6).
    g.set_param(wave, "rows", 13.0);
    g.set_param(wave, "cols", 13.0);
    g.set_param(wave, "spacing", 0.45);
    g.set_param(wave, "speed", 0.35);
    g.set_param(wave, "damping", 0.02);
    g.set_param(wave, "center_x", 6.0);
    g.set_param(wave, "center_y", 0.0);
    // A cool cyan field.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.85);
    g.set_param(tint, "b", 0.95);
    // lfo → drive: a fast (1.5 s) sine oscillating the centre cell → continuous
    // ripples radiate outward.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 1.5);
    g.set_param(lfo, "amplitude", 1.0);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}

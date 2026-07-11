//! The M3 distribution demo — the **default Motion document**: on the LEFT a **radial
//! array** of rings that rotates (`motion.distribute_radial`); on the RIGHT a **180-point
//! Voronoi** cloud that relaxes into a honeycomb (`motion.voronoi`, animated) and is
//! made symmetric by a **mirror** (`motion.mirror`). The right scene deliberately runs
//! Voronoi at a stress count (180, `relax` live) so the parallelised Lloyd is visible
//! in the app — the perf fix in the flesh. Two independent scenes (each its own
//! `motion.output` sink — the bridge composes several into one draw), kept small so
//! each new node reads on its own. A `#[path]` sibling of `motion_state`, kept out for
//! the LOC cap.
//!
//! ```text
//! LEFT  (radial): distribute_radial → move(−6) → tint(amber) → output   lfo → spin
//! RIGHT (mirror): voronoi → mirror → move(+6) → tint(cyan)  → output     lfo → relax
//! ```
//!
//! - **distribute_radial** (`motion.distribute_radial`, doc 25): rings of points; the
//!   `spin` `value.lfo` swings the whole array round.
//! - **voronoi + mirror** (`motion.mirror`, doc 25): the 180-point Voronoi relaxes via
//!   Lloyd (parallelised — smooth at this count), then `motion.mirror` reflects it
//!   across the vertical axis into a symmetric honeycomb (360 dots).
//!
//! See docs/Motion Nodes/25 (radial + mirror). The whole value/pulse vocabulary + the
//! other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
const RADIAL_ROW: f32 = 0.0;
const MIRROR_ROW: f32 = 320.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the radial
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let radial_out = build_radial_scene(g)?;
    let mirror_out = build_mirror_scene(g)?;
    Some(vec![radial_out, mirror_out])
}

/// LEFT: a rotating radial array. Returns its Output node.
fn build_radial_scene(g: &mut Graph) -> Option<NodeId> {
    let radial = g.add_node("motion.distribute_radial");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(radial, 0.0), (mv, 1.0), (tint, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: RADIAL_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: RADIAL_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (radial, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
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
        to: (radial, 0),
        delayed: false,
    })
    .ok()?; // → spin

    // 48 points over 3 rings, on the left half.
    g.set_param(radial, "count", 48.0);
    g.set_param(radial, "rings", 3.0);
    g.set_param(radial, "radius", 2.5);
    g.set_param(radial, "inner", 0.6);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // lfo → spin: a slow (6 s) sine, ±180° → the array swings round and back.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 180.0);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}

/// RIGHT: a 180-point Voronoi (the perf stress) relaxing, then mirrored. Returns its
/// Output node.
fn build_mirror_scene(g: &mut Graph) -> Option<NodeId> {
    let voronoi = g.add_node("motion.voronoi");
    let mirror = g.add_node("motion.mirror");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [
        (voronoi, 0.0),
        (mirror, 1.0),
        (mv, 2.0),
        (tint, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: MIRROR_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: MIRROR_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (voronoi, 0),
        to: (mirror, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mirror, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
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
        to: (voronoi, 0),
        delayed: false,
    })
    .ok()?; // → relax

    // 180 Voronoi points (the count that used to drop to 20fps) in a 4×4 domain,
    // Lloyd re-run every frame via the animated `relax` — now parallelised.
    g.set_param(voronoi, "count", 180.0);
    g.set_param(voronoi, "width", 4.0);
    g.set_param(voronoi, "height", 4.0);
    g.set_param(voronoi, "iterations", 8.0);
    g.set_param(mirror, "axis", 0.0); // Vertical → a symmetric honeycomb (360 dots)
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    // lfo → relax: a 5 s sine about 0.5, ±0.5 → relax ∈ [0, 1] (organise/dissolve).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 5.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

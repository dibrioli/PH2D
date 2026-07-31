//! The **Lloyd/JFA voronoi** scene (`PH2D_GPU_COOK_DEMO=11`, ADR-0139) — the
//! smoke for the engine's first [`GpuAlgorithm`], and for the cap that fell
//! with it.
//!
//! Every other GPU scene answers a question about a per-element MAP or about
//! STATE. This one answers a third: **does a node whose cook is a multi-pass
//! ALGORITHM run on the device?** `motion.voronoi` relaxes a random cloud
//! toward a centroidal Voronoi tessellation — the even, hexagonal packing that
//! stippling and blue-noise want — and on the CPU that is a linear nearest
//! scan per grid sample, `O(iterations · res² · count)`. That cost is why the
//! node shipped with **the smallest cap in the library: 600 points**. The
//! device runs jump flooding instead (count-independent per iteration), so the
//! cap is now 165 000 — and this scene stands at 20 000, **33× past the number
//! the artist could reach yesterday**.
//!
//! ```text
//!   voronoi ─> falloff ─> color_ramp ─> scale ─> output
//!      ▲                      ▲
//!   value.lfo             value.attribute (falloff → t)
//!   (relax 0..1)
//! ```
//!
//! What each wire proves:
//! - **The LFO into `relax`** is the whole point of the node and the WORST case
//!   for its cost: relax is per-frame, so the entire relaxation re-cooks every
//!   frame (8 iterations of seed → flood → reduce → move). Watch the cloud
//!   organise into a honeycomb and dissolve back into white noise, once per
//!   period. Both paths were MEASURED at this exact count and iteration count:
//!   the device costs **3,02 ms/frame** (`gpu-cook/tests/gpu_voronoi.rs::
//!   how_far_does_the_lloyd_scale`) and the CPU reference costs **2160 ms**
//!   (`motion-voronoi::cost_probe`) — **715×**, and the reason the scene can
//!   animate `relax` at all.
//! - **`value.attribute` projecting `falloff` into the ramp's `t`** colours by
//!   radius, which is what makes the packing LEGIBLE: at `relax = 0` the colour
//!   rings are ragged (white noise clumps), at `relax = 1` they are smooth and
//!   evenly spaced. A confetti of index-ordered colour would look identical
//!   either way, and the smoke would prove nothing.
//! - The chain is a **DAG, not a line** (the falloff feeds both the ramp's
//!   stream port and the attribute projection), which the plan claims whole.
//!
//! Under `PH2D_GPU_COOK=1` (the default) nothing here is a boundary: the
//! algorithm, the falloff, the projection, the ramp and the scale all run on
//! the device, zero readback. Auto-plays on tool entry.
//!
//! ⚠️ **The count is a RENDER budget, not the cook's** — the same lesson the
//! zone scene paid for (Enio, 2026-07-20: *"profunda queda de fps"* at 262 k).
//! The cook's ceiling here is the node's own cap; raise `count` to see it.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// How many points the scene relaxes. 20 000 — **33× the pre-ADR-0139 cap of
/// 600**, which is the demonstration; and measured at **3,02 ms/frame** with
/// the relaxation re-cooking every frame, so it leaves the frame budget intact
/// while the artist zooms and scrubs. (The device's own ceiling is the node's
/// cap: 165 000 at 19,4 ms — raise `count` to see it.)
pub(super) const DEMO_POINTS: f32 = 20_000.0;

/// **The breathing honeycomb** (`PH2D_GPU_COOK_DEMO=11`) — the ready-to-smoke
/// scene for the Lloyd/JFA algorithm on the device (ADR-0139). Returns the sink.
pub(super) fn build_gpu_voronoi_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let voronoi = g.add_node("motion.voronoi");
    g.set_param(voronoi, "count", DEMO_POINTS);
    // A wide field so the packing has room to read as a packing.
    g.set_param(voronoi, "width", 24.0);
    g.set_param(voronoi, "height", 14.0);
    g.set_param(voronoi, "seed", 1.0);
    // 8 = the node's default, near-converged. Every one of these re-runs per
    // frame while `relax` animates — which is exactly what is being smoked.
    g.set_param(voronoi, "iterations", 8.0);

    // The relaxation, played: unconnected, an LFO is ONE global oscillation (a
    // length-1 value field), and the device reads its row 0 — the broadcast
    // shape. amplitude 0.5 + offset 0.5 sweeps `relax` across the full 0..1.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);

    // Radial value, so the colour says something about the geometry.
    let falloff = g.add_node("motion.falloff");
    g.set_param(falloff, "radius", 12.0);

    // The ADR-0136 projection: the `falloff` column, as the ramp's `t`. The
    // key is the graph's text channel (`value.attribute::ATTR_KEY`), spelled
    // here the way the strobe scene spells it — the shell depends on the
    // registry, not on each node crate.
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, "attr", "falloff");

    let ramp = g.add_node("motion.color_ramp");
    // Ice — a cold field reads as structure. The gradient IS the `ramp` text param (doc 85);
    // the `preset` param is gone, and setting it made `validate` refuse the graph.
    g.set_text_param(
        ramp,
        "ramp",
        ph2d_color::serialize_gradient(&ph2d_color::GradientPreset::Ice.ramp()),
    );

    // Grains, not blobs: unit quads at this count would be a solid sheet.
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.05);

    let out = g.add_node("motion.output");

    for (i, n) in [voronoi, falloff, ramp, scale, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 160.0,
            },
        );
    }
    // The two drivers sit one row apart: the clock above, the projection below.
    g.set_pos(lfo, Pos { x: 80.0, y: 20.0 });
    g.set_pos(attr, Pos { x: 270.0, y: 330.0 });

    for (from, to, port) in [
        (lfo, voronoi, 0), // → relax
        (voronoi, falloff, 0),
        (falloff, ramp, 0), // the stream
        (falloff, attr, 0), // …and the same stream, projected
        (attr, ramp, 1),    // → t
        (ramp, scale, 0),
        (scale, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

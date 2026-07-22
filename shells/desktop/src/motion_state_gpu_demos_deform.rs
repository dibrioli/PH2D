//! The **deformer** ready-to-smoke document (`PH2D_GPU_COOK_DEMO=12`) — the
//! demonstration surface of the whole-stream reduction channel
//! (`ph2d_nodegraph::reduce_meta`).
//!
//! Sibling of `motion_state_gpu_demos.rs`, which is at the HR-18 cap.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// The **breathing CLOTH** (`PH2D_GPU_COOK_DEMO=12`) — the ready-to-smoke
/// document for the DEFORMER family: `grid(700×700) → bend → twist → output`,
/// both deformers driven by their own LFO, **490.000 instances 100% on the
/// device**.
///
/// ## What you should see
///
/// A dense sheet that **curls into a barrel and uncurls** (the bend, wrapping the
/// sheet's X extent onto an arc) while **winding and unwinding around its centre**
/// (the twist, turning the rim by the full angle and the middle not at all). The
/// two motions are on different periods, so the sheet never repeats a pose for a
/// long while. Nothing should pop, tear, or snap flat.
///
/// ## What it is actually demonstrating
///
/// Until this channel existed, **every deformer in the library was CPU-only** —
/// not because the map was hard, but because element `i`'s answer depends on a
/// number that does not exist until every element has been looked at (the sheet's
/// X extent; the rim radius). The census could not even see the hole, because its
/// corpus contained no deformer.
///
/// ⚠️ **The two deformers are CHAINED on purpose, and that is the sharp part.**
/// The twist's rim radius must be measured on the **bent** sheet, not on the flat
/// one it started as — the reduction is folded at the twist's own stage, over the
/// stream the bend just produced. Hoisting the folds to the top of the cook would
/// look identical on a single deformer and be wrong here: as the bend curls the
/// sheet, its silhouette shrinks, so a stale rim radius would make the twist
/// overshoot exactly when the barrel is tightest. If the sheet ever winds harder
/// as it curls, that ordering broke.
///
/// ⚠️ **Both amounts are BROADCAST**, not per-element: a bare `value.lfo` is one
/// number, held across the whole field (`ColumnAccess::ReadBroadcast`). That is
/// what makes the sheet move as one cloth rather than 490.000 independent quads.
pub(super) fn build_gpu_deform_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // 700 × 700 = 490.000 — the same population as the sim demos, so the GPU
    // meter is comparable across scenes.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    // Unit quads edge to edge: the sheet reads as cloth, and a deformation of it
    // reads as a SHEET bending rather than as points scattering.
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // The bend: the sheet's X extent (~700 units) wraps onto an arc. 150° is
    // most of a half-barrel at full amount — enough that the far edges clearly
    // come toward the viewer, short of the self-overlap that reads as noise.
    let bend = g.add_node("motion.bend");
    g.set_param(bend, "angle", 150.0);
    g.set_param(bend, "pivot_x", 0.0);
    g.set_param(bend, "pivot_y", 0.0);

    // The twist: the rim turns the full angle, the centre not at all.
    let twist = g.add_node("motion.twist");
    g.set_param(twist, "angle", 120.0);
    g.set_param(twist, "pivot_x", 0.0);
    g.set_param(twist, "pivot_y", 0.0);

    // Two LFOs on DIFFERENT periods (SECONDS per cycle — `value.lfo` speaks
    // period, not frequency), so the pair does not beat in lockstep and the sheet
    // keeps finding poses. Amplitude 1 with offset 0 is bipolar, which is what
    // makes the bend curl BOTH ways rather than only up.
    let curl = g.add_node("value.lfo");
    g.set_param(curl, "period", 9.0);
    g.set_param(curl, "amplitude", 1.0);
    let wind = g.add_node("value.lfo");
    g.set_param(wind, "period", 14.0);
    g.set_param(wind, "amplitude", 1.0);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, bend, twist, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 140.0,
            },
        );
    }
    g.set_pos(curl, Pos { x: 200.0, y: 320.0 });
    g.set_pos(wind, Pos { x: 390.0, y: 320.0 });

    for (from, to, port) in [
        (grid, bend, 0u16),
        (curl, bend, 1),
        (bend, twist, 0),
        (wind, twist, 1),
        (twist, out, 0),
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

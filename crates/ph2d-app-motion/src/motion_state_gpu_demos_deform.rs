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

/// The **breathing LENS** (`PH2D_GPU_COOK_DEMO=13`) — the ready-to-smoke document
/// for the `Sum` half of the reduction channel: `grid(700×700) → spherize →
/// output`, the bulge/pinch driven by an LFO, **490.000 instances 100% on the
/// device**.
///
/// ## What you should see
///
/// A dense sheet whose **middle swells toward you and then sucks back in**, like a
/// magnifying glass sliding over it and reversing — the centre spreads out at the
/// peak of the bulge, pulls into a knot at the pinch, and the rim of the lens
/// stays put throughout (the quadratic falloff makes the edge seamless). The
/// motion is smooth and radial; nothing should shear or tear.
///
/// ## What it is actually demonstrating
///
/// The lens is centred on the layout's **centroid**, and the centroid is
/// `Σ P / n` — two `Sum` reductions over the whole stream (`cx`, `cy`), divided by
/// the count. This is the FIRST node whose kernel reads **more than one**
/// whole-stream reduction, and the first to fold with `Sum` rather than `Max`
/// (`bend`/`twist` are bit-exact `Max`; a `Sum` is a documented ε — see
/// `ph2d_nodegraph::reduce_meta`). If the swell ever centres somewhere other than
/// the middle of the sheet, one of the two sums is wrong.
///
/// ⚠️ **`amount` is broadcast at index 0**, matching the CPU node, which reads
/// `vals.first()` — one bulge factor for the whole lens even under a per-element
/// field. The LFO here is length-1, so it is the whole field's breath.
pub(super) fn build_gpu_spherize_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // A large lens (most of the 700-unit sheet) so the bulge is the whole field
    // breathing, not a small bump in the middle.
    let sph = g.add_node("motion.spherize");
    g.set_param(sph, "radius", 320.0);

    // The breath: bipolar `amount` (swells then pinches). Amplitude 0.9 keeps the
    // pinch short of the r=0 singularity the node guards but never wants to sit on.
    let breath = g.add_node("value.lfo");
    g.set_param(breath, "period", 6.0);
    g.set_param(breath, "amplitude", 0.9);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, sph, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 200.0,
                y: 140.0,
            },
        );
    }
    g.set_pos(breath, Pos { x: 280.0, y: 320.0 });

    for (from, to, port) in [(grid, sph, 0u16), (breath, sph, 1), (sph, out, 0)] {
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

/// The **billowing FLAG** (`PH2D_GPU_COOK_DEMO=14`) — the ready-to-smoke document
/// for the widest reduction consumer: `grid(700×700) → four_point_warp → output`,
/// a perspective corner-pin driven by an LFO, **490.000 instances 100% on the
/// device**.
///
/// ## What you should see
///
/// A dense sheet that **tips into perspective and flattens back**, like a flag or
/// a projector keystone: the far corners pull in and up while the near ones stay,
/// straight lines staying straight (that is the perspective divide, not a bilinear
/// bow). At the LFO's zero the sheet is flat; at the peak it is fully pinned into
/// the quad. Nothing should bow or shear along the way.
///
/// ## What it is actually demonstrating
///
/// The warp normalises every element to `(u,v)` within the layout's **bounding
/// box**, and the box is four reductions — `Min`/`Max` over `P.x` and `P.y`. This
/// is the FIRST node to read **four** whole-stream reductions and the first to use
/// `Min` (bend/twist are `Max`, spherize is `Sum`). Because `Min`/`Max` are
/// bit-exact, the box the device computes is the same bits as the CPU's — the only
/// ε is the homography arithmetic. If the sheet ever pins to the wrong rectangle,
/// one of the four box reductions is wrong.
///
/// ⚠️ **`warp` is broadcast at index 0** (the CPU's `first()`), so the LFO is the
/// whole flag's single billow factor, not a per-element field.
pub(super) fn build_gpu_four_point_warp_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // A skewed target quad (world units, offsets from the bbox corners): the top
    // edge pulls in and the right side lifts — a keystone that reads as depth.
    // The sheet is ~700 units, so offsets in the low hundreds are a strong pin.
    let fpw = g.add_node("motion.four_point_warp");
    for (name, v) in [
        ("tl_dx", 160.0f32),
        ("tl_dy", 60.0),
        ("tr_dx", -120.0),
        ("tr_dy", 180.0),
        ("br_dx", -40.0),
        ("br_dy", 40.0),
        ("bl_dx", 40.0),
        ("bl_dy", 0.0),
    ] {
        g.set_param(fpw, name, v);
    }

    // The billow: `warp` scales every corner 0→1 and back. Unipolar (amplitude
    // 0.5, offset 0.5) so it swings between flat (0) and fully pinned (1) rather
    // than inverting the quad.
    let billow = g.add_node("value.lfo");
    g.set_param(billow, "period", 7.0);
    g.set_param(billow, "amplitude", 0.5);
    g.set_param(billow, "offset", 0.5);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, fpw, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 200.0,
                y: 140.0,
            },
        );
    }
    g.set_pos(billow, Pos { x: 280.0, y: 320.0 });

    for (from, to, port) in [(grid, fpw, 0u16), (billow, fpw, 1), (fpw, out, 0)] {
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

/// The spinning **MANDALA** (`PH2D_GPU_COOK_DEMO=15`) — the ready-to-smoke
/// document for the COUNT-CHANGING deformer: `grid(200×200) → move → kaleidoscope
/// → output`, a source blob fanned into 12 mirrored slices and spun by an LFO.
/// **40.000 source elements × 12 = 480.000 on the device.**
///
/// ## What you should see
///
/// A twelve-fold mandala that **rotates**, its slices meeting as mirror images
/// (the true kaleidoscope fold — every other slice is reflected). The source is a
/// dense square parked off the centre, so each slice is a wedge and the twelve
/// wedges close into a ring that turns as the spin LFO sweeps. Nothing should
/// tear at the seams between slices (that is the mirror agreeing at the boundary).
///
/// ## What it is actually demonstrating
///
/// This is the FIRST count-changing node on the device that READS its template.
/// Its output is `n · segments` elements — a fan-out the CPU pays in full — and
/// each output element is the source rotated into its slice. On the GPU it is a
/// `StreamOp::SourceRows` kernel: it reads the source `P` at row `i % n`
/// (`ColumnAccess::SourceRead`, the length-decouple that lets a kernel read a
/// template shorter than its own dispatch), writes the rotated position and the
/// template row, and the sequencer gathers every other column onto each slice —
/// so `size`/`tint`/`id` are duplicated exactly like the CPU's `dup_n`. If a slice
/// ever shows the wrong copy or lands at the origin, the source read or the fan-out
/// broke.
///
/// ⚠️ **`spin` is broadcast at index 0** — one global rotation for the whole
/// mandala, the CPU's `first()`.
pub(super) fn build_gpu_kaleidoscope_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // 200 × 200 = 40.000 source elements; × 12 slices = 480.000 on the device.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 200.0);
    g.set_param(grid, "cols", 200.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // Park the source off the centre so each slice is a WEDGE and the twelve close
    // into a ring (a grid centred on the pivot would just overlap itself).
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 260.0);
    g.set_param(mv, "dy", 0.0);

    let kal = g.add_node("motion.kaleidoscope");
    g.set_param(kal, "segments", 12.0);
    g.set_param(kal, "reflect", 1.0); // the true kaleidoscope fold (Dₙ)

    // The spin: `value.lfo` in degrees, a slow full sweep so the mandala turns.
    // Amplitude 180 + offset 180 keeps it monotonic-looking across a period.
    let spin = g.add_node("value.lfo");
    g.set_param(spin, "period", 12.0);
    g.set_param(spin, "amplitude", 180.0);
    g.set_param(spin, "offset", 180.0);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, mv, kal, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 140.0,
            },
        );
    }
    g.set_pos(spin, Pos { x: 440.0, y: 320.0 });

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, kal, 0),
        (spin, kal, 1),
        (kal, out, 0),
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

/// **O ORGANISMO** (`PH2D_GPU_COOK_DEMO=16`) — the ONE scene that smokes the whole
/// reduction channel end to end: a single dense blob is fanned into a twelve-fold
/// body and then **domed, arced, wound, and tipped into perspective**, each
/// deformation folding its own whole-stream reduction over the live stream the
/// previous one produced. `grid(200×200) → move → kaleidoscope → spherize → bend →
/// twist → four_point_warp → output`. **40.000 source × 12 = 480.000 instances,
/// six kernels deep, 100% on the device.**
///
/// ## What you should see
///
/// A twelve-petalled organism that never sits still: it **spins** (the fan), its
/// centre **breathes toward you and sucks back** (the dome), the whole ring **curls
/// into a shell and uncurls** (the arc), it **winds and unwinds around its axis**
/// (the wind), and the entire churning mass **tips into keystone perspective and
/// flattens back** (the pin). The five motions are on five PRIME periods
/// (13·6·9·11·7 s), so the composite pose cycle is enormous — you will not see it
/// repeat. The seams between the twelve slices stay welded (the mirror agreeing),
/// the dome stays centred (the centroid is honest), and nothing pops, tears, or
/// snaps flat.
///
/// ## What it is actually demonstrating — everything the line built, at once
///
/// - **The count-changing SourceRead** (`kaleidoscope`): 40.000 source rows fanned
///   to 480.000 on the device, each slice reading the template `P` at `i % n`
///   (`ColumnAccess::SourceRead`) and the sequencer gathering `size`/`tint`/`id`
///   onto every copy. Every deformer downstream runs on the *fanned* 480.000.
/// - **All three reduce operators, chained**: `spherize` folds the centroid (two
///   `Sum`s, ÷ count), `bend` folds the X extent (`Max`), `twist` folds the rim
///   radius (`Max`), `four_point_warp` folds the bounding box (`Min`+`Max` ×4).
///   Bit-exact where `Max`/`Min`, a documented ε where `Sum` or the homography
///   (`ph2d_nodegraph::reduce_meta`).
///
/// ⚠️ **The reductions are folded PER STAGE, over the live stream, and that is the
/// sharp part of the whole channel.** The dome centres on the centroid of the
/// *fanned* mandala; the arc wraps the X extent of the *domed* mandala; the wind
/// turns the rim of the *arced* one; the pin frames the box of the *wound* one.
/// Hoisting any fold to the top of the cook would look plausible on one deformer
/// and be wrong here — as each stage reshapes the silhouette, a stale extent /
/// radius / box would make the next stage overshoot exactly when the form is
/// tightest. If the organism ever winds harder as it curls, or the dome drifts off
/// the axis of symmetry, or the pin frames a stale rectangle, that per-stage
/// ordering broke.
///
/// ⚠️ **Every amount is BROADCAST at row 0** (`ColumnAccess::ReadBroadcast`, the
/// CPU's `first()`): five length-1 LFOs, each one number held across all 480.000
/// elements, so the body moves as ONE organism rather than as half a million
/// independent quads. Bisect the device against the reference with `PH2D_GPU_COOK=0`
/// — the two paths must agree within the channel's ε.
pub(super) fn build_gpu_deform_organism_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // 200 × 200 = 40.000 source elements; the kaleidoscope fans it to 480.000 —
    // the same population as every other deform scene, so the GPU meter compares.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 200.0);
    g.set_param(grid, "cols", 200.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);

    // Park the source off the pivot so each slice is a WEDGE and the twelve close
    // into a ring (the fan of a grid centred on the pivot just overlaps itself).
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 260.0);
    g.set_param(mv, "dy", 0.0);

    // 1. THE FAN — count-changing (×12), the true kaleidoscope fold (Dₙ mirror).
    let kal = g.add_node("motion.kaleidoscope");
    g.set_param(kal, "segments", 12.0);
    g.set_param(kal, "reflect", 1.0);

    // 2. THE DOME — centred on the mandala's centroid (two Sum reductions ÷ count).
    // radius 380 covers the fanned ring (outer radius ≈ 360), so the whole body
    // breathes rather than a bump in the middle.
    let sph = g.add_node("motion.spherize");
    g.set_param(sph, "radius", 380.0);

    // 3. THE ARC — the domed mandala's X extent (Max) wraps onto a barrel.
    let bend = g.add_node("motion.bend");
    g.set_param(bend, "angle", 150.0);
    g.set_param(bend, "pivot_x", 0.0);
    g.set_param(bend, "pivot_y", 0.0);

    // 4. THE WIND — the arced form's rim radius (Max) turns; the centre does not.
    let twist = g.add_node("motion.twist");
    g.set_param(twist, "angle", 120.0);
    g.set_param(twist, "pivot_x", 0.0);
    g.set_param(twist, "pivot_y", 0.0);

    // 5. THE PIN — the wound form's bounding box (Min/Max ×4) → keystone quad.
    let fpw = g.add_node("motion.four_point_warp");
    for (name, v) in [
        ("tl_dx", 160.0f32),
        ("tl_dy", 60.0),
        ("tr_dx", -120.0),
        ("tr_dy", 180.0),
        ("br_dx", -40.0),
        ("br_dy", 40.0),
        ("bl_dx", 40.0),
        ("bl_dy", 0.0),
    ] {
        g.set_param(fpw, name, v);
    }

    // Five LFOs on five PRIME periods (seconds/cycle) so no two beat in lockstep
    // and the composite pose never repeats within any smoke session. `spin` and
    // `keystone` are unipolar (a sweep / a flat↔pinned swing); the three shape
    // amounts are bipolar (they deform BOTH ways).
    let spin = g.add_node("value.lfo"); // the fan turns
    g.set_param(spin, "period", 13.0);
    g.set_param(spin, "amplitude", 180.0);
    g.set_param(spin, "offset", 180.0);
    let dome = g.add_node("value.lfo"); // the centre swells/pinches
    g.set_param(dome, "period", 6.0);
    g.set_param(dome, "amplitude", 0.9);
    let arc = g.add_node("value.lfo"); // the ring curls both ways
    g.set_param(arc, "period", 9.0);
    g.set_param(arc, "amplitude", 1.0);
    let wind = g.add_node("value.lfo"); // it winds and unwinds
    g.set_param(wind, "period", 11.0);
    g.set_param(wind, "amplitude", 1.0);
    let keystone = g.add_node("value.lfo"); // it tips into perspective
    g.set_param(keystone, "period", 7.0);
    g.set_param(keystone, "amplitude", 0.5);
    g.set_param(keystone, "offset", 0.5);

    let out = g.add_node("motion.output");

    // The spine runs left to right; the five LFOs sit under the stage each drives.
    for (i, n) in [grid, mv, kal, sph, bend, twist, fpw, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 60.0 + i as f32 * 150.0,
                y: 140.0,
            },
        );
    }
    for (i, n) in [spin, dome, arc, wind, keystone].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 360.0 + i as f32 * 150.0,
                y: 320.0,
            },
        );
    }

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, kal, 0),
        (spin, kal, 1),
        (kal, sph, 0),
        (dome, sph, 1),
        (sph, bend, 0),
        (arc, bend, 1),
        (bend, twist, 0),
        (wind, twist, 1),
        (twist, fpw, 0),
        (keystone, fpw, 1),
        (fpw, out, 0),
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

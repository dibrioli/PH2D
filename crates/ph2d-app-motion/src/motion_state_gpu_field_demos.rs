//! The **field** ready-to-smoke documents (`PH2D_GPU_COOK_DEMO=17..22`) — the
//! demonstration surface of the `field.*` family: the ordinal `field.index_range`
//! band, the spatial `field.box`/`radial_sweep`/`combine` masks, the `field.remap`
//! transfer, and the A1-gpu **Curve** contour (=22), all cooked on the device.
//!
//! Sibling of `motion_state_gpu_demos.rs`, which is at the HR-18 cap.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// The **FIELD family** smoke (`PH2D_GPU_COOK_DEMO=17`): `grid(512×512) →
/// field.index_range → tint(Solid) → output` — **262.144 instances**, every node
/// kernel-covered, so `PH2D_GPU_COOK=1` runs it 100 % GPU-resident (parity-proven
/// bit-exact at 25.6k, `field_index_range_kernel_matches_the_cpu_within_epsilon`).
///
/// `field.index_range` writes the multiplicative `falloff` mask keyed by ORDINAL
/// `i/(count−1)` — row-major here — and the Solid tint lerps the sprites' white
/// toward a saturated red BY that mask. The result is a **horizontal band of the
/// middle rows** glowing red, the rest white, the seam rows fading through the
/// soft edge. It is the one mask a spatial `motion.falloff` cannot draw: it
/// selects by RANK, not position — "the middle third of the clones", the stagger
/// a grid + circle can never reach. Auto-plays on tool entry like every boot doc.
pub(super) fn build_gpu_field_index_range_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    // 512 × 512 = 262.144 unit quads, gap 1.0 tiling them edge-to-edge into a
    // dense field (the panel demo's scale — legibly a grid, substantial on GPU).
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    // gap 0.024 makes the 512-wide field ~12 units (centred on origin), so it
    // FRAMES at the default zoom instead of being a 512-unit wall you must zoom
    // out to see. The 512-row count keeps the ordinal tilt at ~1/512 (a fine band
    // edge, not a staircase).
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    // Shrink the unit quads to dots BEFORE the field writes `falloff` (absent =>
    // reads its 1.0 identity => the scale is UNIFORM): 0.018 < the 0.024 gap, so
    // the dots stay distinct instead of tiling into the sheet that hid the band.
    // (motion.scale eased by falloff = 1 is the full amount — the STRENGTH cluster.)
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    let field = g.add_node("field.index_range");
    // The middle ~half of the ordinal range, soft-edged — a horizontal band of
    // rows (index is row-major), unmistakably index-keyed rather than spatial.
    g.set_param(field, "start", 0.25);
    g.set_param(field, "end", 0.75);
    g.set_param(field, "soft", 0.08);
    g.set_param(field, "curve", 2.0); // Smooth edges
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.25);
    g.set_param(tint, "b", 0.15);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, tint, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [(grid, scale), (scale, field), (field, tint), (tint, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The **spatial** field smoke (`PH2D_GPU_COOK_DEMO=18`): `grid(512×512) →
/// motion.scale → field.box → tint(Solid) → output` — **262.144 instances**,
/// 100 % GPU-resident. The sibling of `=17`: where `field.index_range` masks by
/// ORDINAL (and so tilts ~1 row on a grid), `field.box` masks by POSITION. A
/// wide-`width`, thin-`height` box is a **razor-horizontal blue band** — flat by
/// y, no tilt — the spatial answer to "make it perfectly horizontal". Same
/// frame-on-load sizing as `=17`; auto-plays on tool entry.
pub(super) fn build_gpu_field_box_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    let field = g.add_node("field.box");
    // Wider than the ~12-unit field in x (half 15 > 6, so the band spans the full
    // width), thin in y (a ~5-unit-tall band, half 2.5) with a soft edge: a flat
    // horizontal stripe, razor-level because the mask reads y, not rank.
    g.set_param(field, "width", 30.0);
    g.set_param(field, "height", 5.0);
    g.set_param(field, "soft", 1.5);
    // Tilted by the Rotation param — the field ROTATES (set it to 0 for the
    // razor-horizontal band). Proof that spatial fields carry an orientation, the
    // C4D/Cavalry model, HR-5-safe via the shared parabolic-sine basis.
    g.set_param(field, "rotation", 30.0);
    g.set_param(field, "curve", 2.0); // Smooth edges
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, tint, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [(grid, scale), (scale, field), (field, tint), (tint, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The **angular** field smoke (`PH2D_GPU_COOK_DEMO=20`): `grid(512×512) →
/// motion.scale → field.radial_sweep → tint(Solid) → output` — **262.144
/// instances**, 100 % GPU-resident. The third spatial field and the second the
/// canvas gizmo drives: where `field.box` masks by an axis-aligned rectangle, this
/// masks by ANGLE about a centre. `repetitions = 6` tiles a 30° wedge six times ⇒
/// a **six-pointed blue star** (a fan / radar) — the Cavalry Sweep signature, and
/// the picture that a rectangle cannot make. It is the HR-5 pseudo-angle sector on
/// the device (no `atan2`), taken as a `min` against the radial clip. Same frame-on-load
/// sizing as `=17`/`=18`; auto-plays on tool entry.
pub(super) fn build_gpu_field_radial_sweep_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    let field = g.add_node("field.radial_sweep");
    // A 30° wedge repeated 6× (60° period) ⇒ six beams with six gaps: a star. Radius
    // 7 covers most of the ~12-unit grid but leaves a disk edge visible; a soft edge
    // feathers both the arc and the rim. This is the shape a box CANNOT express — the
    // reason the angular field exists.
    g.set_param(field, "radius", 7.0);
    g.set_param(field, "start_angle", 0.0);
    g.set_param(field, "end_angle", 30.0);
    g.set_param(field, "repetitions", 6.0);
    g.set_param(field, "soft", 0.2);
    g.set_param(field, "curve", 2.0); // Smooth edges
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, tint, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [(grid, scale), (scale, field), (field, tint), (tint, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The **composition** smoke (`PH2D_GPU_COOK_DEMO=19`) — the field family's whole
/// thesis on the device: TWO fields fanned off one `motion.scale`, blended by
/// `field.combine`. `field.index_range` draws a horizontal ORDINAL band, `field.box`
/// a thin VERTICAL band; `Max` (union) lights a red **cross** wherever EITHER field
/// is on — the vertical arm razor-straight (spatial), the horizontal arm faintly
/// tilted (ordinal), so one picture shows both kinds of field AND that they compose.
/// **262.144 instances**, the whole fan-out claimed fully-GPU. Same frame-on-load
/// sizing as `=17`/`=18`; auto-plays on tool entry.
pub(super) fn build_gpu_field_combine_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    // Branch A: a horizontal ordinal band (the middle ~third of the rows).
    let ir = g.add_node("field.index_range");
    g.set_param(ir, "start", 0.36);
    g.set_param(ir, "end", 0.64);
    g.set_param(ir, "soft", 0.05);
    // Branch B: a thin VERTICAL band (narrow in x, tall enough to span y) — the
    // razor-straight spatial arm.
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 3.0);
    g.set_param(bx, "height", 40.0);
    g.set_param(bx, "soft", 0.8);
    // Union of the two masks ⇒ a cross.
    let cmb = g.add_node("field.combine");
    g.set_param(cmb, "mode", 6.0); // Max (union)
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.25);
    g.set_param(tint, "b", 0.15);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, ir, bx, cmb, tint, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 60.0 + i as f32 * 160.0,
                y: 120.0,
            },
        );
    }
    // grid → scale, then scale FANS OUT to both fields; combine takes a = ir, b = box.
    for (a, b, port) in [
        (grid, scale, 0),
        (scale, ir, 0),
        (scale, bx, 0),
        (ir, cmb, 0),
        (bx, cmb, 1),
        (cmb, tint, 0),
        (tint, out, 0),
    ] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, port),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The **remap** smoke (`PH2D_GPU_COOK_DEMO=21`): `grid(512×512) → motion.scale →
/// field.box → field.remap(Quantize) → tint(Solid) → output` — **262.144 instances**,
/// 100 % GPU-resident. The keystone of the field family's D1 factoring: `field.box`
/// paints a soft ramp SMALLER than the grid (so the grid rides the FULL `[0,1]` range,
/// with a `0` white frame outside the box), and `field.remap` QUANTIZES it into **four
/// discrete bands** — nested-square topographic contours the box alone cannot make. It
/// is the C4D Remapping tab as a downstream node (every spatial field defers its remap
/// here), and it REWRITES the mask (a transfer function), not multiplies. Three levels
/// (not four) so the bands are maximally spaced — the tint is one blue at three
/// opacities over white, and adjacent opacities blur, so `{white, half-blue, full-blue}`
/// reads far crisper than four close shades. Same frame-on-load sizing; auto-plays.
pub(super) fn build_gpu_field_remap_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    // A soft box SMALLER than the ~12-unit grid (half-extent 4.5 vs the grid's ±6.1),
    // so the grid sees the FULL ramp: falloff 1 at the centre down to 0 at the box edge,
    // then a flat 0 (white) frame out to the grid edge. Without this the grid rode only
    // the top of the ramp ([0.53, 1]) and Quantize barely banded — the "faint" smoke.
    let field = g.add_node("field.box");
    g.set_param(field, "width", 9.0);
    g.set_param(field, "height", 9.0);
    g.set_param(field, "soft", 4.5); // = half the extent — a linear ramp centre→edge
    g.set_param(field, "curve", 0.0); // Linear ramp, so the bands are evenly spaced
    // Quantize the ramp into 3 discrete levels {0, ½, 1} — the widest possible jumps, so
    // the bands are maximally distinct: a white frame, a half-blue ring, a full-blue
    // core. (4+ levels crowd the opacities of one colour into shades that blur.)
    let remap = g.add_node("field.remap");
    g.set_param(remap, "contour", 3.0); // Quantize
    g.set_param(remap, "steps", 3.0);
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, remap, tint, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [
        (grid, scale),
        (scale, field),
        (field, remap),
        (remap, tint),
        (tint, out),
    ] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// Demo `=22`: the **Curve contour** (A1). Same soft box as `=21`, but the remap runs a
/// non-monotonic **tent** curve `(0,0)→(0.5,1)→(1,0)` authored in the text param — so the
/// mask is white at the centre (falloff 1 → curve 0), full-blue at MID radius (falloff 0.5
/// → curve 1), and white again at the edge (falloff 0 → curve 0): a blue RING. No ramp and
/// no Quantize can make a ring — it is the unmistakable sign the custom transfer took. The
/// kernel declines mode 4, so this cooks on the CPU (A1-gpu bakes the LUT).
pub(super) fn build_gpu_field_curve_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    let field = g.add_node("field.box");
    g.set_param(field, "width", 9.0);
    g.set_param(field, "height", 9.0);
    g.set_param(field, "soft", 4.5); // = half the extent — a linear ramp centre→edge
    g.set_param(field, "curve", 0.0); // Linear ramp, so the curve reads the full [0,1]
    let remap = g.add_node("field.remap");
    g.set_param(remap, "contour", 4.0); // Curve
    // A tent: peak at the middle of the ramp. Linear legs so the ring is crisp.
    g.set_text_param(remap, "curve", "c1 0:0:L 0.5:1:L 1:0:L");
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered tint mode
    g.set_param(tint, "r", 0.16);
    g.set_param(tint, "g", 0.62);
    g.set_param(tint, "b", 0.94);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, remap, tint, out]
        .into_iter()
        .enumerate()
    {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [
        (grid, scale),
        (scale, field),
        (field, remap),
        (remap, tint),
        (tint, out),
    ] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

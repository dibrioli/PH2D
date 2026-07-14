//! Gates for the **spatula** — Flatten / Scrape / Fill, the plane family
//! (`docs/Painter/18_plano_sculpt_relevo.md` §7, Wave 2).
//!
//! A child of [`super`] so the fixtures are shared and cannot drift into two subtly different canvases.
//!
//! The gate that matters most in this file is the first one, and it is the reason the plan calls the tilted
//! fit *"a diferença entre uma ferramenta e um bug"*: **a spatula that flattens toward a HORIZONTAL plane
//! does not flatten a hillside, it cuts a crater into it.** Every other property here (only-down, only-up,
//! the Offset's bite, the volume) is true of the horizontal version too — which is exactly why a suite
//! without `flatten_on_a_pure_ramp_is_a_no_op` would be all green against the broken tool.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::sync::Arc;

/// Modes, by their segmented index.
const SMOOTH: u8 = 0;
const FLATTEN: u8 = 2;
const SCRAPE: u8 = 3;
const FILL: u8 = 4;

/// A canvas whose relief is a **pure tilted ramp** — no bumps, no marks, nothing to remove. Returns the
/// tool, the layer, and the ramp.
fn ramp_canvas(size: u32, gx: f32, gy: f32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    let (mut t, layer, _) = sculpt_canvas(size);
    let n = (size * size) as usize;
    let ramp: Vec<f32> = (0..n)
        .map(|i| {
            let x = (i as u32 % size) as f32;
            let y = (i as u32 / size) as f32;
            // Centred so the ramp stays well inside ±H_CEIL at every texel.
            0.5 + gx * (x - size as f32 * 0.5) + gy * (y - size as f32 * 0.5)
        })
        .collect();
    t.heights.insert(layer, Arc::new(ramp.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    (t, layer, ramp)
}

/// The largest absolute change over the whole canvas.
fn max_change(before: &[f32], after: &[f32]) -> f32 {
    before
        .iter()
        .zip(after)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max)
}

/// **THE gate of Wave 2: Flatten on a pure slope does nothing.**
///
/// A tilted least-squares plane fitted to a tilted surface *is that surface*, so there is no deviation to
/// remove and the spatula must come away clean. The naive spatula — fitting a HORIZONTAL plane at the local
/// mean — instead scrapes the uphill half of its own footprint away and fills the downhill half in, gouging
/// a crater into the hillside at full Strength. It would pass every other gate in this file.
///
/// The tolerance is a real number, not a fudge: the fit is `f64`, the surface is `f32`, and the plan's own
/// promise is that Flatten *follows the surface* — one part in a thousand of a paint-load is far below the
/// relief epsilon the light can even see.
///
/// **Mutation that must bleed:** return `gx: 0.0, gy: 0.0` from `ph2d_painter_brush::plane::PlaneFit::solve`
/// (the horizontal fit).
#[test]
fn flatten_on_a_pure_ramp_is_a_no_op() {
    let size = 200u32;
    // A slope of 0.006 loads/px over a 24-px brush is ~0.15 loads across the footprint — a real hillside,
    // several times the relief the light resolves.
    let (mut t, layer, ramp) = ramp_canvas(size, 0.006, -0.004);
    arm_sculpt(&mut t, FLATTEN, 0.5, 1.0);
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);

    let after = heights_of(&t, layer);
    let worst = max_change(&ramp, &after);
    assert!(
        worst < 1e-3,
        "Flatten gouged a pure slope by up to {worst:.4} paint-loads. The plane is being fitted \
         HORIZONTALLY: it sits at the footprint's mean height, so the uphill half of every dab stands above \
         it and is scraped away while the downhill half is filled in. On a flat surface that bug is \
         invisible; on a painting — where nothing is flat — it destroys the form and leaves the brush-marks."
    );
}

/// **The spatula takes the marks off the hillside and leaves the hillside.**
///
/// The product gate: a ramp with brush-marks on it. Scrape must remove the marks (the deviation from the
/// local plane collapses) while the hill itself survives (the slope is still there afterwards). Either assert
/// alone is satisfiable by a kernel that is wrong in the other direction; together they describe the tool.
///
/// ## The slope is measured ACROSS the stroke, and that is the whole gate
///
/// The first version of this test measured the slope *along* the stroke and stayed **green under the
/// horizontal fit** — which is a fact worth writing down rather than a tolerance to tighten. Along its own
/// path, a horizontal spatula *reconstructs the slope by accident*: each dab's plane sits at the mean height
/// of its own footprint, that mean tracks the hillside, and the per-texel target is the coverage-weighted
/// mean of every plane that touched the texel — so the moving average of level planes climbs the hill.
///
/// Nothing does that **perpendicular** to the stroke. There is only one row of dabs, every one of their
/// planes is at the same height, and the cross-slope is simply erased: the spatula ploughs a level trough
/// through the hillside. So the gate puts the slope where the bug lives, and asks for it back.
///
/// (The marks are +0.20 on a quarter of the texels — mean 0.05 — so the cross-slope must clear that to be
/// visible at all: at `gy = 0.02` the hill rises 0.24 across the brush's own radius, five times the marks.
/// A gentler hill would hide inside the very texture the tool is supposed to be removing.)
///
/// **Mutation that must bleed:** the horizontal fit (`gx: 0.0, gy: 0.0` from `PlaneFit::solve`) — the
/// roughness still drops (it always does; that is why the roughness assert alone proves nothing), and the
/// cross-slope collapses from 0.020 to ~0.005.
#[test]
fn the_scrape_takes_the_marks_off_the_hillside_and_leaves_the_hill() {
    // 140 px so the steep ramp stays inside ±H_CEIL at every texel (0.5 ± 0.02·70).
    let size = 140u32;
    let gy = 0.02_f32;
    let (mut t, layer, _) = ramp_canvas(size, 0.0, gy); // the hill rises ACROSS the stroke
    let n = (size * size) as usize;
    let base = heights_of(&t, layer);
    let marked: Vec<f32> = (0..n)
        .map(|i| {
            let x = i as u32 % size;
            let y = i as u32 / size;
            base[i] + if (x + y).is_multiple_of(4) { 0.20 } else { 0.0 }
        })
        .collect();
    t.heights.insert(layer, Arc::new(marked.clone()));
    t.sync_relief_flags();

    arm_sculpt(&mut t, SCRAPE, 0.5, 1.0);
    let mid = f32::from(70u8);
    drag(&mut t, &[[40.0, mid], [70.0, mid], [100.0, mid]]); // straight across the hill's contour
    let after = heights_of(&t, layer);

    // Roughness = mean |h − ramp| along the scraped row: how much of the marks is left.
    let roughness = |h: &[f32]| -> f64 {
        let y = 70usize;
        let (mut s, mut c) = (0.0, 0.0);
        for x in 50..90usize {
            let i = y * size as usize + x;
            s += f64::from((h[i] - base[i]).abs());
            c += 1.0;
        }
        s / c
    };
    let before_rough = roughness(&marked);
    let after_rough = roughness(&after);
    assert!(
        after_rough < before_rough * 0.5,
        "Scrape left the brush-marks standing: roughness {before_rough:.4} → {after_rough:.4}"
    );

    // …and the HILL is still a hill. Row means (to average the marks out), 8 px apart across the stroke.
    let row_mean = |h: &[f32], y: usize| -> f64 {
        let (mut s, mut c) = (0.0, 0.0);
        for x in 55..85usize {
            s += f64::from(h[y * size as usize + x]);
            c += 1.0;
        }
        s / c
    };
    let slope = (row_mean(&after, 78) - row_mean(&after, 70)) / 8.0;
    assert!(
        (slope - f64::from(gy)).abs() < 5e-3,
        "Scrape flattened the HILL, not just the marks on it: across the scraped band the surface now rises \
         {slope:.5} loads/px, but the hillside it was scraping rises {gy:.5}. This is the horizontal-fit \
         bug — and note it is only visible ACROSS the stroke: along its own path a level spatula \
         reconstructs the slope by accident, because the moving average of its level planes climbs the hill."
    );
}

/// **Scrape only lowers. Fill only raises. Flatten does both.** One kernel, one sign clamp — so one gate,
/// three directions, and the three-way comparison is what makes it impossible to satisfy by accident.
///
/// **Mutation that must bleed:** drop the `SculptMode::Scrape => delta.min(0.0)` arm (Scrape then fills the
/// valleys too, and its "raised" count goes non-zero).
#[test]
fn scrape_only_lowers_fill_only_raises_and_flatten_does_both() {
    let size = 200u32;
    let path = [[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]];

    // (lowered, raised) texel counts for a verb, over a bumpy relief.
    let run = |mode: u8| -> (u32, u32) {
        let (mut t, layer, before) = sculpt_canvas(size);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        drag(&mut t, &path);
        let after = heights_of(&t, layer);
        let (mut down, mut up) = (0u32, 0u32);
        for (a, b) in before.iter().zip(&after) {
            let d = b - a;
            if d < -1e-5 {
                down += 1;
            } else if d > 1e-5 {
                up += 1;
            }
        }
        (down, up)
    };

    let (fl_down, fl_up) = run(FLATTEN);
    let (sc_down, sc_up) = run(SCRAPE);
    let (fi_down, fi_up) = run(FILL);

    assert!(
        fl_down > 100 && fl_up > 100,
        "Flatten moved the relief in only one direction ({fl_down} down, {fl_up} up) — it is supposed to \
         pull toward the plane from BOTH sides"
    );
    assert_eq!(
        sc_up, 0,
        "Scrape RAISED {sc_up} texels. A spatula takes material off; it does not fill the valleys on its \
         way past."
    );
    assert!(sc_down > 100, "Scrape lowered nothing ({sc_down} texels)");
    assert_eq!(
        fi_down, 0,
        "Fill LOWERED {fi_down} texels. It is Scrape's mirror: it puts paint into the hollows and leaves \
         the ridges standing."
    );
    assert!(fi_up > 100, "Fill raised nothing ({fi_up} texels)");
}

/// **The Offset gives the spatula its bite.**
///
/// At Offset 0 the plane sits ON the fitted surface, so Scrape only takes off what stands above the local
/// slope. Sink the plane and it digs: strictly more material comes off, everywhere.
///
/// **Mutation that must bleed:** drop the `+ offset` from the plane target in `render_sculpt`.
#[test]
fn the_plane_offset_gives_the_spatula_its_bite() {
    let size = 200u32;
    let path = [[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]];

    let run = |offset_track: f32| -> Vec<f32> {
        let (mut t, layer, _) = sculpt_canvas(size);
        arm_sculpt(&mut t, SCRAPE, 0.5, 1.0);
        t.set_sculpt_offset(offset_track);
        drag(&mut t, &path);
        heights_of(&t, layer)
    };
    let skim = run(0.5); // the plane on the surface
    let dig = run(0.3); // 0.4 of a paint-load below it

    let mut deeper = 0u32;
    let mut shallower = 0u32;
    for (s, d) in skim.iter().zip(&dig) {
        if *d < *s - 1e-5 {
            deeper += 1;
        } else if *d > *s + 1e-5 {
            shallower += 1;
        }
    }
    assert!(
        deeper > 500,
        "sinking the plane by 0.4 loads dug deeper in only {deeper} texels — the Offset is not reaching the \
         kernel, so the slider's whole job (Scrape's bite) is inert"
    );
    assert_eq!(
        shallower, 0,
        "sinking the Scrape's plane made it take LESS off in {shallower} texels — the sign is inverted"
    );
}

/// **The kernel knows how much paint it removed.**
///
/// Doc 18 §6 leaves the bifurcation open — *where does the scraped paint go?* Blender deletes it; paint does
/// not do that, and we own a conservative displacement engine already. The scope decision was to ship the
/// non-conservative Scrape **but compute the displaced volume and discard it explicitly**, so that W5's
/// Conserve (the bow wave) is a *flag* rather than a rewrite.
///
/// This is what turns that from a promise into a fact: the number is checked against the relief that
/// actually moved, now, while the code that moved it is under my hand. A volume computed in Wave 5 against a
/// kernel written in Wave 2 would be a number nobody could check.
///
/// **Mutation that must bleed:** return `0.0` from `sculpt_displaced_volume`, or sum `pre − h` instead of
/// `h − pre` (the sign is the whole meaning: negative = material removed).
#[test]
fn the_scrape_reports_the_volume_it_removed() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    arm_sculpt(&mut t, SCRAPE, 0.5, 1.0);

    // Mid-gesture (a shape editor keeps the session open): the volume is a property of the LIVE gesture.
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
    for p in [[60.0, 100.0], [140.0, 100.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }

    let reported = t.sculpt_displaced_volume();
    let after = heights_of(&t, layer);
    let truth: f64 = before
        .iter()
        .zip(&after)
        .map(|(a, b)| f64::from(*b - *a))
        .sum();

    assert!(
        truth < -1.0,
        "fixture: the Scrape removed only {truth:.3} loads·texels — nothing to account for"
    );
    assert!(
        (reported - truth).abs() < 1e-3 * truth.abs().max(1.0),
        "the kernel says it displaced {reported:.4} loads·texels; the relief says {truth:.4}. Wave 5's \
         Conserve is supposed to be a FLAG on top of this number — a wrong number makes it a rewrite."
    );
    assert!(
        reported < 0.0,
        "a Scrape reported a POSITIVE displaced volume ({reported:.4}) — the sign is what says whether the \
         bow wave has paint to pile up or a hole to fill"
    );
}

/// **Switching family mid-shape rebuilds the target from the dabs.**
///
/// The two families' per-texel targets are derived from different things, and only one of them is
/// rebuildable from what the session holds. `blur(pre)` is a function of the frozen relief, so it is a memo —
/// throw it away and it comes back. `Σ w·plane(i)` is a function of the **dab list**, which the render does
/// not have.
///
/// So a Smooth → Scrape switch that merely re-rendered would divide by an all-zero `plane_sum` and pull the
/// whole footprint to height **0** — the paint flattened to the floor of the canvas, wearing a scrape's
/// clothes. `set_sculpt_mode` re-stamps the open shape instead, which is what the house already does when
/// Size or Falloff change.
///
/// **Mutation that must bleed:** drop the `self.refill_open_shape()` from the family-switch branch of
/// `set_sculpt_mode`.
#[test]
fn switching_family_mid_shape_rebuilds_the_target() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    arm_sculpt(&mut t, SMOOTH, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

    // A line, SMOOTHED, still open (no Apply).
    for p in [[60.0, 100.0], [140.0, 100.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    assert!(
        t.paint.sculpt.layer.is_some(),
        "fixture: the un-applied shape must keep the session open"
    );

    // …and now the artist reaches for Scrape, mid-shape.
    t.set_sculpt_mode(SCRAPE);
    let after = heights_of(&t, layer);

    // The floor bug is unmistakable: the touched texels get pulled to ~0 from a relief that is nowhere near
    // it. Count how many landed at the floor while the surface under them was well above it.
    let floored = before
        .iter()
        .zip(&after)
        .filter(|(b, a)| **b > 0.15 && a.abs() < 1e-3)
        .count();
    assert_eq!(
        floored, 0,
        "switching Smooth → Scrape inside an open shape flattened {floored} texels to height ZERO. The \
         plane target is derived from the DAB LIST, and the switch re-rendered without re-stamping — so the \
         kernel divided by an all-zero `plane_sum` and pulled the paint to the floor of the canvas."
    );

    // …and it really did scrape: material came off, none went on.
    let (mut down, mut up) = (0u32, 0u32);
    for (bb, aa) in before.iter().zip(&after) {
        if *aa < *bb - 1e-5 {
            down += 1;
        } else if *aa > *bb + 1e-5 {
            up += 1;
        }
    }
    assert!(
        down > 100,
        "after the switch the Scrape removed nothing ({down} texels) — the target was rebuilt as zero \
         WEIGHT rather than zero plane, which is the same bug with a friendlier face"
    );
    assert_eq!(up, 0, "the Scrape raised {up} texels");
}

/// **The plane family costs the same 12 B/px the Smooth family does.**
///
/// The two per-texel targets are mutually exclusive — a verb belongs to exactly one family — so the session
/// carries ONE of them, never both. Get this wrong and five verbs cost 16 B/px where two cost 12, which at
/// 4096² is another 67 MB held for the length of a stroke.
///
/// It is checked on the real thing rather than reasoned about, for the reason HR-13 had to be amended:
/// *a rule that never observes cannot fire* ([[feedback_a_rule_that_never_observes_cannot_fire]]).
///
/// **Mutation that must bleed:** delete the `mode.family() == SculptFamily::Smooth` guard around the memo
/// rebuild in `render_sculpt` — the plane verbs then allocate a blur memo they never read.
#[test]
fn a_plane_stroke_costs_twelve_bytes_per_pixel_too() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, SCRAPE, 0.5, 1.0);

    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));

    let s = &t.paint.sculpt;
    assert!(s.layer.is_some(), "fixture: the gesture is in flight");
    assert_eq!(
        (s.pre.len(), s.amount.len(), s.plane_sum.len()),
        (n, n, n),
        "the plane session's three planes are not canvas-sized — the arithmetic below measures nothing"
    );
    assert!(
        s.blurred.is_empty() && s.blur_done.is_empty(),
        "a Scrape allocated the SMOOTH family's blur memo ({} B) — it never reads it, and it is 4 B/px on \
         top of a session that promises 12",
        s.blurred.len() * 4
    );
    let live = (s.pre.len() + s.amount.len() + s.plane_sum.len()) * 4;
    assert_eq!(
        live / n,
        12,
        "a live plane stroke costs {} B/px, not the 12 the Smooth family costs",
        live / n
    );

    // …and the commit frees it whole, exactly as the Smooth family's does.
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Up));
    let s = &t.paint.sculpt;
    assert!(
        s.layer.is_none() && s.plane_sum.is_empty() && s.fit_scratch.is_empty(),
        "a finished plane stroke left its session behind"
    );
}

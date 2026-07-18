//! The **perf budget of the warp** — the one surface in this module that never had one.
//!
//! `CLAUDE.md` has carried the line *"⚠️ perf do Deform não é gateada (nunca foi) — 3 amostragens/texel a
//! mais no bbox quando há relevo"* since W4 landed. A warning in prose is not a budget: nothing measures
//! it, so nothing can notice it moving. The sibling `sculpt_perf_kill_criterion` says the rule out loud —
//! *a budget nobody enforces is a number in a comment* — and this file is that sentence applied to the
//! module that had been exempt from it.
//!
//! It matters more now than when the note was written. The warp is no longer one tool's engine: the
//! **Smear** was rebuilt on the same displacement field and re-renders through the same single door
//! (`warp_render_relief`), so the colour and the body cannot disagree about where paint went. One door
//! means one place to regress, and it is now on the hot path of two tools instead of one.
//!
//! ## What is measured, and against what
//!
//! Two numbers per tool, because they answer different questions:
//!
//! * **TOTAL** — what the artist actually pays per pointer move. This is what the bar is on.
//! * **RELIEF** — the same stroke on a layer carrying no relief, subtracted. That isolates exactly the
//!   advection the W4 note flagged (the extra samplings of `h`, `covers` and `mats`), so a regression in
//!   the part nobody was watching is visible as its own column rather than buried in a total.
//!
//! And **two bars**, because there are two ways this path goes wrong and only one of them is about speed:
//!
//! * a **ratio** between the two canvas sizes — every kernel here is bounded by the brush footprint, so
//!   quadrupling the canvas must not move the cost. This is the failure that actually happens (some pass
//!   starts walking the whole plane), it names itself in the assert, and being a ratio it is immune to how
//!   fast the machine is;
//! * a **wall-clock kill**, for everything else — read off a measurement, not chosen, and carrying the
//!   headroom a machine documented as drifting ~3× across a session requires. A gate that flakes gets
//!   deleted, and a deleted gate has no budget at all.
//!
//! The ratio is asserted FIRST, and that ordering is load-bearing — see the comment above the loop.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A canvas with one real impasto stroke laid across the middle, or the same canvas with none.
///
/// The paint is laid by the REAL deposit through the REAL pointer route, so what the warp then carries is
/// what the product would have handed it — a hand-filled `heights` would let the fixture drift from the
/// tool and take the gate's meaning with it.
fn warpable_canvas(size: u32, with_relief: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 100.0, // the widest footprint, as the sculpt gate does — the worst case, not the comfy one
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: with_relief,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    if with_relief {
        t.set_brush_impasto_depth(1.0);
    }
    let mid = (size / 2) as f32;
    t.on_canvas_pointer(cp([150.0, mid], PointerPhase::Down));
    let mut x = 200.0;
    while x <= 1000.0 {
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        x += 50.0;
    }
    t.on_canvas_pointer(cp([x, mid], PointerPhase::Up));
    let _ = t.take_preview_arc();
    t
}

/// **The warp costs what this gate says it costs — with the relief advection counted separately.**
///
/// The two tools that ride the field are both measured, because they are not the same load: the Deform
/// resamples the canvas through a cumulative map, while the Smear also *builds* that map from the dab walk
/// on every move.
///
/// Unlike the sculpt's sibling, the assertion is on the **TOTAL** rather than on a delta. The delta is the
/// right subject there because the sculpt is arithmetic added on top of a frame that already worked; here
/// the warp IS the tool, so a total that quietly doubled while the relief delta stayed flat would be a
/// regression this gate exists to catch, and a delta-only bar would sail straight past it.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn warp_perf_kill_criterion() {
    use std::time::Instant;
    const MOVES: u32 = 12;
    /// Moves excluded from the steady-state mean — the first is the session's set-up (the frozen planes
    /// forked, the buffers first-touched), which is a per-STROKE cost and not what a bar per MOVE means.
    /// Its sibling in the sculpt learned this the hard way: leaving it in produced the outlier failures
    /// that reddened the gate against code that had not changed.
    const WARMUP_MOVES: usize = 2;
    /// Measured 2026-07-18, release, brush radius 100 (the widest footprint):
    ///
    /// ```text
    ///            TOTAL @2048 / @4096      of which RELIEF advection      flat canvas
    ///   DEFORM      4,18 / 4,14 ms              2,52 ms  (60%)             1,66
    ///   SMEAR       3,28 / 3,30 ms              1,71 ms  (52%)             1,57
    /// ```
    ///
    /// **The relief advection is the larger half of the Deform**, which is the number the W4 note never
    /// had: it said *"3 amostragens/texel a mais"*, and three extra samplings turn out to more than double
    /// the tool. That is not a regression — it is the honest price of the body travelling with the pixels —
    /// but it is now a measured price rather than a guessed one.
    ///
    /// Same `8.0` as `sculpt_perf_kill_criterion`, deliberately: one module should have one budget
    /// language, and ~1,8× over the worst measurement is the margin that sibling has lived at without
    /// flaking. Target is ≤4, as there.
    const KILL_MS: f64 = 8.0;
    /// **The structural bar, and the one that cannot flake.**
    ///
    /// Every kernel in this module is bounded by the BRUSH FOOTPRINT, so quadrupling the canvas must not
    /// move the cost — measured, 4,18 → 4,14 and 3,28 → 3,30, i.e. flat inside the noise. The failure this
    /// catches is the one that actually happens here: a change that makes some pass walk the whole plane
    /// instead of the write window. A wall-clock bar would catch that only on a big canvas and only if the
    /// machine happened to be quiet; a RATIO between two sizes measured back to back is immune to how fast
    /// the machine is, which is the lesson this line paid for twice.
    const CANVAS_RATIO_MAX: f64 = 2.0;

    let run = |size: u32, with_relief: bool, smear: bool| -> (f64, f64) {
        let mut t = warpable_canvas(size, with_relief);
        if smear {
            t.set_paint_tool_mode("smear");
        } else {
            t.set_paint_tool_mode("deform");
            t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_RESHAPE);
            t.set_deform_mode(0); // Push — the plainest verb, so the number is the FIELD's, not a verb's
        }
        let mut b = t.paint.brush;
        b.radius_px = 100.0;
        b.impasto = false; // the warp reshapes what is there; it deposits nothing
        t.paint.brush = b;
        t.paint.brush_by_mode[super::super::PaintMode::Deform.slot()] = b;
        t.paint.brush_by_mode[super::super::PaintMode::Smear.slot()] = b;

        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut ms: Vec<f64> = Vec::with_capacity(MOVES as usize);
        for i in 0..MOVES {
            let x = 220.0 + f64::from(i) * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x as f32, mid], PointerPhase::Move));
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([1000.0, mid], PointerPhase::Up));
        let steady = &ms[WARMUP_MOVES..];
        let mean = steady.iter().sum::<f64>() / steady.len() as f64;
        let worst = steady.iter().copied().fold(0.0f64, f64::max);
        (mean, worst)
    };

    // ⚠️ Measure BOTH sizes, then assert the RATIO before the wall-clock kill. The order is not
    // cosmetic: it is what makes the ratio bar reachable at all.
    //
    // With the kill asserted inside the size loop, it always fired first — and not by luck. Any
    // canvas-proportional regression big enough to double the ratio is, by arithmetic, big enough to
    // blow an 8 ms bar on the LARGER canvas, so the ratio would have been an assertion that could
    // never go red: green by construction, exactly the shape this line has been burned by before
    // ([[feedback_layered_defenses_need_per_layer_gates]] — a second defence needs its own way to
    // bleed, or it is decoration).
    //
    // Ordered this way each bar owns a question. The ratio answers *did something start walking the
    // plane?* — the failure that actually happens here, with the diagnosis in its name and immune to
    // how fast the machine is. The kill answers *is it simply too slow?*, which is everything else.
    for (label, smear) in [("DEFORM", false), ("SMEAR", true)] {
        let mut by_size: Vec<f64> = Vec::new();
        for size in [2048u32, 4096] {
            let (flat_mean, _) = run(size, false, smear);
            let (mean, worst) = run(size, true, smear);
            by_size.push(mean);
            println!(
                ">>> {label} COST @{size}px: TOTAL mean {mean:.2} ms/move (steady state, \
                 {WARMUP_MOVES} warm-up moves excluded), worst {worst:.2} (kill {KILL_MS}) | \
                 RELIEF advection {:.2} ms/move [flat canvas {flat_mean:.2}]",
                mean - flat_mean
            );
        }
        let ratio = by_size[1] / by_size[0];
        println!(">>> {label} canvas ratio 4096²/2048² = {ratio:.2}× (bar {CANVAS_RATIO_MAX})");
        assert!(
            ratio < CANVAS_RATIO_MAX,
            "{label} costs {ratio:.2}× as much on a canvas with 4× the texels ({:.2} → {:.2} ms/move). \
             This path is bounded by the BRUSH FOOTPRINT, so its cost must not know how big the canvas \
             is: something started walking the whole plane. Look for a pass whose window is the canvas \
             rather than the dab rect — a full-plane fork, a whole-buffer clear, a `for i in 0..n`.",
            by_size[0],
            by_size[1]
        );
        for (size, mean) in [2048u32, 4096].iter().zip(&by_size) {
            assert!(
                *mean < KILL_MS,
                "{label} costs {mean:.2} ms/move at {size}px — past the kill criterion of \
                 {KILL_MS} ms, and the cost did NOT grow with the canvas, so the window is still the \
                 brush's. The suspect is the relief advection: the warp samples `h`, `covers` and \
                 `mats` on top of the colour, so work added per texel is paid three times over."
            );
        }
    }
}

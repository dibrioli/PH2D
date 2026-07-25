//! **Measurement probe** for the Mask coverage rewrite (`HANDOFF_line_Painter_mask_rewrite_2026-07-25`).
//!
//! Not a gate: these are `#[ignore]`d probes that MEASURE the reported defect (the mask edge goes
//! jagged/torn under many passes) and DUMP the coverage so it can be looked at. The gates that pin the
//! cure live in `mask_tests.rs`; this file is the "render and look" half the handoff §8 prescribes, kept
//! so the next agent can re-measure instead of re-arguing.
//!
//! It also OWNS the measurement helpers (`band_px`, `cross_x`, `vstroke`, …) that the gates in
//! `mask_tests.rs` stand on — one copy of the oracle, so a gate and the probe that motivated it can
//! never disagree about what "the transition band" means.
//!
//! Run:  `cargo test -p ph2d-tool-painter mask_probe -- --ignored --nocapture`
//! Dump: `PH2D_MASK_PROBE_DIR=/tmp/mask cargo test … ` (raw `.gray` files + a `.txt` index; convert
//!       with `magick -size WxH -depth 8 gray:FILE out.png`).

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};

/// A Painter with a flat opaque canvas, in **Mask** paint mode, with the PRODUCT-default brush
/// (radius 25, hardness 0, Smooth falloff, spacing 10 %, strength/flow 1) — the fixture must contain
/// the phenomenon, so nothing here hardens the tip (the reverted gate's `hardness = 1` disk could not
/// grow a feather at all, so it could not see a feather collapse).
pub(super) fn mask_tool(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![0u8; (size * size * 4) as usize];
    for c in px.chunks_exact_mut(4) {
        c[0] = 200;
        c[1] = 30;
        c[2] = 30;
        c[3] = 255;
    }
    t.set_source(px, size, size);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t
}

pub(super) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// One straight vertical stroke at `x`, from `y0` to `y1`, delivered as `steps` Moves (the shell's
/// per-frame delivery), then drained like a frame would.
pub(super) fn vstroke(t: &mut PainterTool, x: f32, y0: f32, y1: f32, steps: u32) {
    t.on_canvas_pointer(cp([x, y0], PointerPhase::Down));
    for i in 1..=steps {
        let y = y0 + (y1 - y0) * (i as f32) / (steps as f32);
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x, y1], PointerPhase::Up));
    let _ = t.take_preview_arc();
}

/// The **protection** coverage the overlay reads: `1 - R/255` (white scratch = unprotected = 0).
pub(super) fn coverage(t: &PainterTool, size: u32) -> Vec<f32> {
    let s = &t.paint.mask_scratch_rgba;
    let n = size as usize * size as usize;
    // No scratch = nothing protected, which is exactly what the overlay draws in that state — so the
    // oracle can be asked BEFORE the first stroke (and after an Apply) without a special case. The
    // gates still prove a scratch exists by asserting the stroke PROTECTS something.
    if s.len() < n * 4 {
        return vec![0.0; n];
    }
    (0..n).map(|i| 1.0 - f32::from(s[i * 4]) / 255.0).collect()
}

/// Sub-pixel x where the row `y` crosses `level` on its RIGHT flank (linear interpolation between
/// the bracketing texels). `None` if the row never crosses.
pub(super) fn cross_x(cov: &[f32], size: u32, y: u32, level: f32) -> Option<f32> {
    let row = y as usize * size as usize;
    let mut prev = cov[row];
    for x in 1..size as usize {
        let v = cov[row + x];
        if prev >= level && v < level {
            let t = (prev - level) / (prev - v).max(1e-9);
            return Some(x as f32 - 1.0 + t);
        }
        prev = v;
    }
    None
}

/// Width in texels of the transition band on the right flank of row `y` (the distance between the
/// `hi` and `lo` crossings) — the number that says "feather" or "hard edge".
pub(super) fn band_px(cov: &[f32], size: u32, y: u32) -> f32 {
    let (a, b) = (cross_x(cov, size, y, 0.9), cross_x(cov, size, y, 0.1));
    match (a, b) {
        (Some(a), Some(b)) => b - a,
        _ => f32::NAN,
    }
}

/// Peak-to-peak wobble of the 50 % iso-contour along the stroke — the SCALLOP the dab spacing leaves.
/// A contour that follows the path is flat; one that follows the individual dab discs saws.
fn contour_ripple(cov: &[f32], size: u32, y0: u32, y1: u32) -> (f32, f32) {
    let xs: Vec<f32> = (y0..y1)
        .filter_map(|y| cross_x(cov, size, y, 0.5))
        .collect();
    if xs.len() < 3 {
        return (f32::NAN, f32::NAN);
    }
    let mn = xs.iter().copied().fold(f32::MAX, f32::min);
    let mx = xs.iter().copied().fold(f32::MIN, f32::max);
    // Local sawtooth: the mean |second difference| ignores a slow drift and reports the per-dab kink.
    let saw = xs
        .windows(3)
        .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
        .sum::<f32>()
        / (xs.len() - 2) as f32;
    (mx - mn, saw)
}

/// Dump `cov` as an 8-bit gray raster next to a line in the index file, for `magick`+eyes.
fn dump(name: &str, cov: &[f32], size: u32) {
    let Ok(dir) = std::env::var("PH2D_MASK_PROBE_DIR") else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let bytes: Vec<u8> = cov
        .iter()
        .map(|c| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
        .collect();
    let path = format!("{dir}/{name}.gray");
    let _ = std::fs::write(&path, &bytes);
    println!("DUMP {path} ({size}x{size}, 8-bit gray, coverage: white = protected)");
}

/// **PROBE 1 — the reported defect.** How the feather band and the contour scallop evolve with the
/// number of identical passes, on the product build-up that ships today.
#[test]
#[ignore]
fn probe_mask_edge_under_many_passes() {
    const S: u32 = 256;
    let mut t = mask_tool(S);
    println!(
        "brush: r={} hardness={} spacing={} strength={} flow={} accumulate={} falloff={:?}",
        t.paint.brush.radius_px,
        t.paint.brush.hardness,
        t.paint.brush.spacing,
        t.paint.brush.strength,
        t.paint.brush.flow,
        t.paint.brush.accumulate,
        t.paint.brush.falloff
    );
    println!(
        "pass  band(0.9->0.1 px)  x@0.5   ripple p2p   sawtooth   cov@x=128  cov@140  cov@150"
    );
    for pass in 1..=20u32 {
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
        let cov = coverage(&t, S);
        if matches!(pass, 1 | 2 | 3 | 5 | 10 | 15 | 20) {
            let (p2p, saw) = contour_ripple(&cov, S, 80, 180);
            let row = 130usize * S as usize;
            println!(
                "{pass:4}  {:16.2}  {:6.2}  {:11.4}  {:9.4}  {:9.3}  {:7.3}  {:7.3}",
                band_px(&cov, S, 130),
                cross_x(&cov, S, 130, 0.5).unwrap_or(f32::NAN),
                p2p,
                saw,
                cov[row + 128],
                cov[row + 140],
                cov[row + 150],
            );
            dump(&format!("pass{pass:02}"), &cov, S);
        }
    }
}

/// **PROBE 2 — the seam.** Two adjacent parallel strokes: the valley between them must fill like
/// paint (the §13.6 envelope left it LIGHTER than the additive deposit, which read as a white line).
#[test]
#[ignore]
fn probe_mask_valley_between_neighbour_strokes() {
    const S: u32 = 256;
    let mut t = mask_tool(S);
    let r = t.paint.brush.radius_px;
    for x in [128.0 - r * 0.8, 128.0 + r * 0.8] {
        vstroke(&mut t, x, 60.0, 200.0, 25);
    }
    let cov = coverage(&t, S);
    let row = 130usize * S as usize;
    let (lo, hi) = (
        (128.0 - r * 0.8) as usize - 4,
        (128.0 + r * 0.8) as usize + 4,
    );
    let valley = (lo..hi).map(|x| cov[row + x]).fold(f32::MAX, f32::min);
    let core = (lo..hi).map(|x| cov[row + x]).fold(f32::MIN, f32::max);
    println!(
        "two neighbour strokes @ ±{:.1}px: valley min = {valley:.4}, core max = {core:.4}, dip = {:.4}",
        r * 0.8,
        core - valley
    );
    dump("valley", &cov, S);
}

/// **PROBE 3 — is the law a fact of the PATH or of the batching?** The same geometric stroke
/// delivered in few vs many pointer events must leave the same coverage; a product over dabs does
/// not (the disease this line cured 4× in the relief).
#[test]
#[ignore]
fn probe_mask_is_a_fact_of_the_path_not_the_polling() {
    const S: u32 = 256;
    let mut coarse = mask_tool(S);
    vstroke(&mut coarse, 128.0, 60.0, 200.0, 5);
    let mut fine = mask_tool(S);
    vstroke(&mut fine, 128.0, 60.0, 200.0, 100);
    let (a, b) = (coverage(&coarse, S), coverage(&fine, S));
    let maxd = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f32, f32::max);
    let ndiff = a
        .iter()
        .zip(b.iter())
        .filter(|(x, y)| (*x - *y).abs() > 1.5 / 255.0)
        .count();
    println!(
        "same path, 5 vs 100 pointer events: max |Δcov| = {maxd:.4}, texels differing > 1 level = {ndiff}"
    );
    println!(
        "  band coarse = {:.2}px, fine = {:.2}px",
        band_px(&a, S, 130),
        band_px(&b, S, 130)
    );
}

/// One CURVED stroke (a quarter arc) — the geometry the defect was reported on ("serrilha numa
/// curva"): a straight stroke's iso-contour is a straight line, which no amount of hardening can make
/// look jagged, so a straight fixture cannot contain the phenomenon.
pub(super) fn arc_stroke(t: &mut PainterTool, cx: f32, cy: f32, r: f32, steps: u32) {
    let at = |i: u32| {
        let a = std::f32::consts::FRAC_PI_2 * (i as f32) / (steps as f32);
        [cx + r * a.cos(), cy + r * a.sin()]
    };
    t.on_canvas_pointer(cp(at(0), PointerPhase::Down));
    for i in 1..=steps {
        t.on_canvas_pointer(cp(at(i), PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(at(steps), PointerPhase::Up));
    let _ = t.take_preview_arc();
}

/// **PROBE 5 — the reported picture.** The same arc painted 1 vs N times: measure how ragged the
/// 50 % contour gets (the sawtooth) and dump both so the edge can be LOOKED at.
#[test]
#[ignore]
fn probe_mask_curve_under_many_passes() {
    const S: u32 = 256;
    let mut t = mask_tool(S);
    println!(
        "arc r=70 with brush r={}: pass  sawtooth(px)  p2p(px)  band(px)",
        t.paint.brush.radius_px
    );
    for pass in 1..=15u32 {
        arc_stroke(&mut t, 40.0, 40.0, 70.0, 60);
        if matches!(pass, 1 | 2 | 5 | 15) {
            let cov = coverage(&t, S);
            // Rows well inside the arc's span, where the contour is genuinely curved.
            let (p2p, saw) = contour_ripple(&cov, S, 60, 100);
            let rough = contour_roughness(&cov, S, 60, 100);
            println!(
                "{pass:4}  {saw:12.4}  {p2p:7.3}  {:7.2}   roughness={rough:.4} (rel {:.4})",
                band_px(&cov, S, 80),
                rough / band_px(&cov, S, 80)
            );
            dump(&format!("arc{pass:02}"), &cov, S);
        }
    }
}

/// Raggedness of the 50 % contour with the geometry taken OUT: the residual after a least-squares
/// QUADRATIC fit. A straight or gently curved stroke is fitted almost exactly, so what remains is the
/// per-dab scallop + quantisation — the actual "serrilhado". (The raw second difference cannot tell
/// them apart: on an arc it reports the arc's own curvature, which is not a defect.)
fn contour_roughness(cov: &[f32], size: u32, y0: u32, y1: u32) -> f32 {
    let pts: Vec<(f32, f32)> = (y0..y1)
        .filter_map(|y| cross_x(cov, size, y, 0.5).map(|x| (y as f32, x)))
        .collect();
    if pts.len() < 8 {
        return f32::NAN;
    }
    // Normal equations for x = a + b·y + c·y² (centred so the 3×3 stays well conditioned).
    let ybar = pts.iter().map(|p| p.0).sum::<f32>() / pts.len() as f32;
    let (mut s0, mut s1, mut s2, mut s3, mut s4) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut t0, mut t1, mut t2) = (0.0f64, 0.0f64, 0.0f64);
    for &(y, x) in &pts {
        let (u, x) = (f64::from(y - ybar), f64::from(x));
        s0 += 1.0;
        s1 += u;
        s2 += u * u;
        s3 += u * u * u;
        s4 += u * u * u * u;
        t0 += x;
        t1 += x * u;
        t2 += x * u * u;
    }
    // Solve the symmetric 3×3 by Cramer.
    let m = [[s0, s1, s2], [s1, s2, s3], [s2, s3, s4]];
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-9 {
        return f32::NAN;
    }
    let rhs = [t0, t1, t2];
    let mut coef = [0.0f64; 3];
    for (k, c) in coef.iter_mut().enumerate() {
        let mut mk = m;
        for (row, r) in mk.iter_mut().enumerate() {
            r[k] = rhs[row];
        }
        let d = mk[0][0] * (mk[1][1] * mk[2][2] - mk[1][2] * mk[2][1])
            - mk[0][1] * (mk[1][0] * mk[2][2] - mk[1][2] * mk[2][0])
            + mk[0][2] * (mk[1][0] * mk[2][1] - mk[1][1] * mk[2][0]);
        *c = d / det;
    }
    let rms = pts
        .iter()
        .map(|&(y, x)| {
            let u = f64::from(y - ybar);
            let fit = coef[0] + coef[1] * u + coef[2] * u * u;
            (f64::from(x) - fit).powi(2)
        })
        .sum::<f64>()
        / pts.len() as f64;
    rms.sqrt() as f32
}

/// **PROBE 6 — the gesture the artist actually makes.** A back-and-forth SCRIBBLE of overlapping
/// strokes: the valleys between neighbours must fill like paint (the §13.6 envelope-across-strokes
/// left them lighter than their surroundings — the white lines Enio rejected). Reports the darkest
/// valley inside the scribbled block and dumps it to be looked at.
#[test]
#[ignore]
fn probe_mask_scribble_seams() {
    const S: u32 = 256;
    for (label, step) in [("tight", 0.6_f32), ("normal", 1.0), ("loose", 1.4)] {
        let mut t = mask_tool(S);
        let r = t.paint.brush.radius_px;
        let lanes = 7;
        let x0 = 128.0 - (lanes as f32 - 1.0) * 0.5 * r * step;
        // Nobody scribbles a region ONCE: the sweep is repeated, and each repetition composites over
        // the last. Report every repetition — the first one is the worst case, not the gesture.
        for sweep in 1..=3u32 {
            for i in 0..lanes {
                vstroke(&mut t, x0 + i as f32 * r * step, 70.0, 190.0, 30);
            }
            let cov = coverage(&t, S);
            let (a, b) = (
                (x0 + r * step * 0.5) as usize,
                (x0 + (lanes as f32 - 1.5) * r * step) as usize,
            );
            let row = 130usize * S as usize;
            let lo = (a..b).map(|x| cov[row + x]).fold(f32::MAX, f32::min);
            let hi = (a..b).map(|x| cov[row + x]).fold(f32::MIN, f32::max);
            println!(
                "scribble {label:6} step {:.1}px  sweep {sweep}: interior min = {lo:.4}, max = {hi:.4}, seam = {:.4}",
                r * step,
                hi - lo
            );
            if sweep == 3 {
                dump(&format!("scribble_{label}"), &cov, S);
            }
        }
        let cov = coverage(&t, S);
        // Inside the block (skip the outer half-lane on each side), the darkest texel of the mid row
        // is the seam if there is one.
        let (a, b) = (
            (x0 + r * step * 0.5) as usize,
            (x0 + (lanes as f32 - 1.5) * r * step) as usize,
        );
        let row = 130usize * S as usize;
        let lo = (a..b).map(|x| cov[row + x]).fold(f32::MAX, f32::min);
        let hi = (a..b).map(|x| cov[row + x]).fold(f32::MIN, f32::max);
        println!(
            "scribble {label:6} (lane step {:.1} px = {step}·r): interior min = {lo:.4}, max = {hi:.4}, seam depth = {:.4}",
            r * step,
            hi - lo
        );
        dump(&format!("scribble_{label}"), &cov, S);
    }
}

/// **PROBE 4 — the fill methods.** A Line/Curve mask stroke re-stamps the WHOLE shape every frame.
/// If the deposit re-multiplies the already-committed scratch, holding still DARKENS it per frame
/// (an artefact of the frame rate, not of the gesture).
#[test]
#[ignore]
fn probe_mask_fill_method_holding_still() {
    const S: u32 = 256;
    let mut t = mask_tool(S);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Down));
    println!(
        "after Down with StrokeMethod::Line: mask scratch is {} bytes (0 = the shape router never \
         reached `paint_begin`, so `ensure_mask_scratch` never ran and the mask route early-returns)",
        t.paint.mask_scratch_rgba.len()
    );
    if t.paint.mask_scratch_rgba.is_empty() {
        t.on_canvas_pointer(cp([128.0, 200.0], PointerPhase::Move));
        let _ = t.take_preview_arc();
        println!(
            "  after one Move: {} bytes — a fill-method mask stroke paints NOTHING (pre-existing; \
             not this wave's law)",
            t.paint.mask_scratch_rgba.len()
        );
        return;
    }
    let mut seen = Vec::new();
    for frame in 0..6u32 {
        // The pointer does NOT move: every frame re-stamps the same line.
        t.on_canvas_pointer(cp([128.0, 200.0], PointerPhase::Move));
        let _ = t.take_preview_arc();
        let cov = coverage(&t, S);
        let row = 130usize * S as usize;
        seen.push((frame, cov[row + 128], cov[row + 140], band_px(&cov, S, 130)));
    }
    t.on_canvas_pointer(cp([128.0, 200.0], PointerPhase::Up));
    let _ = t.take_preview_arc();
    for (f, c0, c1, b) in seen {
        println!("frame {f}: cov@128 = {c0:.4}  cov@140 = {c1:.4}  band = {b:.2}px");
    }
}

/// **PROBE 7 — the scrub.** Back-and-forth WITHOUT lifting the pen (the gesture "muitas passadas"
/// most often means): under Krita's Wash law the stroke's coverage is its dabs' envelope, so scrubbing
/// re-lays what is already there and the shoulder cannot move. Reports the deltas a gate can pin.
#[test]
#[ignore]
fn probe_mask_scrub_within_one_stroke() {
    const S: u32 = 256;
    let sweep_once = {
        let mut t = mask_tool(S);
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
        coverage(&t, S)
    };
    let scrubbed = {
        let mut t = mask_tool(S);
        // ONE pen-down, four sweeps over the same segment.
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Down));
        for leg in 0..4u32 {
            let (a, b) = if leg % 2 == 0 {
                (60.0, 200.0)
            } else {
                (200.0, 60.0)
            };
            for i in 1..=25u32 {
                let y = a + (b - a) * (i as f32) / 25.0;
                t.on_canvas_pointer(cp([128.0, y], PointerPhase::Move));
            }
        }
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        coverage(&t, S)
    };
    let (mut maxd, mut at) = (0.0_f32, 0usize);
    for (i, (a, b)) in sweep_once.iter().zip(scrubbed.iter()).enumerate() {
        if (b - a).abs() > maxd {
            maxd = (b - a).abs();
            at = i;
        }
    }
    println!(
        "  worst texel at ({}, {}): single = {:.3}, scrubbed = {:.3}",
        at % S as usize,
        at / S as usize,
        sweep_once[at],
        scrubbed[at]
    );
    // Where the two strokes genuinely overlap (away from the end caps), the envelope must agree.
    let mid: f32 = (90..170)
        .flat_map(|y| (100..160).map(move |x| (y, x)))
        .map(|(y, x)| {
            let i = y * S as usize + x;
            (scrubbed[i] - sweep_once[i]).abs()
        })
        .fold(0.0_f32, f32::max);
    println!(
        "  max |Δcov| in the stroke's MIDDLE (rows 90..170) = {mid:.4} ({} levels)",
        (mid * 255.0).round()
    );
    let levels = (maxd * 255.0).round();
    println!(
        "scrub (4 legs, one pen-down) vs a single sweep: max |Δcov| = {maxd:.4} ({levels} levels of 255)"
    );
    println!(
        "  band single = {:.2}px, scrubbed = {:.2}px",
        band_px(&sweep_once, S, 130),
        band_px(&scrubbed, S, 130)
    );
}

/// **PROBE 8 — the floor and the clock.** Where the transition band settles under a great many
/// passes (the number the anti-aliasing gate can stand on), and what a mask move COSTS at 2048² and
/// 4096² (the handoff's FPS constraint: the rewrite must stay under 16.7 ms/frame).
#[test]
#[ignore]
fn probe_mask_band_floor_and_cost() {
    const S: u32 = 256;
    let mut t = mask_tool(S);
    for pass in 1..=60u32 {
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
        if matches!(pass, 1 | 5 | 15 | 30 | 45 | 60) {
            let cov = coverage(&t, S);
            println!("pass {pass:3}: band = {:.3} px", band_px(&cov, S, 130));
        }
    }
    for size in [2048u32, 4096] {
        let mut t = mask_tool(size);
        let c = size as f32 * 0.5;
        t.set_brush_size_px(120.0); // a big soft mask brush: the worst per-move footprint
        t.on_canvas_pointer(cp([c - 200.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut worst = std::time::Duration::ZERO;
        let mut total = std::time::Duration::ZERO;
        let n = 40;
        for i in 1..=n {
            let x = c - 200.0 + (i as f32) * 10.0;
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
            let dt = t0.elapsed();
            worst = worst.max(dt);
            total += dt;
        }
        t.on_canvas_pointer(cp([c + 200.0, c], PointerPhase::Up));
        println!(
            "mask move @{size}²: mean {:.2} ms, worst {:.2} ms (brush r=60, {n} moves)",
            total.as_secs_f64() * 1000.0 / f64::from(n),
            worst.as_secs_f64() * 1000.0
        );
    }
}

/// **PROBE 9 — does the per-layer-COLOUR route escape the coverage law?** Arming layers also installs
/// them as the Shape silhouette (hard border by design), so the FEATHER cannot answer. The scrub can:
/// under the envelope a re-crossing dab adds nothing, under a route with no coverage buffer it darkens.
#[test]
#[ignore]
fn probe_mask_per_layer_colour_route() {
    const SZ: u32 = 256;
    let arm = |t: &mut PainterTool| {
        let red: Vec<u8> = (0..64).flat_map(|_| [220u8, 20, 20, 255]).collect();
        let blue: Vec<u8> = (0..64).flat_map(|_| [20u8, 20, 220, 255]).collect();
        t.set_brush_shape_layers(vec![(red, 8, 8), (blue, 8, 8)]);
        t.toggle_brush_shape_per_layer_color();
    };
    let mut single = mask_tool(SZ);
    arm(&mut single);
    vstroke(&mut single, 128.0, 60.0, 200.0, 25);
    let a = coverage(&single, SZ);
    let mut scrub = mask_tool(SZ);
    arm(&mut scrub);
    scrub.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Down));
    for leg in 0..4u32 {
        let (p, q) = if leg % 2 == 0 { (60.0, 200.0) } else { (200.0, 60.0) };
        for i in 1..=25u32 {
            scrub.on_canvas_pointer(cp([128.0, p + (q - p) * (i as f32) / 25.0], PointerPhase::Move));
        }
    }
    scrub.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Up));
    let _ = scrub.take_preview_arc();
    let b = coverage(&scrub, SZ);
    let body = (90..170)
        .flat_map(|y| (100..160).map(move |x| y * SZ as usize + x))
        .map(|i| (b[i] - a[i]).abs())
        .fold(0.0_f32, f32::max);
    println!(
        "per-layer-colour mask stroke: scrub vs single, body max |Δcov| = {body:.4} ({} levels); \
         coloured texels = {}",
        (body * 255.0).round(),
        {
            let s = scrub.paint.mask_scratch_rgba.clone();
            (70..190usize)
                .flat_map(|y| (110..150usize).map(move |x| (y * SZ as usize + x) * 4))
                .filter(|&i| s[i] != s[i + 1] || s[i + 1] != s[i + 2])
                .count()
        }
    );
}

//! Gates for **Inflate's Smoothness (Radius)** knob — split from [`super::inflate`] for the file-LOC cap.
//!
//! Enio, 2026-07-14: *"o Inflate sofre com uma borda rígida como o smooth com radius muito alto … não
//! conseguiríamos resolver essa borda rígida com um parâmetro radius como no smooth?"* We could, and did —
//! a ball dilation makes flat-topped plateaus, and a small blur of the offset rounds their cliff into a
//! shoulder, the way Smooth's own Radius sets its kernel.

use super::*;
use ph2d_painter_brush::Falloff;
use std::sync::Arc;

const INFLATE: u8 = 7;
/// Depth `+0.5` loads on the `0..1` track.
const DEPTH_UP: f32 = 0.75;

// ── Smoothness: the ball's hard edge, rounded (Enio 2026-07-14) ──────────────────────────────────────

/// The steepest single-texel step in a height field over the `y = mid` row within `[x0, x1)` — a proxy for
/// "how hard is the edge". A dilation leaves a cliff (a big step); a blur spreads it (small steps).
fn max_step(h: &[f32], size: u32, mid: u32, x0: u32, x1: u32) -> f32 {
    let mut worst = 0.0f32;
    for x in x0..x1.min(size - 1) {
        let i = (mid * size + x) as usize;
        worst = worst.max((h[i + 1] - h[i]).abs());
    }
    worst
}

/// **The Smoothness knob rounds Inflate's hard edge — and `0` is byte-identical to the raw ball.**
///
/// A morphological dilation makes flat-topped plateaus with a **cliff** at the ball's reach — it is Smooth's
/// own complaint at large radius, which is exactly the parallel Enio drew. So Inflate gained a Radius: a blur
/// of the offset that spreads that cliff into a shoulder. The gate measures the steepest step across the
/// inflated edge and asks that turning Smoothness up makes it gentler — while `0` changes nothing at all.
///
/// **Mutations that must bleed** (both checked): drop the `smooth` branch in `fill_one_tile` (turning the
/// knob does nothing — the edge stays a cliff); and make `MemoKey::reach` ignore the `+ smooth` (the tile
/// seam shows, and the byte-identity gate below bleeds).
#[test]
fn the_smoothness_knob_rounds_the_inflate_edge() {
    let size = 200u32;
    let run = |smooth_norm: f32| -> Vec<f32> {
        // A flat mesa of paint with a sharp edge: dilating it moves the cliff out, and the cliff is what
        // the Smoothness is meant to soften.
        let (mut t, layer, _) = sculpt_canvas(size);
        let n = (size * size) as usize;
        let c = size as f32 * 0.5;
        let mesa: Vec<f32> = (0..n)
            .map(|i| {
                let x = (i % size as usize) as f32;
                let y = (i / size as usize) as f32;
                let r = ((x - c) * (x - c) + (y - c) * (y - c)).sqrt();
                if r < 40.0 { 1.0 } else { 0.0 }
            })
            .collect();
        t.heights.insert(layer, Arc::new(mesa));
        t.covers.insert(layer, Arc::new(vec![255u8; n]));
        t.sync_relief_flags();

        arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
        let mut b = t.paint.brush;
        b.radius_px = 90.0; // cover the whole mesa + its dilated rim
        b.falloff = Falloff::Constant;
        t.paint.brush = b;
        t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
        t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ an 8-px ball
        t.set_sculpt_smooth(smooth_norm);
        drag(&mut t, &[[c, c], [c + 1.0, c]]);
        heights_of(&t, layer)
    };

    let mid = size / 2;
    let sharp = run(0.0); // the raw ball
    let soft = run(1.0); // 16-px smoothing

    // The inflated edge sits out around x ≈ centre + 40 (mesa) + 8 (ball). Measure the cliff there.
    let (x0, x1) = (mid + 40, mid + 70);
    let sharp_step = max_step(&sharp, size, mid, x0, x1);
    let soft_step = max_step(&soft, size, mid, x0, x1);
    assert!(
        sharp_step > 0.05,
        "fixture: the raw ball left no edge to soften ({sharp_step:.3} loads/texel) — the mesa or the ball \
         is wrong, and the comparison below is vacuous"
    );
    assert!(
        soft_step < sharp_step * 0.6,
        "Smoothness did not round the edge: the steepest step across the inflated rim is {soft_step:.3} \
         loads/texel with the knob at max, {sharp_step:.3} with it at zero. A ball dilation leaves a cliff \
         (Smooth's own complaint at large radius); the Radius knob is supposed to spread it into a shoulder."
    );

    // …and Smoothness 0 is EXACTLY the raw ball (opt-in: Inflate is byte-unchanged until you reach for it).
    let zero = run(0.0);
    let differing = sharp
        .iter()
        .zip(&zero)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "Smoothness 0 changed the offset in {differing} texels — the bottom of the track must be the pure \
         ball, or every canvas inflated before this knob existed just moved"
    );
}

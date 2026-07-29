//! **The Expression modal's live preview** (plano 10 W2) — the third column.
//!
//! Two views, and both are needed:
//!
//! * the **puppet frame**, which answers *what will it look like*;
//! * the **curve strip**, which answers *what is it doing* — the value across the
//!   whole preview window, with the un-driven baseline dashed under it.
//!
//! ⚠️ The strip is the half nobody copies. Blender's Drivers Editor plots the
//! MAPPING (driver in, property out) rather than the result, and seeing the
//! relation is what makes a driver comprehensible; every other product shows only
//! the outcome. A dot that moves tells you it moves. The strip tells you *how*.
//!
//! ⚠️ **One door, two views.** [`sample_window`] evaluates the stack across the
//! window ONCE per frame; the strip plots that vector and the puppet indexes it at
//! the current phase. A puppet that evaluated on its own would drift from the
//! curve drawn beside it, and the artist would have no way to know which one lies.
//!
//! ⚠️ **The clock is a FRAME COUNT**, and that is the house convention for chrome
//! (`ToastQueue::tick` ages by frames). The "never count ticks, count wall-clock"
//! law this repo learned in the impasto is about the ARTWORK — where the frame
//! rate would leak into what the artist produces. A thumbnail is not artwork, and
//! a frame count is what makes the preview reproducible in a gate.

use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_polyline, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_expr_recipes::RecipeStack;
use ph2d_timeline::PropKind;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Samples across the preview window. ⚠️ Derived from the strip's width: the
/// column is 190 px, so ~120 samples is a shade under two per pixel — past that
/// the polyline gains vertices the screen cannot show.
pub const PREVIEW_SAMPLES: usize = 120;

/// The window the preview loops over, in seconds.
pub const PREVIEW_SECONDS: f64 = 2.0;

/// Frames the loop takes at 60 fps.
const PREVIEW_FRAMES: u32 = 120;

/// The `value` the preview evaluates against — the property with no formula on it.
///
/// ⚠️ Declared, not discovered (plano 10 P3). The composed value of the property is
/// not in the panel's snapshot; when it becomes available this constant is the ONE
/// place that changes, and the dashed baseline moves with it.
pub const PREVIEW_VALUE: f32 = 0.0;

// ── Proportions of the drawing itself ────────────────────────────────────────
//
// ⚠️ Every number below is a FRACTION of the box it is drawn in, not a measure:
// the puppet has to read the same at any card size, so it is built from ratios
// and never from pixels. Naming them keeps the figures adjustable in one place.
/// Headroom added above and below the plotted curve.
const CURVE_PAD_FRAC: f32 = 0.1; // LITERAL-PX-OK: fracao do desenho, nao medida
/// Dashes across the baseline.
const BASELINE_DASHES: f32 = 24.0; // LITERAL-PX-OK: CONTAGEM de tracos, nao medida
/// Margin inside the puppet frame.
const PUPPET_PAD_FRAC: f32 = 0.18; // LITERAL-PX-OK: fracao do desenho, nao medida
/// The moving figure's size, as a fraction of the frame.
const PUPPET_UNIT_FRAC: f32 = 0.22; // LITERAL-PX-OK: fracao do desenho, nao medida
/// Needle length, as a fraction of the frame.
const NEEDLE_FRAC: f32 = 0.4; // LITERAL-PX-OK: fracao do desenho, nao medida
/// A scale puppet spans this fraction of its range at the bottom of the curve…
const SCALE_MIN_FRAC: f32 = 0.3; // LITERAL-PX-OK: fracao do desenho, nao medida
/// …and grows by this much across it.
const SCALE_SPAN_FRAC: f32 = 0.7; // LITERAL-PX-OK: fracao do desenho, nao medida
/// The long side of a scale puppet's box.
const SCALE_LONG_FRAC: f32 = 0.6; // LITERAL-PX-OK: fracao do desenho, nao medida
/// The short side of a scale puppet's box.
const SCALE_SHORT_FRAC: f32 = 0.35; // LITERAL-PX-OK: fracao do desenho, nao medida
/// The fading square's side.
const FADE_SIDE_FRAC: f32 = 0.55; // LITERAL-PX-OK: fracao do desenho, nao medida
/// A fading square never disappears entirely — an empty frame reads as broken.
const FADE_FLOOR: f32 = 0.05; // LITERAL-PX-OK: piso de alfa, nao medida
/// The filling bar's height.
const BAR_H_FRAC: f32 = 0.3; // LITERAL-PX-OK: fracao do desenho, nao medida
/// Half, for centring.
const HALF: f32 = 0.5; // LITERAL-PX-OK: aritmetica de centralizacao, nao medida

/// The preview's phase (`0..1`) for a frame counter.
#[must_use]
pub fn phase(frame: u32) -> f32 {
    (frame % PREVIEW_FRAMES) as f32 / PREVIEW_FRAMES as f32
}

/// What the stack produces across the whole window.
///
/// ⚠️ Through `ph2d_expr::eval` — the PRODUCT's evaluator, the same one the
/// property is actually driven by. Non-finite samples become the baseline rather
/// than poisoning the extent: a formula the artist is halfway through typing must
/// not blank the column.
#[must_use]
pub fn sample_window(stack: &RecipeStack) -> Vec<f32> {
    let src = stack.to_formula();
    let Ok(e) = ph2d_expr_parse::parse(&src) else {
        return vec![PREVIEW_VALUE; PREVIEW_SAMPLES];
    };
    struct B(f32);
    impl ph2d_expr::Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "value" => PREVIEW_VALUE,
                "time" => self.0,
                _ => 0.0,
            }
        }
        fn param(&self, _: &str) -> f32 {
            0.0
        }
    }
    (0..PREVIEW_SAMPLES)
        .map(|i| {
            let t = (i as f64 / PREVIEW_SAMPLES as f64) * PREVIEW_SECONDS;
            let v = ph2d_expr::eval(&e, &B(t as f32));
            if v.is_finite() { v } else { PREVIEW_VALUE }
        })
        .collect()
}

/// The figure the frame draws — chosen by the PROPERTY being driven.
///
/// ⚠️ One function, not a table (plano 10 P6 asked for this on `PropKind`; it
/// lives here instead because the panel is its only consumer and the model crate
/// has no business knowing what a thumbnail looks like). A second mapping would
/// let the frame draw a needle for a property the strip is plotting as a slide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Puppet {
    /// Slides along X.
    SlideX,
    /// Slides along Y.
    SlideY,
    /// A needle that turns.
    Needle,
    /// A box that grows on X.
    ScaleX,
    /// A box that grows on Y.
    ScaleY,
    /// A square that fades.
    Fade,
    /// A bar that fills — for a normalised `0..1` property.
    Bar,
}

#[must_use]
pub fn puppet_for(prop: PropKind) -> Puppet {
    match prop {
        PropKind::TranslationX | PropKind::Position => Puppet::SlideX,
        PropKind::TranslationY => Puppet::SlideY,
        PropKind::Rotation => Puppet::Needle,
        PropKind::ScaleX => Puppet::ScaleX,
        PropKind::ScaleY => Puppet::ScaleY,
        PropKind::Opacity => Puppet::Fade,
        // Morph is a normalised `0..1`; TimeRemap is a clock, and a clock has no
        // pose — a filling bar is the honest figure for both.
        _ => Puppet::Bar,
    }
}

/// The span the strip plots over, always non-degenerate.
///
/// ⚠️ A flat curve has zero extent, and normalising by zero is how a preview goes
/// blank on the most common formula there is (`value`, the empty sheet). The
/// baseline is always inside the span, so the dashed line has somewhere to sit.
#[must_use]
pub fn extent(samples: &[f32]) -> (f32, f32) {
    let mut lo = PREVIEW_VALUE;
    let mut hi = PREVIEW_VALUE;
    for &v in samples {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if (hi - lo).abs() < f32::EPSILON {
        (lo - 1.0, hi + 1.0)
    } else {
        let pad = (hi - lo) * CURVE_PAD_FRAC;
        (lo - pad, hi + pad)
    }
}

/// Paint the preview column: the puppet frame above, the curve strip below.
pub(crate) fn paint(
    scene: &mut VectorScene,
    theme: Theme,
    frame_rect: Rect,
    strip_rect: Rect,
    samples: &[f32],
    puppet: Puppet,
    frame: u32,
) {
    let (lo, hi) = extent(samples);
    let norm = |v: f32| ((v - lo) / (hi - lo)).clamp(0.0, 1.0); // CLAMP-OK: bounds literais nao-NaN e ordenados
    let ph = phase(frame);
    let at = samples
        .get(((ph * samples.len() as f32) as usize).min(samples.len().saturating_sub(1)))
        .copied()
        .unwrap_or(PREVIEW_VALUE);

    // ── The puppet frame ──
    stroke_rounded_rect(
        scene,
        frame_rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_puppet(scene, theme, frame_rect, puppet, norm(at));

    // ── The curve strip ──
    stroke_rounded_rect(
        scene,
        strip_rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    // The un-driven baseline, dashed: how far the formula departs from leaving the
    // property alone. Without it the curve is a squiggle with no reference.
    let base_y = strip_rect.y + strip_rect.h * (1.0 - norm(PREVIEW_VALUE));
    let dash = strip_rect.w / BASELINE_DASHES;
    let mut x = strip_rect.x;
    while x < strip_rect.x + strip_rect.w {
        let x2 = (x + dash * HALF).min(strip_rect.x + strip_rect.w);
        stroke_polyline(
            scene,
            &[(x, base_y), (x2, base_y)],
            StrokeToken::Thin.px(),
            resolve(ColorToken::Text2, theme),
        );
        x += dash;
    }
    // The output.
    let pts: Vec<(f32, f32)> = samples
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            (
                strip_rect.x + strip_rect.w * (i as f32 / (samples.len().max(2) - 1) as f32),
                strip_rect.y + strip_rect.h * (1.0 - norm(v)),
            )
        })
        .collect();
    stroke_polyline(
        scene,
        &pts,
        StrokeToken::Thin.px(),
        resolve(ColorToken::TimelinePlayhead, theme),
    );
    // The phase marker — what ties the two panes together: the puppet is AT this
    // instant of the curve.
    let mx = strip_rect.x + strip_rect.w * ph;
    stroke_polyline(
        scene,
        &[(mx, strip_rect.y), (mx, strip_rect.y + strip_rect.h)],
        StrokeToken::Thin.px(),
        resolve(ColorToken::Text2, theme),
    );
}

/// Draw the figure at normalised position `t` (`0..1` across the frame's span).
fn paint_puppet(scene: &mut VectorScene, theme: Theme, r: Rect, puppet: Puppet, t: f32) {
    let ink = resolve(ColorToken::TimelinePlayhead, theme);
    let pad = r.w.min(r.h) * PUPPET_PAD_FRAC;
    let inner = Rect::new(r.x + pad, r.y + pad, r.w - pad * 2.0, r.h - pad * 2.0);
    let unit = inner.w.min(inner.h) * PUPPET_UNIT_FRAC;
    let (cx, cy) = (inner.x + inner.w * HALF, inner.y + inner.h * HALF);

    match puppet {
        Puppet::SlideX => {
            let x = inner.x + inner.w * t;
            fill_rounded_rect(
                scene,
                Rect::new(x - unit * HALF, cy - unit * HALF, unit, unit),
                unit * HALF,
                ink,
            );
        }
        Puppet::SlideY => {
            let y = inner.y + inner.h * (1.0 - t);
            fill_rounded_rect(
                scene,
                Rect::new(cx - unit * HALF, y - unit * HALF, unit, unit),
                unit * 0.5,
                ink,
            );
        }
        Puppet::Needle => {
            // `t` spans the sampled range; one full turn across it reads as
            // rotation without pretending the numbers are degrees.
            let a = t * core::f32::consts::TAU;
            let (s, c) = (a.sin(), a.cos());
            let len = inner.w.min(inner.h) * NEEDLE_FRAC;
            stroke_polyline(
                scene,
                &[(cx, cy), (cx + c * len, cy - s * len)],
                StrokeToken::Thick.px(),
                ink,
            );
        }
        Puppet::ScaleX | Puppet::ScaleY => {
            let k = SCALE_MIN_FRAC + t * SCALE_SPAN_FRAC;
            let (w, h) = if puppet == Puppet::ScaleX {
                (inner.w * SCALE_LONG_FRAC * k, inner.h * SCALE_SHORT_FRAC)
            } else {
                (inner.w * SCALE_SHORT_FRAC, inner.h * SCALE_LONG_FRAC * k)
            };
            fill_rounded_rect(
                scene,
                Rect::new(cx - w * HALF, cy - h * HALF, w, h),
                Radius::Xs.px(),
                ink,
            );
        }
        Puppet::Fade => {
            let side = inner.w.min(inner.h) * FADE_SIDE_FRAC;
            let c = ink.multiply_alpha(t.clamp(FADE_FLOOR, 1.0)); // CLAMP-OK: bounds literais nao-NaN e ordenados
            fill_rounded_rect(
                scene,
                Rect::new(cx - side * HALF, cy - side * HALF, side, side),
                Radius::Xs.px(),
                c,
            );
        }
        Puppet::Bar => {
            let h = inner.h * BAR_H_FRAC;
            stroke_rounded_rect(
                scene,
                Rect::new(inner.x, cy - h * HALF, inner.w, h),
                Radius::Xs.px(),
                StrokeToken::Thin.px(),
                resolve(ColorToken::Border, theme),
            );
            fill_rounded_rect(
                scene,
                Rect::new(inner.x, cy - h * HALF, inner.w * t, h),
                Radius::Xs.px(),
                ink,
            );
        }
    }
}

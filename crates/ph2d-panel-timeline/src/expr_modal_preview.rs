//! **The Expression modal's live preview** (plano 10 W2) — the WAVE half.
//!
//! Two views, and both are needed:
//!
//! * the **curve strip**, which answers *what is it doing* — the value across the
//!   whole preview window, with the un-driven baseline dashed under it. It lives in
//!   the card, under the sheet, the whole card wide.
//! * the **ghost**, which answers *what will it look like* — and it lives in
//!   `expr_modal_stage`, on a card of its own beside this one.
//!
//! ⚠️ The strip is the half nobody copies. Blender's Drivers Editor plots the
//! MAPPING (driver in, property out) rather than the result, and seeing the
//! relation is what makes a driver comprehensible; every other product shows only
//! the outcome. A dot that moves tells you it moves. The strip tells you *how*.
//!
//! ⚠️ **One door, two views.** [`sample_window`] evaluates the stack across the
//! window ONCE per frame; the strip plots that vector and the ghost indexes it at
//! the current phase. A ghost that evaluated on its own would drift from the curve
//! drawn beside it, and the artist would have no way to know which one lies. The
//! two are on separate CARDS now, which makes the shared vector more load-bearing,
//! not less.
//!
//! ⚠️ **The clock is a FRAME COUNT**, and that is the house convention for chrome
//! (`ToastQueue::tick` ages by frames). The "never count ticks, count wall-clock"
//! law this repo learned in the impasto is about the ARTWORK — where the frame
//! rate would leak into what the artist produces. A thumbnail is not artwork, and
//! a frame count is what makes the preview reproducible in a gate.

use ph2d_editor_core::paint::{resolve, stroke_polyline, stroke_rounded_rect};
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
/// not in the panel's snapshot; when it becomes available this function is the ONE
/// place that changes, and the dashed baseline moves with it.
///
/// ⚠️ **Not one constant, because a resting property is not one number.** A
/// translation rests at `0` and a SCALE rests at `1` — an object at scale zero is
/// not a preview, it is an empty frame, and an opacity of zero is the same mistake
/// wearing another name. This is the only place in the modal that needs to know
/// which, and it needs to know because the ghost has to be visible before the
/// artist has done anything.
#[must_use]
pub fn preview_value(prop: PropKind) -> f32 {
    match prop {
        PropKind::ScaleX | PropKind::ScaleY | PropKind::Opacity => 1.0,
        _ => 0.0,
    }
}

// ── Proportions of the drawing itself ────────────────────────────────────────
//
// ⚠️ Every number below is a FRACTION of the box it is drawn in, not a measure:
// the strip has to read the same at any card width, so it is built from ratios and
// never from pixels. (The ghost's own proportions live with the ghost.)
/// Headroom added above and below the plotted curve.
const CURVE_PAD_FRAC: f32 = 0.1; // LITERAL-PX-OK: fracao do desenho, nao medida
/// Dashes across the baseline.
const BASELINE_DASHES: f32 = 24.0; // LITERAL-PX-OK: CONTAGEM de tracos, nao medida
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
pub fn sample_window(stack: &RecipeStack, base: f32) -> Vec<f32> {
    let src = stack.to_formula();
    let Ok(e) = ph2d_expr_parse::parse(&src) else {
        return vec![base; PREVIEW_SAMPLES];
    };
    struct B(f32, f32);
    impl ph2d_expr::Bindings for B {
        fn attr(&self, name: &str) -> f32 {
            match name {
                "value" => self.1,
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
            let v = ph2d_expr::eval(&e, &B(t as f32, base));
            if v.is_finite() { v } else { base }
        })
        .collect()
}

/// The span the strip plots over, always non-degenerate.
///
/// ⚠️ A flat curve has zero extent, and normalising by zero is how a preview goes
/// blank on the most common formula there is (`value`, the empty sheet). The
/// baseline is always inside the span, so the dashed line has somewhere to sit.
#[must_use]
pub fn extent(samples: &[f32], base: f32) -> (f32, f32) {
    let mut lo = base;
    let mut hi = base;
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

/// The sample the ghost stands on RIGHT NOW — the shared vector indexed at the
/// shared phase, which is the whole reason the two cards cannot disagree.
#[must_use]
pub fn sample_at(samples: &[f32], frame: u32, base: f32) -> f32 {
    let ph = phase(frame);
    samples
        .get(((ph * samples.len() as f32) as usize).min(samples.len().saturating_sub(1)))
        .copied()
        .unwrap_or(base)
}

/// Paint the wave strip: the value across the window, the baseline dashed under it,
/// and the phase marker that ties it to the ghost on the stage.
pub(crate) fn paint_strip(
    scene: &mut VectorScene,
    theme: Theme,
    rect: Rect,
    samples: &[f32],
    base: f32,
    frame: u32,
) {
    let (lo, hi) = extent(samples, base);
    let norm = |v: f32| ((v - lo) / (hi - lo)).clamp(0.0, 1.0); // CLAMP-OK: bounds literais nao-NaN e ordenados
    stroke_rounded_rect(
        scene,
        rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    // The un-driven baseline, dashed: how far the formula departs from leaving the
    // property alone. Without it the curve is a squiggle with no reference.
    let base_y = rect.y + rect.h * (1.0 - norm(base));
    let dash = rect.w / BASELINE_DASHES;
    let mut x = rect.x;
    while x < rect.x + rect.w {
        let x2 = (x + dash * HALF).min(rect.x + rect.w);
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
                rect.x + rect.w * (i as f32 / (samples.len().max(2) - 1) as f32),
                rect.y + rect.h * (1.0 - norm(v)),
            )
        })
        .collect();
    stroke_polyline(
        scene,
        &pts,
        StrokeToken::Thin.px(),
        resolve(ColorToken::TimelinePlayhead, theme),
    );
    // The phase marker — what ties the two CARDS together: the ghost on the stage
    // is at this instant of this curve.
    let mx = rect.x + rect.w * phase(frame);
    stroke_polyline(
        scene,
        &[(mx, rect.y), (mx, rect.y + rect.h)],
        StrokeToken::Thin.px(),
        resolve(ColorToken::Text2, theme),
    );
}

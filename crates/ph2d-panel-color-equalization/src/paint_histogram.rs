//! Color Equalization panel — RGB histogram overlay.
//!
//! Split out from [`crate::paint_sections`] to keep that file under the
//! Wave 10 panel-* LOC cap. Pure paint helper — no widget registration,
//! no interaction state, no panel chrome. The chrome (backing surface +
//! border) is drawn here too because the overlay is conceptually one
//! atomic widget.

use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
use ph2d_tool_color_equalization::algorithm::HistogramData;
use ph2d_vector::VectorScene;

/// Number of bars per RGB channel in the histogram overlay. 256 bins
/// down-sampled to 64 bars (4 bins per bar) keeps the painter snappy
/// while still showing meaningful distribution shape.
// LITERAL-PX-OK: visualization grid count, not a UI metric.
const HISTOGRAM_BARS: usize = 64;

/// Paint the RGB histogram overlay inside `rect`. Down-samples each
/// channel's 256 bins to [`HISTOGRAM_BARS`] bars and normalises bar
/// height to the max bin count across all three channels. Semantic
/// tokens proxy channel colours (`Danger` = R, `Success` = G, `Info`
/// = B) so the overlay respects the active theme.
///
/// When `hist` is `None` (host hasn't published one yet) only the empty
/// chrome frame is drawn.
pub(crate) fn paint_histogram_overlay(
    rect: Rect,
    hist: Option<&HistogramData>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    // Backing surface — Bg2 + thin BorderStrong stroke.
    let bg = resolve(ColorToken::Bg2, theme);
    let border = resolve(ColorToken::BorderStrong, theme);
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, bg);
    stroke_rounded_rect(scene, rect, radius, StrokeToken::Hairline.px(), border);

    let Some(h) = hist else {
        return;
    };
    if h.opaque_count == 0 {
        return;
    }

    // Down-sample each channel's 256 bins into HISTOGRAM_BARS buckets.
    // Each bucket sums `256 / HISTOGRAM_BARS` (= 4) bins.
    let bucket = 256 / HISTOGRAM_BARS;
    let mut r_buckets = [0u32; HISTOGRAM_BARS];
    let mut g_buckets = [0u32; HISTOGRAM_BARS];
    let mut b_buckets = [0u32; HISTOGRAM_BARS];
    for (i, b) in r_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.r[start..start + bucket].iter().sum();
    }
    for (i, b) in g_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.g[start..start + bucket].iter().sum();
    }
    for (i, b) in b_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.b[start..start + bucket].iter().sum();
    }
    // Global max across all channels so the bars share a y-scale.
    let max_bucket = r_buckets
        .iter()
        .chain(g_buckets.iter())
        .chain(b_buckets.iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);

    // Geometry: 1-px inset, bars span (HISTOGRAM_BARS) divisions.
    let inset = 2.0_f32;
    let inner = Rect::new(
        rect.x + inset,
        rect.y + inset,
        (rect.w - inset * 2.0).max(0.0),
        (rect.h - inset * 2.0).max(0.0),
    );
    let bar_w = inner.w / HISTOGRAM_BARS as f32;
    let bar_h_max = inner.h;

    let r_color = resolve(ColorToken::Danger, theme);
    let g_color = resolve(ColorToken::Success, theme);
    let b_color = resolve(ColorToken::Info, theme);
    let alpha = 0.6_f32; // LITERAL-PX-OK: histogram bar alpha (visual blend, not a UI metric)
    let r_color = with_alpha(r_color, alpha);
    let g_color = with_alpha(g_color, alpha);
    let b_color = with_alpha(b_color, alpha);

    for i in 0..HISTOGRAM_BARS {
        let x = inner.x + i as f32 * bar_w;
        let bar_w_filled = (bar_w - 0.5).max(0.5);
        // Each channel's bar height — overdraw with channel alpha so
        // overlapping regions show their additive sum.
        let hr = (r_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        let hg = (g_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        let hb = (b_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        if hr > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hr, bar_w_filled, hr),
                0.0,
                r_color,
            );
        }
        if hg > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hg, bar_w_filled, hg),
                0.0,
                g_color,
            );
        }
        if hb > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hb, bar_w_filled, hb),
                0.0,
                b_color,
            );
        }
    }
}

/// Reapply `alpha` to a token-resolved Color while keeping its RGB
/// components. Lets us draw semi-transparent histogram bars without
/// editing tokens.json. We deliberately avoid `Color::rgba8(...)` —
/// the channel hue still comes from the active theme via `resolve`.
fn with_alpha(color: ph2d_vector::Color, alpha: f32) -> ph2d_vector::Color {
    color.multiply_alpha(alpha.clamp(0.0, 1.0))
}

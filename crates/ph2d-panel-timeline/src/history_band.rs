//! **The recorded-history band** (Motion Nodes doc 132 §2): a thin strip along the ruler's
//! bottom edge, FULL where a producer holds the recorded history and ABSENT before it — the
//! idiom of Blender's physics-cache bar. A producer publishes the span as data, in timeline
//! seconds, once per frame (the Motion bridge publishes its input tape's `[base, last]`); the
//! panel never learns who.

use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Theme};
use std::cell::Cell;

thread_local! {
    static BAND: Cell<Option<(f64, f64)>> = const { Cell::new(None) };
}

/// The band's thickness, at the ruler's bottom edge.
const BAND_H: f32 = 3.0; // LITERAL-PX-OK: history band thickness (the loop brace's width)

/// Publish the recorded span `[start, end]` in timeline seconds — `None` when nothing is recorded.
pub fn set_recorded_history(band: Option<(f64, f64)>) {
    BAND.with(|b| b.set(band));
}

/// The span the producer published this frame.
#[must_use]
pub fn recorded_history() -> Option<(f64, f64)> {
    BAND.with(Cell::get)
}

/// Where the band lies under a ruler strip at `region` — `None` when off-screen or empty. Pure,
/// so «absent before the start» has an oracle a paint test cannot give it.
pub(crate) fn band_rect(
    region: Rect,
    ruler_h: f32,
    time_to_x: &impl Fn(f64) -> f32,
    band: Option<(f64, f64)>,
) -> Option<Rect> {
    let (a, b) = band?;
    let right = region.x + region.w;
    let x0 = safe_clamp(time_to_x(a), region.x, right);
    let x1 = safe_clamp(time_to_x(b), region.x, right);
    (x1 > x0).then(|| Rect::new(x0, region.y + ruler_h - BAND_H, x1 - x0, BAND_H))
}

/// Paint the published band on the ruler strip (timeline clock only — the caller gates it).
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    ruler_h: f32,
    time_to_x: &impl Fn(f64) -> f32,
) {
    if let Some(r) = band_rect(region, ruler_h, time_to_x, recorded_history()) {
        fill_rounded_rect(
            ctx.scene,
            r,
            ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
            resolve(ColorToken::Success, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The band spans exactly the published seconds and is ABSENT before its start; off-screen or
    /// unpublished, there is none.
    #[test]
    fn the_band_spans_the_published_history_and_nothing_before_it() {
        let region = Rect::new(10.0, 0.0, 600.0, 200.0);
        // 100 px per second, view starting at 0 s.
        let x = |t: f64| 10.0 + (t * 100.0) as f32;
        let r = band_rect(region, 22.0, &x, Some((2.0, 4.5))).expect("on screen");
        assert_eq!((r.x, r.w), (210.0, 250.0));
        assert_eq!(r.y + r.h, 22.0, "on the ruler's bottom edge");
        assert!(band_rect(region, 22.0, &x, None).is_none());
        assert!(
            band_rect(region, 22.0, &x, Some((7.0, 9.0))).is_none(),
            "off-screen"
        );
        set_recorded_history(Some((1.0, 2.0)));
        assert_eq!(recorded_history(), Some((1.0, 2.0)));
        set_recorded_history(None);
        assert_eq!(recorded_history(), None);
    }
}

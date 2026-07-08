//! Time ruler (W2.E3) — a scrubbable strip below the transport bar with second
//! ticks + labels, plus the playhead line crossing the whole body.
//!
//! Scrub is a 1D horizontal slider over the strip (the sanctioned panel pattern
//! for a 1-axis drag): the panel paints the ruler itself and registers the strip
//! as the `TIMELINE_RULER` slider hit; the generic dispatch maps the pointer x to
//! a `0..1` value + fires `ValueChanged`, which `event.rs` maps back to an
//! absolute time (via the view span `paint` stores in the panel state) and sends
//! as a Scrub. Adaptive tick density arrives with zoom (E6); E3 uses fixed 1 s
//! major / 0.5 s minor ticks at the default zoom.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::SliderState;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};

use crate::ids;
use crate::state::TimelinePanelState;

const RULER_H: f32 = 22.0; // LITERAL-PX-OK: ruler strip height
const TICK_MAJOR_H: f32 = 10.0; // LITERAL-PX-OK: labelled second tick height
const TICK_MINOR_H: f32 = 5.0; // LITERAL-PX-OK: half-second tick height
const PLAYHEAD_W: f32 = 2.0; // LITERAL-PX-OK: playhead line width
const TICK_MAJOR_S: f64 = 1.0; // LITERAL-PX-OK: labelled tick interval (seconds), not a UI metric
const TICK_MINOR_S: f64 = 0.5; // LITERAL-PX-OK: unlabelled tick interval (seconds), not a UI metric

/// Paint the ruler + playhead across `region` (the panel body below the
/// transport bar). Registers the ruler scrub hit and writes the view span into
/// `state` for `event.rs`.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let px_per_s = if state.px_per_s > 0.0 {
        state.px_per_s
    } else {
        crate::state::DEFAULT_PX_PER_S
    };
    let view_start = state.view_start_s;
    let span = f64::from(region.w) / px_per_s;
    state.view_span_s = span;

    let time_to_x = |t: f64| region.x + ((t - view_start) * px_per_s) as f32;

    // Ruler strip background.
    let strip = Rect::new(region.x, region.y, region.w, RULER_H);
    fill_rounded_rect(
        ctx.scene,
        strip,
        Radius::Xs.px(),
        resolve(ColorToken::Bg2, theme),
    );

    // Minor + major ticks (+ labels on majors).
    paint_ticks(
        ctx,
        theme,
        region,
        view_start,
        span,
        TICK_MINOR_S,
        TICK_MINOR_H,
        false,
        &time_to_x,
    );
    paint_ticks(
        ctx,
        theme,
        region,
        view_start,
        span,
        TICK_MAJOR_S,
        TICK_MAJOR_H,
        true,
        &time_to_x,
    );

    // Playhead line across the whole body (ruler + lanes area below).
    let px = time_to_x(snap.time_seconds);
    if px >= region.x && px <= region.x + region.w {
        let line = Rect::new(px - PLAYHEAD_W * 0.5, region.y, PLAYHEAD_W, region.h);
        fill_rounded_rect(
            ctx.scene,
            line,
            Radius::Xs.px(),
            resolve(ColorToken::Accent, theme),
        );
    }

    // Register the scrub hit (the strip) + mirror the playhead into the slider
    // value when the user isn't dragging, so the drag baseline tracks it.
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_RULER, strip);
    let dragging = matches!(
        ctx.host.store().slider(ids::TIMELINE_RULER),
        Some((SliderState::Dragging, _))
    );
    if !dragging && span > 0.0 {
        let v = ((snap.time_seconds - view_start) / span).clamp(0.0, 1.0) as f32;
        if let Some(InteractiveState::Slider { value, .. }) =
            ctx.host.store_mut().get_mut(ids::TIMELINE_RULER)
        {
            *value = v;
        }
    }
}

/// Paint ticks at `step`-second intervals across the visible span; labels on
/// majors (whole seconds).
#[allow(clippy::too_many_arguments)]
fn paint_ticks(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    view_start: f64,
    span: f64,
    step: f64,
    tick_h: f32,
    label: bool,
    time_to_x: &impl Fn(f64) -> f32,
) {
    if step <= 0.0 {
        return;
    }
    let first = (view_start / step).ceil();
    let last = ((view_start + span) / step).floor();
    let mut i = first;
    while i <= last {
        let t = i * step;
        let x = time_to_x(t);
        let tick = Rect::new(x, region.y + RULER_H - tick_h, 1.0, tick_h); // LITERAL-PX-OK: 1px tick
        fill_rounded_rect(
            ctx.scene,
            tick,
            Radius::Xs.px(),
            resolve(ColorToken::Text3, theme),
        );
        if label {
            let font = TypeToken::Sm.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                &format!("{}", t.round() as i64),
                x + Spacing::Xxs.px(),
                region.y,
                font,
                region.x + region.w - x,
                resolve(ColorToken::Text2, theme),
            );
        }
        i += 1.0;
    }
}

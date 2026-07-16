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

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::SliderState;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill};

use crate::ids;

pub(crate) const RULER_H: f32 = 22.0; // LITERAL-PX-OK: ruler strip height
const TICK_MAJOR_H: f32 = 10.0; // LITERAL-PX-OK: labelled second tick height
const TICK_MINOR_H: f32 = 5.0; // LITERAL-PX-OK: half-second tick height
const PLAYHEAD_W: f32 = 2.0; // LITERAL-PX-OK: playhead line width
const TICK_MAJOR_S: f64 = 1.0; // LITERAL-PX-OK: labelled tick interval (seconds), not a UI metric
const TICK_MINOR_S: f64 = 0.5; // LITERAL-PX-OK: unlabelled tick interval (seconds), not a UI metric
/// Visible width of a loop brace's vertical bar.
const BRACE_W: f32 = 3.0; // LITERAL-PX-OK: loop-brace bar width
/// Half-width of a brace's grab target — wider than the bar so it is easy to hit.
const BRACE_HIT_HW: f32 = 6.0; // LITERAL-PX-OK: loop-brace grab half-width
/// Width + height of a marker pennant. Wider than tall so the triangle reads as
/// a flag pinned to the time, not a thin spike.
const MARKER_W: f32 = 12.0; // LITERAL-PX-OK: marker pennant width
const MARKER_H: f32 = 8.0; // LITERAL-PX-OK: marker pennant height
/// Extra grab padding either side of a marker pennant.
const MARKER_HIT_PAD: f32 = 4.0; // LITERAL-PX-OK: marker grab padding

/// **What this ruler measures — and therefore what it may offer.**
///
/// The Arrange tab rules the TIMELINE: the playhead is the transport's, and every
/// gesture on the strip (scrub, loop braces, markers) edits the very thing it
/// points at.
///
/// The Keys tab rules the active CLIP. **Without a stack the two are one clock**,
/// so the Keys ruler is the ruler it has always been and everything works exactly
/// as before. Under a stack they are two clocks, and the clip's ruler goes
/// read-only — not out of caution, but because *the inverse does not exist*: a
/// looping strip maps many timeline instants onto the same clip instant, so
/// "scrub the clip to 2.0 s" names no single place to put the playhead. The loop
/// braces and the markers are the timeline's, and drawing them on the clip's ruler
/// would stand them at the wrong second.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerClock {
    /// Where the playhead stands **on this ruler**, or `None` when the clock it
    /// measures has no answer here (the clip is not playing, or plays twice).
    pub now: Option<f64>,
    /// This ruler measures the timeline, so it scrubs and carries the loop braces
    /// and the markers. False = a clip ruler under a stack: it reads, never writes.
    pub live: bool,
}

/// **Which clock this tab's ruler measures.** Pure, and tested as such.
///
/// The playhead is paint, not a widget — no hit index remembers it, so a seam test
/// can prove the ruler's *hits* and never its *line*. Answering here rather than
/// inside the paint loop is what gives that line an oracle at all: the same move
/// the strip's locked-grip refusal already made when it went to `hit_plan`.
pub(crate) fn clock_for(tab: crate::tab::Tab, snap: &TimelineViewSnapshot) -> RulerClock {
    if tab.shows_lanes() {
        // The Arrange tab IS the timeline: the transport's playhead, and every
        // gesture on the strip edits the thing it points at.
        return RulerClock {
            now: Some(snap.time_seconds),
            live: true,
        };
    }
    RulerClock {
        // The clip's own clock — which without a stack the snapshot publishes as the
        // playhead itself, so this is the ruler it has always been.
        now: snap.clip_time,
        live: !snap.stacked(),
    }
}

/// Paint the ruler strip (ticks + labels), the playhead line across `region`,
/// and register the scrub hit. The view (`view_start`, `px_per_s`) is computed
/// once in `paint.rs` (page-follow) and shared with the lanes so ticks and key
/// diamonds align.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    clock: RulerClock,
) {
    let span = f64::from(region.w) / px_per_s;
    let time_to_x = |t: f64| region.x + ((t - view_start) * px_per_s) as f32;

    // Ruler strip background.
    let strip = Rect::new(region.x, region.y, region.w, RULER_H);
    fill_rounded_rect(
        ctx.scene,
        strip,
        Radius::Xs.px(),
        resolve(ColorToken::TimelineRulerBg, theme),
    );
    // The loop's shaded band goes BEHIND the ticks + labels so it tints the strip
    // without hiding the time numbers. It is the TIMELINE's loop, so it only
    // belongs on a ruler that measures the timeline.
    if clock.live {
        paint_loop_band(ctx, theme, region, &time_to_x, snap);
    }

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

    // Playhead line across the whole body (ruler + lanes area below) — at the
    // instant THIS ruler's clock reads. `None` means that clock has no answer
    // here, and no line is the honest drawing: a playhead is a claim about where
    // you are, and under a stack a clip that is not playing has no "where".
    if let Some(now) = clock.now {
        let px = time_to_x(now);
        if px >= region.x && px <= region.x + region.w {
            let line = Rect::new(px - PLAYHEAD_W * 0.5, region.y, PLAYHEAD_W, region.h);
            fill_rounded_rect(
                ctx.scene,
                line,
                Radius::Xs.px(),
                resolve(ColorToken::TimelinePlayhead, theme),
            );
        }
    }

    // Everything below WRITES the timeline: the scrub, the loop braces, the
    // markers. A clip ruler under a stack registers none of it — not even dimmed.
    // The inverse map does not exist, so there is nothing honest for a drag here
    // to do, and a hit that is registered but meaningless is a control that lies
    // ([[feedback_disabled_button_still_dispatches]]).
    if !clock.live {
        return;
    }

    // Register the scrub hit (the strip) + mirror the playhead into the slider
    // value when the user isn't dragging, so the drag baseline tracks it.
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_RULER, strip);
    // The loop braces + markers go on TOP of the scrub strip — registered after
    // it so their grab targets win the hit where they overlap (`HitIndex::hit`
    // walks in reverse). Drawn last for the same reason visually.
    paint_loop_braces(ctx, theme, region, &time_to_x, snap);
    paint_markers(ctx, theme, region, &time_to_x, snap);
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

/// Paint just the loop range's shaded band. Called BEFORE the ticks so the
/// numbers stay legible over it. A no-op when no loop is set.
fn paint_loop_band(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
) {
    let Some((a, b)) = snap.loop_range else {
        return;
    };
    let right = region.x + region.w;
    let x0 = safe_clamp(time_to_x(a), region.x, right);
    let x1 = safe_clamp(time_to_x(b), region.x, right);
    if x1 > x0 {
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x0, region.y, x1 - x0, RULER_H),
            Radius::Xs.px(),
            resolve(ColorToken::TimelineLoopRegion, theme),
        );
    }
}

/// Paint the loop's two braces + register the three grab targets (start · end ·
/// body). Called AFTER the scrub hit so the braces win the overlap. A no-op when
/// no loop is set.
fn paint_loop_braces(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
) {
    let Some((a, b)) = snap.loop_range else {
        return;
    };
    let right = region.x + region.w;
    let x0 = safe_clamp(time_to_x(a), region.x, right);
    let x1 = safe_clamp(time_to_x(b), region.x, right);
    // Body grab (move the whole range) UNDER the edge handles, registered first so
    // the edges win where they overlap.
    if x1 - x0 > BRACE_HIT_HW * 2.0 {
        let body = Rect::new(
            x0 + BRACE_HIT_HW,
            region.y,
            x1 - x0 - BRACE_HIT_HW * 2.0,
            RULER_H,
        );
        register_brace(ctx, ids::timeline_loop_brace_id(2), 2, body);
    }
    // The two brace bars + their grab targets.
    for (edge, x, onscreen) in [
        (
            0u8,
            time_to_x(a),
            time_to_x(a) >= region.x - BRACE_HIT_HW && time_to_x(a) <= right,
        ),
        (
            1u8,
            time_to_x(b),
            time_to_x(b) >= region.x && time_to_x(b) <= right + BRACE_HIT_HW,
        ),
    ] {
        if !onscreen {
            continue;
        }
        let clamped = safe_clamp(x, region.x, right);
        fill_rounded_rect(
            ctx.scene,
            Rect::new(clamped - BRACE_W * 0.5, region.y, BRACE_W, RULER_H),
            Radius::Xs.px(),
            resolve(ColorToken::TimelineLoopBrace, theme),
        );
        // A little tick at the top, on the INNER side, so the brace reads as a
        // bracket [ or ] rather than a second playhead line.
        let tick_x = if edge == 0 {
            clamped + BRACE_W * 0.5
        } else {
            clamped - BRACE_W * 0.5 - BRACE_W
        };
        fill_rounded_rect(
            ctx.scene,
            Rect::new(tick_x, region.y, BRACE_W, StrokeToken::Thick.px()),
            Radius::Xs.px(),
            resolve(ColorToken::TimelineLoopBrace, theme),
        );
        let hit = Rect::new(
            clamped - BRACE_HIT_HW,
            region.y,
            BRACE_HIT_HW * 2.0,
            RULER_H,
        );
        register_brace(ctx, ids::timeline_loop_brace_id(edge), edge, hit);
    }
}

/// Paint a pennant + label per marker at the top of the ruler, and register each
/// as a `Marker` hit (keyed by storage index). Culls markers scrolled off-screen.
fn paint_markers(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    time_to_x: &impl Fn(f64) -> f32,
    snap: &TimelineViewSnapshot,
) {
    let right = region.x + region.w;
    let font = TypeToken::Xs.px();
    let half = MARKER_W * 0.5;
    for (index, (t, label)) in snap.markers.iter().enumerate() {
        let x = time_to_x(*t);
        if x < region.x - half || x > right + half {
            continue;
        }
        let color = resolve(ColorToken::TimelineMarker, theme);
        // The stem: a 1px line down the ruler strip, marking the exact time.
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x - 0.5, region.y, 1.0, RULER_H), // LITERAL-PX-OK: 1px marker stem
            Radius::Xs.px(),
            color,
        );
        // The pennant: a triangle whose apex points DOWN to the stem, base along
        // the top of the ruler — a flag pinned at the time.
        fill_triangle(
            ctx,
            [
                (x - half, region.y),
                (x + half, region.y),
                (x, region.y + MARKER_H),
            ],
            color,
        );
        // Label to the right of the triangle, elided so it never runs off.
        ph2d_editor_core::text_elide::paint_text_elided(
            ctx.text_system,
            ctx.scene,
            label,
            x + half + Spacing::Xs.px(),
            region.y + (MARKER_H - font) * 0.5,
            font,
            (right - x - half).max(0.0),
            resolve(ColorToken::Text2, theme),
        );
        // Grab target = the triangle only (top `MARKER_H`), never the stem below,
        // so clicking the LINE scrubs while clicking the TRIANGLE grabs the marker.
        let hit = Rect::new(
            x - half - MARKER_HIT_PAD,
            region.y,
            MARKER_W + MARKER_HIT_PAD * 2.0,
            MARKER_H,
        );
        let id = ids::timeline_marker_hit_id(index);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::Marker { index },
                canvas: hit,
            },
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// Fill a triangle through three points (the marker pennant).
fn fill_triangle(ctx: &mut PaintCtx, corners: [(f32, f32); 3], color: ph2d_vector::Color) {
    let mut p = BezPath::new();
    p.move_to((f64::from(corners[0].0), f64::from(corners[0].1)));
    p.line_to((f64::from(corners[1].0), f64::from(corners[1].1)));
    p.line_to((f64::from(corners[2].0), f64::from(corners[2].1)));
    p.close_path();
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &p,
    );
}

/// Register one loop brace as a timeline surface + hit rect.
fn register_brace(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, edge: u8, rect: Rect) {
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::LoopBrace { edge },
            canvas: rect,
        },
    );
    ctx.host.hit_index_mut().register(id, rect);
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
            resolve(ColorToken::TimelineRulerTick, theme),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::Tab;

    fn snap(stacked: bool, playhead: f64, clip_time: Option<f64>) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            time_seconds: playhead,
            clip_time,
            lanes: if stacked {
                vec![ph2d_timeline::LaneView {
                    name: "L".into(),
                    muted: false,
                    weight: 1.0,
                    mode: ph2d_timeline::LaneMode::Override,
                    strips: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            ..TimelineViewSnapshot::default()
        }
    }

    /// **Nothing changes without a stack** — the case an animator is in almost all of
    /// the time. The clip IS the timeline, so the Keys ruler shows the playhead and
    /// writes it, exactly as it always has.
    #[test]
    fn without_a_stack_both_tabs_rule_the_one_clock_there_is() {
        let s = snap(false, 1.25, Some(1.25));
        for tab in [Tab::Keys, Tab::Arrange] {
            assert_eq!(
                clock_for(tab, &s),
                RulerClock {
                    now: Some(1.25),
                    live: true
                },
                "{tab:?} lost the ruler it always had"
            );
        }
    }

    /// **The bug, as a number.** Playhead at timeline 4.0, the active clip reading 2.0.
    /// The Keys ruler must stand at 2.0 — over the clip's keys — and the Arrange ruler
    /// at 4.0. One ruler cannot be both.
    #[test]
    fn under_a_stack_each_tab_reads_its_own_clock() {
        let s = snap(true, 4.0, Some(2.0));
        assert_eq!(clock_for(Tab::Keys, &s).now, Some(2.0), "the CLIP's clock");
        assert_eq!(
            clock_for(Tab::Arrange, &s).now,
            Some(4.0),
            "the TIMELINE's clock"
        );
    }

    /// A clip that is not playing (or plays twice) has no instant to point at, and a
    /// playhead is a claim about where you are. No line is the honest drawing.
    #[test]
    fn a_clip_with_no_answer_gets_no_playhead() {
        let s = snap(true, 4.0, None);
        assert_eq!(clock_for(Tab::Keys, &s).now, None);
        assert_eq!(
            clock_for(Tab::Arrange, &s).now,
            Some(4.0),
            "the timeline always knows where it is"
        );
    }

    /// The clip's ruler under a stack never WRITES: the inverse map does not exist.
    /// The Arrange ruler always does.
    #[test]
    fn only_a_ruler_that_measures_the_timeline_may_write_it() {
        assert!(!clock_for(Tab::Keys, &snap(true, 4.0, Some(2.0))).live);
        assert!(clock_for(Tab::Keys, &snap(false, 4.0, Some(4.0))).live);
        for stacked in [false, true] {
            assert!(
                clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).live,
                "the Arrange tab IS the timeline"
            );
        }
    }
}

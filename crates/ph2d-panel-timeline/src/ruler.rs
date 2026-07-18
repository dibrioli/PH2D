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
/// Thickness of the visible move-handle LINE at the top of the loop band.
const LOOP_MOVE_LINE_H: f32 = 3.0; // LITERAL-PX-OK: loop move-handle line height
/// Height of the move-handle GRAB strip (top of the band). Grabbing here slides the
/// loop; a press BELOW it inside the band falls through to the scrub strip and seeks
/// the playhead — so the loop no longer swallows every click over it (Enio,
/// 2026-07-16). Kept well under [`RULER_H`] so most of the band scrubs.
const LOOP_MOVE_GRAB_H: f32 = 8.0; // LITERAL-PX-OK: loop move-handle grab height
/// Width + height of a marker pennant. Wider than tall so the triangle reads as
/// a flag pinned to the time, not a thin spike.
const MARKER_W: f32 = 12.0; // LITERAL-PX-OK: marker pennant width
const MARKER_H: f32 = 8.0; // LITERAL-PX-OK: marker pennant height
/// Extra grab padding either side of a marker pennant.
const MARKER_HIT_PAD: f32 = 4.0; // LITERAL-PX-OK: marker grab padding

/// **What this ruler measures — and therefore what it may offer.**
///
/// Both tabs SCRUB their own clock (the AE precomp model): Arrange scrubs the
/// timeline playhead, Keys scrubs the active clip's independent playhead — moving
/// it drives the soloed scene, which is how you author keys where you can see the
/// pose. The old "read-only clip ruler under a stack" is gone: it was a workaround
/// for not having an independent clip clock, and now there is one, so nothing needs
/// inverting.
///
/// **The LOOP is not furniture** — each tab draws its OWN loop (Arrange the
/// timeline's, Keys the clip's), because [`TimelineViewSnapshot::loop_range`] is
/// already the view-appropriate one, in this ruler's clock. Only the MARKERS
/// differ: they are timeline-time, so on the clip's ruler under a stack they would
/// stand at the wrong second, and the Keys ruler carries none there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerClock {
    /// Where the playhead stands **on this ruler**, or `None` when the clock it
    /// measures has no answer here (an Arrange-side clip that plays zero/two times).
    pub now: Option<f64>,
    /// This ruler scrubs — registers the scrub hit and mirrors the playhead into it.
    /// Both tabs do (the Keys ruler scrubs the clip clock).
    pub scrub: bool,
    /// This ruler carries the TIMELINE's markers. Arrange always; Keys only without
    /// a stack (then the clip IS the timeline). The loop is NOT gated by this — it is
    /// each view's own, drawn against the very clock this ruler scrubs.
    pub markers: bool,
}

/// **Which clock this tab's ruler measures.** Pure, and tested as such.
///
/// The playhead is paint, not a widget — no hit index remembers it, so a seam test
/// can prove the ruler's *hits* and never its *line*. Answering here rather than
/// inside the paint loop is what gives that line an oracle at all.
pub(crate) fn clock_for(tab: crate::tab::Tab, snap: &TimelineViewSnapshot) -> RulerClock {
    if tab.shows_lanes() {
        // Arrange IS the timeline: the transport's playhead, and it owns the markers
        // because they are drawn against the very clock it scrubs.
        return RulerClock {
            now: Some(snap.time_seconds),
            scrub: true,
            markers: true,
        };
    }
    // Keys scrubs the active clip's own clock (published as `clip_time`). It carries
    // the markers **only when there is no stack** — then the clip IS the timeline,
    // one clock, and the Keys ruler is the timeline ruler it has always been. Under a
    // stack the clip clock ≠ the timeline clock, so timeline markers would sit at the
    // wrong second and belong on the Arrange ruler only. Its LOOP, however, is the
    // clip's own and is always drawn.
    RulerClock {
        now: snap.clip_time,
        scrub: true,
        markers: !snap.stacked(),
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
    // without hiding the time numbers. Each view draws its OWN loop (`snap.loop_range`
    // is already the view-appropriate one, in this ruler's clock), so both tabs paint
    // it; a no-op when this view has no loop.
    paint_loop_band(ctx, theme, region, &time_to_x, snap);

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

    // A ruler that does not scrub registers nothing — a hit that is meaningless is
    // a control that lies ([[feedback_disabled_button_still_dispatches]]). Both tabs
    // scrub today, but the gate stays: it is what a future non-scrubbing ruler would
    // trip on.
    if clock.scrub {
        // Register the scrub hit (the strip) + mirror the playhead into the slider
        // value when the user isn't dragging, so the drag baseline tracks it. The
        // baseline is THIS ruler's clock (`clock.now`), not `time_seconds`: on the
        // Keys ruler those differ only during the one-frame lag after a tab switch,
        // but reading the wrong one would make the first drag frame jump.
        ctx.host
            .hit_index_mut()
            .register(ids::TIMELINE_RULER, strip);
        let dragging = matches!(
            ctx.host.store().slider(ids::TIMELINE_RULER),
            Some((SliderState::Dragging, _))
        );
        if !dragging
            && span > 0.0
            && let Some(now) = clock.now
        {
            let v = ((now - view_start) / span).clamp(0.0, 1.0) as f32;
            if let Some(InteractiveState::Slider { value, .. }) =
                ctx.host.store_mut().get_mut(ids::TIMELINE_RULER)
            {
                *value = v;
            }
        }
    }

    // The loop braces are each view's own — drawn on both tabs. They go on TOP of the
    // scrub strip so their grab targets win the hit where they overlap (`HitIndex::hit`
    // walks in reverse); drawn last for the same reason visually.
    paint_loop_braces(ctx, theme, region, &time_to_x, snap);
    // The markers ARE timeline furniture (timeline-time) — Arrange, or Keys without a
    // stack. On the clip's ruler under a stack they would stand at the wrong second.
    if clock.markers {
        paint_markers(ctx, theme, region, &time_to_x, snap);
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

/// The MOVE-handle grab strip for a loop drawn across `[x0, x1]` at the top of
/// `region`, or `None` when the band is too narrow to host one between its two edge
/// grabs. Callers register THIS as the body (`edge 2`) grab.
///
/// It is a THIN strip at the top, not the whole band: a press below it inside the
/// loop finds only the scrub strip and seeks the playhead, which is the whole point
/// (Enio, 2026-07-16 — the loop used to swallow every click over it). Pure, so the
/// "top strip, not the full ruler" rule has an oracle a paint test cannot give it.
fn loop_move_strip(region: Rect, x0: f32, x1: f32) -> Option<Rect> {
    (x1 - x0 > BRACE_HIT_HW * 2.0).then(|| {
        Rect::new(
            x0 + BRACE_HIT_HW,
            region.y,
            x1 - x0 - BRACE_HIT_HW * 2.0,
            LOOP_MOVE_GRAB_H,
        )
    })
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
    // the edges win where they overlap. It is only the TOP strip — the move-handle —
    // so clicking below it inside the loop scrubs (see `loop_move_strip`).
    if let Some(body) = loop_move_strip(region, x0, x1) {
        register_brace(ctx, ids::timeline_loop_brace_id(2), 2, body);
        // The visible move-handle line at the very top, so the grab reads as a handle
        // to drag rather than dead space. Drawn only where the grab exists — an
        // affordance you can see but not use is a lie.
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x0, region.y, x1 - x0, LOOP_MOVE_LINE_H),
            Radius::Xs.px(),
            resolve(ColorToken::TimelineLoopBrace, theme),
        );
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
    /// the time. The clip IS the timeline, so both tabs scrub the one clock there is
    /// AND both carry the markers: the Keys ruler is the timeline ruler it has always
    /// been.
    #[test]
    fn without_a_stack_both_tabs_rule_the_one_clock_there_is() {
        let s = snap(false, 1.25, Some(1.25));
        for tab in [Tab::Keys, Tab::Arrange] {
            assert_eq!(
                clock_for(tab, &s),
                RulerClock {
                    now: Some(1.25),
                    scrub: true,
                    markers: true
                },
                "{tab:?} lost the ruler it always had"
            );
        }
    }

    /// **The Keys ruler reads the CLIP's clock, the Arrange ruler the TIMELINE's.**
    /// The snapshot carries the clip playhead (`clip_time`) independently, so under a
    /// stack the Keys ruler stands at 2.0 (over the clip's keys) while Arrange is at
    /// 4.0. One ruler cannot be both.
    #[test]
    fn each_tab_reads_its_own_clock() {
        let s = snap(true, 4.0, Some(2.0));
        assert_eq!(clock_for(Tab::Keys, &s).now, Some(2.0), "the CLIP's clock");
        assert_eq!(
            clock_for(Tab::Arrange, &s).now,
            Some(4.0),
            "the TIMELINE's clock"
        );
    }

    /// **Both tabs SCRUB — the Keys ruler is no longer read-only under a stack.** The
    /// independent clip playhead is what a Keys scrub moves, so there is nothing to
    /// invert; the old read-only was a workaround for not having that clock (Enio,
    /// 2026-07-16: *"a playhead precisa ser movida no modo keys"*).
    #[test]
    fn both_tabs_scrub_their_own_clock() {
        for stacked in [false, true] {
            assert!(
                clock_for(Tab::Keys, &snap(stacked, 4.0, Some(2.0))).scrub,
                "the Keys ruler scrubs the clip clock, stacked={stacked}"
            );
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).scrub);
        }
    }

    /// **The markers follow the TIMELINE clock.** The Arrange ruler always has them.
    /// The Keys ruler has them ONLY without a stack (then it IS the timeline); under a
    /// stack the clip clock differs, so timeline markers would sit at the wrong second
    /// and belong on Arrange alone. The LOOP is NOT gated by this — each view draws its
    /// own (proven by the snapshot filling `loop_range` from the view's pair).
    #[test]
    fn the_markers_follow_the_timeline_clock() {
        // No stack: the Keys ruler is the timeline ruler, so it keeps the markers.
        assert!(clock_for(Tab::Keys, &snap(false, 4.0, Some(4.0))).markers);
        // Under a stack: only Arrange carries them.
        assert!(!clock_for(Tab::Keys, &snap(true, 4.0, Some(2.0))).markers);
        for stacked in [false, true] {
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).markers);
        }
    }

    /// **The loop's MOVE grab is a thin strip at the TOP of the band, not the whole
    /// ruler** — that is what leaves the rest of the band to the scrub, so a click
    /// below the move-line inside the loop seeks the playhead instead of sliding the
    /// loop (Enio, 2026-07-16). A band too narrow to host the strip between its edge
    /// grabs offers no move handle (resize only).
    #[test]
    fn the_loop_move_grab_is_a_top_strip_not_the_whole_ruler() {
        let region = Rect::new(10.0, 0.0, 400.0, RULER_H);
        // A wide loop: the move strip exists, sits at the top, and is far shorter
        // than the ruler (so most of the band scrubs).
        let strip = loop_move_strip(region, 100.0, 300.0).expect("wide loop has a move strip");
        assert_eq!(strip.y, region.y, "the strip is at the TOP of the band");
        assert_eq!(strip.h, LOOP_MOVE_GRAB_H, "grab height is the thin strip");
        assert!(
            strip.h < RULER_H * 0.5,
            "the strip leaves most of the band ({RULER_H}px) to the scrub, got {}",
            strip.h
        );
        // Inset from the edges by the edge-grab half-width, so the resize handles win.
        assert!(strip.x >= 100.0 + BRACE_HIT_HW && strip.w <= 200.0 - BRACE_HIT_HW * 2.0);
        // A band narrower than the two edge grabs has no room for a move handle.
        assert!(
            loop_move_strip(region, 100.0, 100.0 + BRACE_HIT_HW).is_none(),
            "a too-narrow loop offers resize only, no move strip"
        );
    }
}

//! The clip-stack lanes: a row per lane, a rectangle per strip (ADR-0115 B1).
//!
//! **The crossfade is drawn from the same numbers the evaluator weights by.** The
//! snapshot's `blend_in`/`blend_out` come straight off `ClipLane::blend_in/out` —
//! the panel never re-derives them. So the wedge you see IS the blend you hear;
//! they cannot drift apart, which is the failure mode a "draw it approximately"
//! shortcut would have shipped.
//!
//! Nothing here knows what an overlap is either. Two strips whose rectangles touch
//! simply have a blend window at that end, because the lane says so.

use std::borrow::Cow;

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::TimeView;
use crate::state::TimelinePanelState;
use crate::{geom, ids};

/// Vertical inset of a strip inside its row, so lanes read as separate rows.
const STRIP_PAD_Y: f32 = 3.0; // LITERAL-PX-OK: a strip must not touch its row's edge
/// Grab width of a strip's trim edges.
const EDGE_W: f32 = 6.0; // LITERAL-PX-OK: a pointer-sized grip, like the loop brace's
/// Grab size of the ease handle at a strip's top corner (B4).
const EASE_W: f32 = 7.0; // LITERAL-PX-OK: a pointer-sized grip, like the trim edge's
/// The ease handle's resting inset from the corner, when the strip has NO fade yet.
///
/// It rests just PAST the trim grip instead of on the corner, so the two grips never
/// share a pixel: a handle stacked on the trim edge would have to win the click (it is
/// registered later), and the artist reaching for the trim would author a fade instead.
/// Dragging it back to here means zero fade.
const EASE_REST_X: f32 = EDGE_W; // LITERAL-PX-OK: exactly clear of the trim grip
/// Below this, a rate is real time and the strip says nothing about it.
const SPEED_EPS: f64 = 1e-6; // LITERAL-PX-OK: not a length — a "is it exactly 1" epsilon
/// The lane weight field's width.
const WEIGHT_W: f32 = 38.0; // LITERAL-PX-OK: "1.00" at TypeToken::Sm, plus the field's padding
/// How much of the label column the lane's controls take (weight + two buttons).
const CONTROLS_W: f32 = WEIGHT_W + (ROW_H_PX - STRIP_PAD_Y * 2.0) * 2.0 + 12.0; // LITERAL-PX-OK: three Xs gaps
/// A lane's weight step per drag notch, and the decimals it shows.
const WEIGHT_STEP: f64 = 0.01; // LITERAL-PX-OK: 1% of the [0, 1] range
/// Decimals shown in the weight field.
const WEIGHT_DECIMALS: usize = 2; // LITERAL-PX-OK: 1% resolution needs two

/// The "+ Lane" button, beside "+ Track" in the label column's header strip.
pub(crate) fn paint_add_lane(ctx: &mut PaintCtx, theme: Theme, header: Rect) {
    let st = ctx
        .host
        .store()
        .button_state(ids::TIMELINE_ADD_LANE)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(
        ids::TIMELINE_ADD_LANE,
        ph2d_i18n::tr("panel.timeline.add_lane"),
    )
    .state(st);
    paint_button(&btn, header, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_LANE, header);
}

/// Paint every lane of the clip stack, above the Summary channel and the tracks.
/// A no-op when the document has no stack — which is the norm.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let region = g.rows;
    let bands: Vec<(usize, f32, f32)> = geom::stack_bands(snap, region.y, state.scroll_y).collect();
    for (i, y, h) in bands {
        // Scrolled out of the band: neither paint nor leave hits behind.
        if y + h <= region.y || y >= region.y + region.h {
            continue;
        }
        paint_lane(
            ctx,
            theme,
            g,
            view,
            snap,
            i,
            Rect::new(region.x, y, region.w, h),
        );
    }
}

/// One lane: its label slice, then its strips.
#[allow(clippy::too_many_lines)] // the row IS the layout: splitting it hides the arithmetic
fn paint_lane(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    snap: &TimelineViewSnapshot,
    index: usize,
    row: Rect,
) {
    let lane = &snap.lanes[index];
    fill_rounded_rect(
        ctx.scene,
        row,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );

    // ── the label column: name, and what the lane DOES ──
    let font = TypeToken::Sm.px();
    let text_y = row.y + (row.h - font) * 0.5;
    let dim = lane.muted || lane.weight <= 0.0;
    // The name's budget stops at the controls, not at the column's edge — text that
    // elides UNDER a button looks like a button with a word behind it.
    let name_w = (g.label_w - CONTROLS_W - Spacing::Sm.px() * 2.0).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &lane.name,
        row.x + Spacing::Sm.px(),
        text_y,
        font,
        name_w,
        resolve(
            if dim {
                ColorToken::Text3
            } else {
                ColorToken::Text1
            },
            theme,
        ),
    );

    // ── the lane's controls, right-aligned in the label column ──
    //
    // The WEIGHT is here and the MODE is in the right-click menu, and that split is
    // the point: weight is a number you nudge and re-nudge while watching the
    // canvas, so it belongs under the pointer; mode is set once per lane. Delete
    // Lane is in the menu too — not because deleting is rare, but because a row
    // this narrow has no room for a third button, and a control that only appears
    // when the column is wide enough is worse than one that is always one click in.
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let btn_y = row.y + STRIP_PAD_Y;
    let add = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        btn_y,
        btn_w,
        btn_w,
    );
    let mute = Rect::new(add.x - btn_w - Spacing::Xs.px(), btn_y, btn_w, btn_w);
    let weight = Rect::new(
        mute.x - WEIGHT_W - Spacing::Xs.px(),
        row.y + STRIP_PAD_Y,
        WEIGHT_W,
        (row.h - STRIP_PAD_Y * 2.0).max(0.0),
    );
    // Every hit in the label column is clipped to the visible band: a lane scrolled
    // half under the ruler must not register its mute where "+ Lane" is painted.
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_weight(
        ctx,
        theme,
        ids::TIMELINE_LANE_WEIGHT[index],
        weight,
        band,
        lane,
    );
    paint_lane_button(
        ctx,
        theme,
        ids::TIMELINE_LANE_MUTE[index],
        mute,
        band,
        lane.muted,
    );
    paint_lane_button(
        ctx,
        theme,
        ids::TIMELINE_LANE_ADD_STRIP[index],
        add,
        band,
        false,
    );

    // The label itself is the right-click surface (mode, Delete Lane). Its rect
    // STOPS where the controls start, so it cannot steal their clicks — the hit
    // index does not need a z-order rule it would only have to remember.
    let label = Rect::new(row.x, row.y, (weight.x - row.x).max(0.0), row.h);
    if let Some(r) = clipped(label, band) {
        ctx.host.store_mut().register(
            ids::TIMELINE_LANE_ROW[index],
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::LaneHeader { lane: index },
                canvas: row,
            },
        );
        ctx.host
            .hit_index_mut()
            .register(ids::TIMELINE_LANE_ROW[index], r);
    }

    // ── the strips ──
    //
    // Two passes, and the order is load-bearing. The hit index resolves to the
    // LAST id registered over a point, and a lane's strips MAY OVERLAP — that is
    // the crossfade. Registering each strip whole (body, then its edges) before
    // the next one puts the right strip's BODY on top of the left strip's END
    // edge, which is precisely the edge you reach for to tune the crossfade you
    // just made. Bodies first for the whole lane, edges after, and every edge
    // outranks every body.
    let mut boxes: Vec<(&ph2d_timeline::StripView, Rect)> = Vec::new();
    for s in &lane.strips {
        let (x0, x1) = (view.x(s.t_start), view.x(s.t_end));
        // Off-screen, or collapsed to nothing: no rect, no hit.
        if x1 <= view.time_x || x0 >= view.right || x1 - x0 < 1.0 {
            continue;
        }
        let body = Rect::new(
            x0,
            row.y + STRIP_PAD_Y,
            (x1 - x0).max(0.0),
            (row.h - STRIP_PAD_Y * 2.0).max(0.0),
        );
        paint_strip(ctx, theme, view, body, s, dim);
        boxes.push((s, body));
    }
    // Clipped to the TIME area, never the label column: a strip that starts left of
    // the view has an x0 far off to the left, and an unclipped body rect would
    // blanket the lane's name, weight, mute and "+ strip" — and, registered after
    // them, win every one of their clicks.
    let time_band = Rect::new(
        view.time_x,
        g.rows.y,
        (view.right - view.time_x).max(0.0),
        g.rows.h,
    );
    let spans: Vec<(u64, Rect)> = boxes.iter().map(|(s, b)| (s.id.0, *b)).collect();
    // The ease grips (B4) — only where the strip owns the edge. Where a neighbour
    // overlaps it, the OVERLAP is the fade (Unity's rule): the grip is painted greyed
    // and gets **no hit** at all, because a dimmed control that still dispatches is a
    // control that lies ([[feedback_disabled_button_still_dispatches]]).
    let eases: Vec<(u64, u8, Rect, bool)> = boxes
        .iter()
        .flat_map(|(s, body)| {
            [
                (EASE_IN, s.blend_in, s.ease_locked_in),
                (EASE_OUT, s.blend_out, s.ease_locked_out),
            ]
            .into_iter()
            .filter_map(|(edge, blend, locked)| {
                let px = blend_px(view, s.t_start, blend);
                ease_grip(*body, px, edge).map(|r| (s.id.0, edge, r, locked))
            })
            .collect::<Vec<_>>()
        })
        .collect();
    for (strip, edge, rect) in hit_plan(&spans, &eases, time_band) {
        put_strip_hit(ctx, row, rect, index, strip, edge);
    }
}

/// The blend window's width in pixels — the wedge the panel already draws, measured.
fn blend_px(view: TimeView, t_start: f64, blend: f64) -> f32 {
    view.x(t_start + blend) - view.x(t_start)
}

/// Hit/paint code for the fade-in grip (the strip's start corner).
const EASE_IN: u8 = 3;
/// …and the fade-out grip (its end corner).
const EASE_OUT: u8 = 4;

/// **Where the ease handle sits**: at the TIP of the wedge, on the strip's top edge —
/// so the thing you grab is the thing you see, and dragging it is dragging the fade.
///
/// `None` when the strip is too narrow to hold both trim grips AND the two handles: on
/// a strip that small the handles would sit on top of the trim edges and steal them (an
/// ease grip outranks a trim grip in the hit order). A strip you cannot fade at this
/// zoom is honest; a strip you can no longer TRIM is a bug.
///
/// With no fade authored, the tip is at the corner — where the trim grip already is —
/// so the handle RESTS one grip-width in ([`EASE_REST_X`]). Dragging it back there means
/// zero fade, which is exactly where it came from.
fn ease_grip(body: Rect, blend_px: f32, edge: u8) -> Option<Rect> {
    // Both trim grips + both handles, side by side, with nothing overlapping.
    if body.w < (EASE_REST_X + EASE_W) * 2.0 {
        return None;
    }
    let reach = blend_px.max(EASE_REST_X);
    let x = if edge == EASE_IN {
        (body.x + reach).min(body.x + body.w - EASE_REST_X - EASE_W)
    } else {
        (body.x + body.w - reach - EASE_W).max(body.x + EASE_REST_X)
    };
    // Top band only: the rest of the strip's height stays the BODY's, so the slide
    // gesture is not shrunk to a sliver by a grip that only needs a corner.
    let h = (body.h * 0.5).min(EASE_W);
    Some(Rect::new(x, body.y, EASE_W, h))
}

/// **The order the strip hits are registered in — and it is load-bearing.**
///
/// The hit index resolves a point to the LAST id registered over it, and a lane's
/// strips MAY OVERLAP: that is the crossfade. Registering each strip whole (body,
/// then its own edges) put the right strip's BODY on top of the left strip's END
/// edge — precisely the grip you reach for to tune the crossfade you just made.
/// So: every body first, every edge after. An edge always outranks a box.
///
/// Pure, and tested, because no headless paint can reach it: the defect lived in
/// the order of two loops and no `apply_event` test could ever have seen it.
///
/// The ease grips (B4) come LAST, and only over their own strip's body: a fade handle
/// sits ON the body it belongs to, so it has to outrank it. It never lands on a trim
/// grip — [`ease_grip`] declines the handle entirely on a strip too narrow to hold
/// both — which is why "last" is safe here and would not be otherwise.
///
/// **A LOCKED ease grip is painted and not registered** (`locked` = a neighbour's overlap
/// defines that fade, so the number is not the strip's to author). The refusal lives HERE,
/// in the pure function, and not in the paint loop that calls it — a rule that only exists
/// where no test can reach it is a rule that will be deleted by someone tidying up, and the
/// artist would find out by dragging a handle that silently writes a number nobody reads
/// ([[feedback_disabled_button_still_dispatches]]).
fn hit_plan(
    spans: &[(u64, Rect)],
    eases: &[(u64, u8, Rect, bool)],
    band: Rect,
) -> Vec<(u64, u8, Rect)> {
    let mut out = Vec::with_capacity(spans.len() * 3 + eases.len());
    for &(id, body) in spans {
        if let Some(r) = clipped(body, band) {
            out.push((id, 2, r));
        }
    }
    for &(id, body) in spans {
        let w = EDGE_W.min(body.w * 0.5);
        for (edge, rect) in [
            (0, Rect::new(body.x, body.y, w, body.h)),
            (1, Rect::new(body.x + body.w - w, body.y, w, body.h)),
        ] {
            // The WHOLE grip must be inside the band, not merely overlap it: a
            // sliver of an edge poking into view is not something a pointer can aim
            // at, and the start edge of a strip that begins off-screen left would
            // otherwise be trimmable blind.
            if rect.x >= band.x
                && rect.x + rect.w <= band.x + band.w
                && let Some(r) = clipped(rect, band)
            {
                out.push((id, edge, r));
            }
        }
    }
    for &(id, edge, rect, locked) in eases {
        // Same rule as the trim grips: a handle half-visible at the edge of the view is
        // not one a pointer can aim at. And a locked one is not grabbable at all.
        if !locked
            && rect.x >= band.x
            && rect.x + rect.w <= band.x + band.w
            && let Some(r) = clipped(rect, band)
        {
            out.push((id, edge, r));
        }
    }
    out
}

/// The part of `r` the panel can actually show — its intersection with the band.
///
/// A hit rect that reaches outside the band is a control you cannot see and can
/// still click. A lane scrolled half under the ruler would otherwise register its
/// mute where "+ Lane" is painted, and a strip that starts left of the view would
/// register its body across the whole label column, on top of every control there.
/// Both are the same bug, and this is the one place it dies.
fn clipped(r: Rect, band: Rect) -> Option<Rect> {
    let (x0, y0) = (r.x.max(band.x), r.y.max(band.y));
    let (x1, y1) = (
        (r.x + r.w).min(band.x + band.w),
        (r.y + r.h).min(band.y + band.h),
    );
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// The lane's weight, as a bounded number field: drag it, or type into it.
///
/// The range is REGISTERED (`set_number_range`), not merely clamped afterwards —
/// dispatch scales a body-drag by the field's range, so a field that never
/// declared one drags at the default scale and a lane's `[0, 1]` weight would fly
/// past both ends in a few pixels ([[reference_number_input_register_range]]).
fn paint_weight(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    lane: &ph2d_timeline::LaneView,
) {
    use ph2d_editor_core::widget::{NumberInput, paint_number_input_with_buffer, showcase};
    {
        let store = ctx.host.store_mut();
        crate::transport::mirror_number(store, id, lane.weight, WEIGHT_DECIMALS);
        store.set_number_range(id, 0.0, 1.0, WEIGHT_STEP);
    }
    let (state, _v, buf, caret, anchor) = showcase::read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", lane.weight)
        .step(WEIGHT_STEP)
        .state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        r,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// One of a lane's header controls. Square, and `on` fills it — the mute reads as
/// pressed, and a lane that is muted is dimmed everywhere else too.
fn paint_lane_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    on: bool,
) {
    fill_rounded_rect(
        ctx.scene,
        r,
        Radius::Xs.px(),
        resolve(
            if on {
                ColorToken::Accent
            } else {
                ColorToken::Bg3
            },
            theme,
        ),
    );
    stroke_rounded_rect(
        ctx.scene,
        r,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    ctx.host.store_mut().register(id, InteractiveState::Plain);
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// One strip: its box, its name, and the two blend wedges — which are the
/// crossfade, drawn from the lane's own numbers.
fn paint_strip(
    ctx: &mut PaintCtx,
    theme: Theme,
    view: TimeView,
    body: Rect,
    s: &ph2d_timeline::StripView,
    dim: bool,
) {
    fill_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        resolve(
            if dim {
                ColorToken::Bg3
            } else {
                ColorToken::TimelineKey
            },
            theme,
        ),
    );
    stroke_rounded_rect(
        ctx.scene,
        body,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::BorderStrong, theme),
    );

    // The blend windows. A window at an end means the strip is fading there — into
    // its neighbour if it has one (the overlap), out of nothing if it does not.
    // Drawn as a darker wedge, so the eye reads the crossfade the ear will hear.
    let wedge = resolve(ColorToken::Bg3, theme);
    for (t_from, secs) in [
        (s.t_start, s.blend_in),
        (s.t_end - s.blend_out, s.blend_out),
    ] {
        if secs <= 0.0 {
            continue;
        }
        let a = view.x(t_from).max(body.x);
        let b = view.x(t_from + secs).min(body.x + body.w);
        if b <= a {
            continue;
        }
        fill_rounded_rect(
            ctx.scene,
            Rect::new(a, body.y, b - a, body.h),
            Radius::Xs.px(),
            wedge,
        );
    }

    // The ease grips (B4), at the tip of each wedge. GREYED where a neighbour defines
    // the window — there the overlap IS the fade, and the way to change it is to move
    // the strips; the grip is painted so the artist can SEE that the edge is spoken
    // for, and it is not registered, so it cannot be dragged into a number nobody reads.
    for (edge, blend, locked) in [
        (EASE_IN, s.blend_in, s.ease_locked_in),
        (EASE_OUT, s.blend_out, s.ease_locked_out),
    ] {
        let Some(g) = ease_grip(body, blend_px(view, s.t_start, blend), edge) else {
            continue;
        };
        fill_rounded_rect(
            ctx.scene,
            g,
            Radius::Xs.px(),
            resolve(
                if locked || dim {
                    ColorToken::Border
                } else {
                    ColorToken::TimelinePlayhead
                },
                theme,
            ),
        );
    }

    // The rate leads the name. A retimed strip is pixel-identical to a trimmed one
    // — the box says how long it plays, never how fast — so the surprising fact
    // goes FIRST, where elision cannot eat it on a narrow strip. The guessable one
    // (which clip) can be cut.
    let label: Cow<'_, str> = if (s.speed - 1.0).abs() > SPEED_EPS {
        Cow::Owned(format!("{:.2}\u{d7} {}", s.speed, s.clip_name))
    } else {
        Cow::Borrowed(s.clip_name.as_str())
    };
    let font = TypeToken::Sm.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &label,
        body.x + Spacing::Xs.px(),
        body.y + (body.h - font) * 0.5,
        font,
        (body.w - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
}

/// One strip hit target. `edge`: `0` = start, `1` = end, `2` = body.
fn put_strip_hit(ctx: &mut PaintCtx, row: Rect, rect: Rect, lane: usize, strip: u64, edge: u8) {
    let id = ids::timeline_strip_hit_id(lane as u64, strip, edge);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Strip { lane, strip, edge },
            canvas: row,
        },
    );
    ctx.host.hit_index_mut().register(id, rect);
}

#[cfg(test)]
#[path = "stack_lane_paint_tests.rs"]
mod tests;

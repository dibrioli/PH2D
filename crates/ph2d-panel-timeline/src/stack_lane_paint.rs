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
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &lane.name,
        row.x + Spacing::Sm.px(),
        text_y,
        font,
        (g.label_w - Spacing::Sm.px() * 2.0).max(0.0),
        resolve(
            if dim {
                ColorToken::Text3
            } else {
                ColorToken::Text1
            },
            theme,
        ),
    );

    // ── the lane's two controls, right-aligned in the label column ──
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let btn_y = row.y + STRIP_PAD_Y;
    let add = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        btn_y,
        btn_w,
        btn_w,
    );
    let mute = Rect::new(add.x - btn_w - Spacing::Xs.px(), btn_y, btn_w, btn_w);
    paint_lane_button(ctx, theme, ids::TIMELINE_LANE_MUTE[index], mute, lane.muted);
    paint_lane_button(ctx, theme, ids::TIMELINE_LANE_ADD_STRIP[index], add, false);

    // ── the strips ──
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
        register_hits(ctx, row, body, index, s.id.0);
    }
}

/// One of a lane's header controls. Square, and `on` fills it — the mute reads as
/// pressed, and a lane that is muted is dimmed everywhere else too.
fn paint_lane_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
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
    ctx.host.hit_index_mut().register(id, r);
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

    let font = TypeToken::Sm.px();
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &s.clip_name,
        body.x + Spacing::Xs.px(),
        body.y + (body.h - font) * 0.5,
        font,
        (body.w - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
}

/// The three grab targets: body, then the two edges LAST so they win the overlap
/// (the convention the resize grips already state — outermost-last).
fn register_hits(ctx: &mut PaintCtx, row: Rect, body: Rect, lane: usize, strip: u64) {
    let mut put = |edge: u8, rect: Rect| {
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
    };
    put(2, body);
    let w = EDGE_W.min(body.w * 0.5);
    put(0, Rect::new(body.x, body.y, w, body.h));
    put(1, Rect::new(body.x + body.w - w, body.y, w, body.h));
}

//! Timeline panel paint.
//!
//! Visibility gate → resolve the panel rect (the user-resized one, else the
//! bottom dock from `ctx.layout.timeline`) → publish it for dispatch → canonical
//! dark-glass chrome → transport bar → dope sheet (label column + ruler + lanes
//! + scrollbar).
//!
//! **Two-pass geometry.** `interact::process` runs BEFORE the rect is finalized,
//! against the rect and `time_x` this frame started with, so a zoom/pan/resize
//! lands on the same frame it is dragged instead of one frame late. Then the rect
//! is re-resolved (a resize may have moved it) and everything paints from that.

use crate::state::{self, TimelinePanelState, set_last_content_h, set_last_visible_h};
use crate::{TimelinePanel, geom, ids, ruler, scrollbar, tracks, transport};
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, paint_panel_close_button, paint_panel_corner_dot,
    paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::zones::Rect;

pub(crate) fn paint(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TimelinePanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning the panel
        // (and `dispatch_wheel` stops zooming its time axis) once it is hidden.
        ctx.host.store_mut().clear_panel_rect(ids::TIMELINE_PANEL);
        ctx.host.store_mut().clear_timeline_canvas();
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();
    let viewport = ctx.layout.viewport;
    let docked = ctx.layout.timeline;
    if state.px_per_s <= 0.0 {
        state.px_per_s = state::DEFAULT_PX_PER_S;
    }

    // Pass 1: drain this frame's wheel + gestures against the rect we start with.
    let rect0 = geom::clamp_to(state.rect.unwrap_or(docked), viewport);
    crate::interact::process(state, ctx, rect0, geom::time_x(rect0), viewport, &snapshot);

    // Pass 2: a resize may have just moved the panel — paint from the new rect.
    let rect = geom::clamp_to(state.rect.unwrap_or(docked), viewport);
    ctx.host
        .store_mut()
        .set_panel_rect(ids::TIMELINE_PANEL, rect);
    // Resize grippers FIRST, so every later hit (close button, lanes, keys) wins
    // over the border strips where they overlap.
    register_resize_grips(ctx, rect);

    // Canonical dark-glass chrome — identical to the other docked panels.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        "Timeline",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::TIMELINE_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body = geom::body(rect, title_size);
    let after_transport = transport::paint_bar(ctx, theme, body, &snapshot);
    let g = geom::resolve(rect, after_transport);

    // The WHOLE dope sheet takes the wheel (not just the time column) so a wheel
    // over the label names still scrolls the rows.
    ctx.host
        .store_mut()
        .set_timeline_canvas(ids::TIMELINE_PANEL, g.region);

    // Measure the scroll range, then clamp the model into it.
    let content_h = geom::content_h(snapshot.tracks.len());
    state.scroll_max = geom::scroll_max(snapshot.tracks.len(), g.rows.h);
    state.scroll_y = state.scroll_y.clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max

    // Resolve the time view (after the wheel landed) and share it with the ruler
    // + lanes so ticks and key diamonds align.
    let px_per_s = state.px_per_s;
    let span = f64::from(g.time_area.w) / px_per_s;
    let mut view_start = state.view_start_s;
    // Page-follow ONLY while playing: the view chases the playhead during
    // playback, but a manual pan/zoom while paused is never yanked back.
    if snapshot.playing
        && span > 0.0
        && (snapshot.time_seconds < view_start || snapshot.time_seconds >= view_start + span)
    {
        view_start = snapshot.time_seconds.max(0.0);
    }
    state.view_start_s = view_start;
    state.view_span_s = span;

    let preview_dx = crate::interact::preview_dx(state, px_per_s, &snapshot);

    // "+Track" button in the label column, aligned with the ruler strip.
    let header = Rect::new(g.region.x, g.region.y, g.label_w, ruler::RULER_H);
    tracks::paint_add_track(ctx, theme, header);
    // Track rows (labels + key diamonds) below the ruler strip.
    tracks::paint_rows(
        ctx,
        theme,
        g.rows,
        g.label_w,
        g.time_area.x,
        view_start,
        px_per_s,
        preview_dx,
        state.scroll_y,
        &snapshot,
    );
    scrollbar::paint(ctx, theme, g.scrollbar, state, content_h);
    // Time axis last, so ticks + playhead overlay the rows.
    ruler::paint(ctx, theme, g.time_area, view_start, px_per_s, &snapshot);

    // "+Track" property dropdown overlay — painted last so it sits on top.
    tracks::paint_add_track_popover(ctx, theme, header, state.add_track_open);

    set_last_content_h(content_h);
    set_last_visible_h(g.rows.h);
}

/// Register the eight edge/corner grippers as `TimelineSurface` hits so dispatch
/// streams their drag to `interact::apply_resize`. Invisible by design — the
/// panel border is the affordance.
fn register_resize_grips(ctx: &mut PaintCtx, rect: Rect) {
    for (id, edges, r) in geom::resize_grips(rect) {
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::ResizeEdge { edges },
                canvas: rect,
            },
        );
        ctx.host.hit_index_mut().register(id, r);
    }
}

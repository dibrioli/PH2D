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

use crate::geom::SPLIT_GRIP;
use crate::state::{self, TimelinePanelState, set_last_content_h, set_last_visible_h};
use crate::{TimelinePanel, geom, graph, ids, ruler, scrollbar, tracks, transport};
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, paint_panel_close_button, paint_panel_corner_dot,
    paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

/// Thickness of the hairline drawn on the label/time seam.
const SPLIT_LINE_W: f32 = 1.0; // LITERAL-PX-OK: splitter hairline width

pub(crate) fn paint(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TimelinePanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning the panel
        // (and `dispatch_wheel` stops zooming its time axis) once it is hidden.
        ctx.host.store_mut().clear_panel_rect(ids::TIMELINE_PANEL);
        ctx.host.store_mut().clear_timeline_canvas();
        // Drop any in-flight gesture: hiding the panel mid-drag must not leave a
        // marquee to resolve (or repaint) when it comes back.
        state.box_drag = None;
        state.box_commit = None;
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
    let time_x0 = geom::time_x(rect0, state.label_w);
    crate::interact::process(state, ctx, rect0, time_x0, viewport, &snapshot);

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
    let g = geom::resolve(rect, after_transport, state.label_w);
    // Write the clamped width back, so a drag that ran past the bounds does not
    // have to be dragged all the way back before the column moves again.
    state.label_w = g.label_w;

    // The WHOLE dope sheet takes the wheel (not just the time column) so a wheel
    // over the label names still scrolls the rows.
    ctx.host
        .store_mut()
        .set_timeline_canvas(ids::TIMELINE_PANEL, g.region);

    // Measure the scroll range, then clamp the model into it.
    state.graph_h = graph::clamp_graph_h(state.graph_h);
    let content_h = geom::content_h(&snapshot, &state.expanded, state.graph_h);
    state.scroll_max = geom::scroll_max(content_h, g.rows.h);
    state.scroll_y = state.scroll_y.clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max

    // `F` fits the time axis to the keys; it needs the time area's pixel width,
    // known only now. Runs before the view is read, so the fit lands this frame.
    if state::take_fit_request() {
        crate::view::apply_fit(state, g.time_area.w, &snapshot);
    }
    // A transport jump (go-to-start/end, frame step, typed time) pans the view
    // after it, so the playhead never lands off-screen while paused.
    if state::take_reveal_request() {
        crate::view::reveal_time(state, g.time_area.w, snapshot.time_seconds);
    }

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

    // The time axis, shared by the ruler, the lanes and every expanded graph.
    let view = graph::TimeView {
        time_x: g.time_area.x,
        right: g.rows.x + g.rows.w,
        view_start,
        px_per_s,
    };
    // A box-select that ended this frame resolves HERE, where the rows' `y` is
    // finally known (the gesture only recorded the marquee).
    crate::box_select::commit(state, g.rows, view, &snapshot);

    let preview_dx = crate::interact::preview_dx(state, px_per_s, &snapshot);

    // "+Track" button in the label column, aligned with the ruler strip.
    let header = Rect::new(g.region.x, g.region.y, g.label_w, ruler::RULER_H);
    tracks::paint_add_track(ctx, theme, header);
    // Track rows (labels + key diamonds + expanded graph bands) below the ruler.
    tracks::paint_rows(ctx, theme, &g, view, preview_dx, state, &snapshot);
    // A handle drag whose row got culled (scrolled away, or its track unbound)
    // never reached `graph::resolve_drag`: close its undo bracket here so the
    // next atomic edit is not silently swallowed into it.
    if state.handle_drag.is_some_and(|d| d.ending) {
        state.handle_drag = None;
        state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
    }
    // The live box-select rubber band rides over the diamonds it is picking.
    if let Some(b) = state.box_drag {
        tracks::paint_marquee(ctx, theme, b.rect());
    }
    scrollbar::paint(ctx, theme, g.scrollbar, state, content_h);
    // Time axis last, so ticks + playhead overlay the rows.
    ruler::paint(ctx, theme, g.time_area, view_start, px_per_s, &snapshot);

    // The label/time splitter sits over the lanes AND the ruler, so it is
    // registered after both — a grab on the seam must not scrub the ruler.
    paint_label_splitter(
        ctx,
        theme,
        g.region,
        g.region.x + g.label_w,
        state.label_drag.is_some(),
    );

    // "+Track" property dropdown overlay — painted last so it sits on top.
    tracks::paint_add_track_popover(ctx, theme, header, state.add_track_open);

    set_last_content_h(content_h);
    set_last_visible_h(g.rows.h);
}

/// The seam between the track-name column and the time area: a hairline, plus a
/// wider invisible strip to grab it by. Highlights while dragging.
fn paint_label_splitter(ctx: &mut PaintCtx, theme: Theme, region: Rect, x: f32, active: bool) {
    let line = Rect::new(x - SPLIT_LINE_W * 0.5, region.y, SPLIT_LINE_W, region.h);
    let tok = if active {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    fill_rounded_rect(ctx.scene, line, 0.0, resolve(tok, theme));

    let grip = Rect::new(x - SPLIT_GRIP, region.y, SPLIT_GRIP * 2.0, region.h);
    ctx.host.store_mut().register(
        ids::TIMELINE_LABEL_SPLIT,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::LabelSplitter,
            canvas: grip,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_LABEL_SPLIT, grip);
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

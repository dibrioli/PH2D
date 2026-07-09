//! Timeline panel paint (W2.E0 scaffold).
//!
//! Visibility gate → bottom-dock rect from `ctx.layout.timeline` → publish the
//! rect for dispatch → canonical dark-glass chrome (surface + corner dots +
//! title + X close). The transport bar, ruler and dope-sheet lanes land in
//! W2.E2+; for now the body is empty (it reads the published
//! [`TimelineViewSnapshot`] so the wiring is exercised).

use crate::state::{self, TimelinePanelState, set_last_content_h, set_last_visible_h};
use crate::{TimelinePanel, ids, ruler, tracks, transport};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

pub(crate) fn paint(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TimelinePanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning the panel
        // once it is hidden.
        ctx.host.store_mut().clear_panel_rect(ids::TIMELINE_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.timeline;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::TIMELINE_PANEL, rect);

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

    // Body: the transport row, then the ruler + lanes region below it.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Sm.px();
    let body = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        body_top,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0),
    );
    let after_transport = transport::paint_bar(ctx, theme, body, &snapshot);
    // Dope-sheet region below the transport bar: a left label column (+Track /
    // track names) and a time area (ruler + lanes) to its right.
    let region = Rect::new(
        body.x,
        after_transport,
        body.w,
        (body.y + body.h - after_transport).max(0.0),
    );
    let label_w = tracks::LABEL_COL_W.min(region.w);
    let time_area = Rect::new(
        region.x + label_w,
        region.y,
        (region.w - label_w).max(0.0),
        region.h,
    );
    // Compute the time view ONCE (page-follow) and share it with the ruler +
    // lanes so ticks and key diamonds align. When the playhead leaves the
    // visible span (e.g. free playback), view_start jumps to it (E6 refines).
    let px_per_s = if state.px_per_s > 0.0 {
        state.px_per_s
    } else {
        state::DEFAULT_PX_PER_S
    };
    let span = f64::from(time_area.w) / px_per_s;
    let mut view_start = state.view_start_s;
    if span > 0.0
        && (snapshot.time_seconds < view_start || snapshot.time_seconds >= view_start + span)
    {
        view_start = snapshot.time_seconds.max(0.0);
    }
    state.view_start_s = view_start;
    state.view_span_s = span;

    // Drain this frame's dope-sheet gestures (select / drag-move / clear) before
    // drawing, so the drag preview offset is current for the diamonds below.
    crate::interact::process(state, ctx, px_per_s, &snapshot);
    let preview_dx = crate::interact::preview_dx(state, px_per_s, &snapshot);

    // "+Track" buttons in the label column, aligned with the ruler strip.
    tracks::paint_add_track(
        ctx,
        theme,
        Rect::new(region.x, region.y, label_w, ruler::RULER_H),
    );
    // Track rows (labels + key diamonds) below the ruler strip.
    tracks::paint_rows(
        ctx,
        theme,
        Rect::new(
            region.x,
            region.y + ruler::RULER_H,
            region.w,
            (region.h - ruler::RULER_H).max(0.0),
        ),
        label_w,
        time_area.x,
        view_start,
        px_per_s,
        preview_dx,
        &snapshot,
    );
    // Time axis last, so ticks + playhead overlay the rows.
    ruler::paint(ctx, theme, time_area, view_start, px_per_s, &snapshot);

    // "+Track" property dropdown overlay — painted last so it sits on top.
    tracks::paint_add_track_popover(
        ctx,
        theme,
        Rect::new(region.x, region.y, label_w, ruler::RULER_H),
        state.add_track_open,
    );

    set_last_content_h(0.0);
    set_last_visible_h(rect.h);
}

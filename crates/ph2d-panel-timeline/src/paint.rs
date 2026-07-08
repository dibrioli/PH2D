//! Timeline panel paint (W2.E0 scaffold).
//!
//! Visibility gate → bottom-dock rect from `ctx.layout.timeline` → publish the
//! rect for dispatch → canonical dark-glass chrome (surface + corner dots +
//! title + X close). The transport bar, ruler and dope-sheet lanes land in
//! W2.E2+; for now the body is empty (it reads the published
//! [`TimelineViewSnapshot`] so the wiring is exercised).

use crate::state::{self, TimelinePanelState, set_last_content_h, set_last_visible_h};
use crate::{TimelinePanel, ids, transport};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

pub(crate) fn paint(_state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
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

    // Body: the transport row (ruler + dope-sheet lanes land in E3+).
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Sm.px();
    let body = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        body_top,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0),
    );
    transport::paint_bar(ctx, theme, body, &snapshot);

    set_last_content_h(0.0);
    set_last_visible_h(rect.h);
}

//! The panel body. Walks [`crate::rows::SECTIONS`] — the same table `populate`
//! and `event` walk — so what is drawn, what is registered and what dispatches
//! cannot disagree.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    PHYSICS_SCROLLBAR_ID, paint_scrollbar, paint_slider_with_chip_layout_adaptive,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use crate::state::{self, PhysicsPanelState, set_last_content_h, set_last_visible_h};
use crate::{PhysicsPanel, rows};

mod body;
mod interact;
mod joint;
mod matrix;

/// Label gutter. Wider than the padding panel's because these labels are words
/// ("Sub-steps", "Contact Hz"), not single edges.
pub(crate) const LABEL_COL_W: f32 = 78.0; // LITERAL-PX-OK: panel grid metric (label gutter width)

pub(crate) fn paint(_state: &mut PhysicsPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PhysicsPanel::ID) {
        // Symmetric stale-rect cleanup, so `panel_at` stops returning
        // PHYSICS_PANEL the moment the panel is closed.
        ctx.host.store_mut().clear_panel_rect(ids::PHYSICS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    let snapshot = state::current();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::PHYSICS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles. Reuse the Inspector ids because the
    // right dock slot is shared — the resize delta should persist when the
    // artist switches between Inspector and a world panel.
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ids::INSP_RESIZE_HANDLE, panel_resize_handle_rect(rect));
        hit_index.register(
            ids::INSP_RESIZE_HANDLE_BL,
            panel_resize_handle_rect_bl(rect),
        );
    }

    let title_size = paint_panel_title(
        rect,
        tr("panel.physics.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::PHYSICS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::PHYSICS_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = body::paint_sections(
        ctx,
        &snapshot,
        rect.x + PANEL_HEAD_PAD,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        body_top - scroll,
    );
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);
}

/// One slider+chip row, from the table.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_row(
    ctx: &mut PaintCtx,
    row: &rows::Row,
    value: f32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // The published value is the truth; the stored track is only what the drag
    // left behind. Seeding the track from the value every frame is what keeps
    // the slider honest when the settings change from anywhere else (Reset to
    // Defaults, a project load, an undo).
    let track = row.track_of(value);
    let display = f64::from(value);
    let text = format!("{display:.*}", row.decimals);
    let label = tr(row.label);

    paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        label,
        track,
        display,
        Some(&text),
        row.slider,
        row.chip,
        LABEL_COL_W,
        ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

/// One Interaction slider+chip row, from the Interaction table. The twin of
/// [`paint_row`] — the two tables have the same row shape and different owners,
/// so this is a delegation rather than a second layout.
pub(crate) fn paint_irow(
    ctx: &mut PaintCtx,
    row: &crate::interact::IRow,
    value: f32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let track = row.track_of(value);
    let display = f64::from(value);
    let text = format!("{display:.*}", row.decimals);
    paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        tr(row.label),
        track,
        display,
        Some(&text),
        row.slider,
        row.chip,
        LABEL_COL_W,
        ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == ids::PHYSICS_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(PHYSICS_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::PHYSICS_PANEL, content_h);
    store.set_panel_visible_h(ids::PHYSICS_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::PHYSICS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::PHYSICS_PANEL, max_scroll);
    }
}

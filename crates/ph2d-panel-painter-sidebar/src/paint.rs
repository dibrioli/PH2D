//! Painter sidebar paint — T2.1 Day-7 functional + chrome canon.
//!
//! Render canon (mirror BgRemoval/Padding sidebar pattern):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_sidebar`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Painter"
//!   + close (X) button (PANEL_HEADER_CLOSE_RESERVE)
//! - Drag handle + 2 resize handles (Inspector slot shared canon)
//! - 2 sliders via `paint_slider_with_chip_layout_adaptive` (label demota
//!   pra linha própria em dock estreito — iPad portrait):
//!   * Size (0..1 → px display via `ph2d_tool_painter::size01_to_px`)
//!   * Opacity (0..1 → 0..100% display)
//! - Body inside scroll clip; `content_h` / `visible_h` publish
//!
//! W2.T2.2 (undo/redo buttons) e T2.4 (modifier square) virão em commits
//! seguintes.

use crate::PainterSidebarPanel;
use crate::state::{self, PainterSidebarPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::paint_slider_with_chip_layout_adaptive;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

const SLIDER_LABEL_W: f32 = 70.0;
const SLIDER_CHIP_W: f32 = 64.0;

pub(crate) fn paint(_state: &mut PainterSidebarPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PainterSidebarPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_SIDEBAR_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_sidebar;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_SIDEBAR_PANEL, rect);

    // Chrome: dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared canon — Inspector right-dock
    // slot persistence). BgRemoval/Padding pattern.
    {
        let drag_rect = panel_drag_handle_rect(
            rect,
            ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
            PANEL_HEADER_CLOSE_RESERVE,
        );
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ph2d_editor_core::ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    // Title — reserve room pra close button.
    let title_size = paint_panel_title(
        rect,
        "Painter",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Close (X) button — routes pra CancelActiveTool (canon BgRemoval).
    paint_panel_close_button(
        rect,
        core_ids::PAINTER_SIDEBAR_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // Body region (clipped) — sliders dentro.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let row_pad = Spacing::Md.px();

    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let mut y = body_top;

    // Size slider — size_px display via display_override + SSOT map.
    // Adaptive layout (audit W-4): demotes the label to its own row when
    // the dock is too narrow (iPad portrait) instead of collapsing the
    // track to a ~1 px sliver.
    let size_px = ph2d_tool_painter::size01_to_px(snapshot.size01);
    let size_display = format!("{size_px:.0} px");
    let size_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let size_h = paint_slider_with_chip_layout_adaptive(
        size_rect,
        "Size",
        snapshot.size01,
        size_px as f64,
        Some(&size_display),
        core_ids::PAINTER_SIDEBAR_SIZE_SLIDER,
        core_ids::PAINTER_SIDEBAR_SIZE_CHIP,
        SLIDER_LABEL_W,
        SLIDER_CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    y += size_h + row_pad;

    // Opacity slider (display 0..100%, SSOT map).
    let opacity_pct = ph2d_tool_painter::opacity01_to_pct(snapshot.opacity01);
    let opacity_display = format!("{opacity_pct:.0}%");
    let opacity_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let opacity_h = paint_slider_with_chip_layout_adaptive(
        opacity_rect,
        "Opacity",
        snapshot.opacity01,
        opacity_pct as f64,
        Some(&opacity_display),
        core_ids::PAINTER_SIDEBAR_OPACITY_SLIDER,
        core_ids::PAINTER_SIDEBAR_OPACITY_CHIP,
        SLIDER_LABEL_W,
        SLIDER_CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    y += opacity_h + row_pad;

    let content_h = (y - body_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Bottom-LEFT resize corner dot (mirror canon BR). Painted AFTER body
    // widgets pra ficar visualmente em cima de qualquer drift no canto.
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Re-register close button no fim do frame pra scrolled body widgets
    // não shadowarem o close (canon panel_chrome doc).
    ctx.host.hit_index_mut().register(
        core_ids::PAINTER_SIDEBAR_CLOSE,
        panel_close_button_rect(rect),
    );
}

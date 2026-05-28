//! Painter sidebar paint — T2.1 Day-7 functional.
//!
//! Render canon (mirror BgRemoval/Padding sidebar pattern):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_sidebar`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Painter"
//! - 2 sliders via `paint_slider_with_chip_layout`:
//!   * Size (0..1 → 1..2048 px display via `display_override`)
//!   * Opacity (0..1 → 0..100% display)
//! - `content_h` / `visible_h` publish pra scroll bounds
//!
//! W2.T2.2 (undo/redo buttons) e T2.4 (modifier square) virão em commits
//! seguintes.

use crate::PainterSidebarPanel;
use crate::state::{self, PainterSidebarPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::paint_slider_with_chip_layout;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_H_DEFAULT, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

const SIZE_MAX_PX: f32 = 2048.0;
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

    // Chrome: dark-glass surface + corner accent + title.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_title(rect, "Painter", 0.0, ctx.scene, ctx.text_system, theme);

    // Body layout — y-cursor convention.
    let mut y = rect.y + PANEL_HEADER_H_DEFAULT;
    let row_pad = Spacing::Md.px();

    // Size slider (size_px display via display_override).
    let size_px = (snapshot.size01.clamp(0.0, 1.0) * (SIZE_MAX_PX - 1.0)) + 1.0;
    let size_display = format!("{size_px:.0} px");
    let size_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_slider_with_chip_layout(
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
    y += ROW_H_PX + row_pad;

    // Opacity slider (display 0..100%).
    let opacity_pct = snapshot.opacity01.clamp(0.0, 1.0) * 100.0;
    let opacity_display = format!("{opacity_pct:.0}%");
    let opacity_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_slider_with_chip_layout(
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
    y += ROW_H_PX + row_pad;

    // Content + visible heights pra scroll bounds.
    set_last_content_h(y - rect.y);
    set_last_visible_h(rect.h);
}

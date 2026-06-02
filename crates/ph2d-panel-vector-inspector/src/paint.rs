//! Vector inspector paint — minimal v1: chrome + fill-color swatch.

use crate::VectorInspectorPanel;
use crate::state::{self, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_surface, paint_panel_title, panel_close_button_rect,
};
use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, paint_color_swatch};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

const FILL_LABEL_W: f32 = 80.0; // LITERAL-PX-OK: "Fill" label column width

pub(crate) fn paint(_state: &mut crate::VectorInspectorPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(VectorInspectorPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::VECTOR_INSPECTOR_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    // Right-dock geometry (shared Inspector slot; the shell hides the real
    // Inspector while a vector selection tool is active).
    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::VECTOR_INSPECTOR_PANEL, rect);

    // Chrome.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        "Vector",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        core_ids::VECTOR_INSPECTOR_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // Body — a single "Fill" row: label + right-pinned picker swatch.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;
    let font = TypeToken::Base.px();
    let y = body_top;

    paint_text(
        ctx.text_system,
        ctx.scene,
        "Fill",
        rect.x + PANEL_HEAD_PAD,
        y + (ROW_H_PX - font) * 0.5,
        font,
        FILL_LABEL_W,
        resolve(ColorToken::Text1, theme),
    );
    let swatch_w = SwatchSize::Md.px();
    let swatch_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD + content_w - swatch_w,
        y,
        swatch_w,
        ROW_H_PX,
    );
    let swatch = ColorSwatch::new(
        core_ids::VECTOR_INSPECTOR_FILL_SWATCH,
        "Fill color",
        state::current_fill(),
    )
    .size(SwatchSize::Md);
    paint_color_swatch(&swatch, swatch_rect, ctx.scene, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::VECTOR_INSPECTOR_FILL_SWATCH, swatch_rect);
    // Generic picker dispatch: Down on this swatch opens the Blender picker.
    ctx.host
        .store_mut()
        .register_picker_swatch(core_ids::VECTOR_INSPECTOR_FILL_SWATCH);

    let content_h = (y + ROW_H_PX + PANEL_HEAD_PAD - body_top).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    // Re-register close chrome after the body (last-registered-wins).
    ctx.host.hit_index_mut().register(
        core_ids::VECTOR_INSPECTOR_CLOSE,
        panel_close_button_rect(rect),
    );
}

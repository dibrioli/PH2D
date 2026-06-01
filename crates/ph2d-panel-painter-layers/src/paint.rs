//! Painter layers panel paint — SCAFFOLD chrome + placeholder body.
//!
//! Render canon (mirror do `ph2d-panel-painter-sidebar` paint):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_layers`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Layers"
//!   + close (X) button (PANEL_HEADER_CLOSE_RESERVE)
//! - Drag handle + 2 resize handles (Inspector slot shared canon)
//! - Body clipado com placeholder ("No layers") até o Implementador
//!   preencher as layer rows reais.
//!
//! **SCAFFOLD (Coordenador):** este arquivo entrega o chrome real + o
//! body placeholder. O Implementador substitui o placeholder pelas layer
//! rows (thumb + name + visibility + opacity slider + blend dropdown) no
//! ponto marcado `// TODO(impl W3.T3.4): layer rows` abaixo.

use crate::PainterLayersPanel;
use crate::state::{self, PainterLayersPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, rect_to_vello};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::paint::resolve;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

pub(crate) fn paint(_state: &mut PainterLayersPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PainterLayersPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_LAYERS_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_layers;
    let theme = ctx.host.theme();

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_LAYERS_PANEL, rect);

    // Chrome: dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared canon — Inspector right-dock
    // slot persistence, mirror do sidebar). Parentam aos handles do
    // Inspector slot (single dock-slot persistence).
    {
        let drag_rect = panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    // Title — reserve room pra close button.
    let title_size = paint_panel_title(
        rect,
        "Layers",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Close (X) button — routes pra CancelActiveTool (canon BgRemoval).
    paint_panel_close_button(
        rect,
        core_ids::PAINTER_LAYERS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // Body region (clipped).
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);

    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let mut y = body_top;

    // ── PLACEHOLDER body ──────────────────────────────────────────────
    // SCAFFOLD: renders a single "No layers" text row (or the live layer
    // count when the shell publishes a snapshot) so the panel is visibly
    // alive. The Implementer replaces THIS block with the real layer rows.
    //
    // TODO(impl W3.T3.4): layer rows
    //   Read the published stack via `state::current_layers()` (Option<
    //   ph2d_tool_painter::LayerStack>). For each layer in root z-order
    //   (top→bottom), paint a row:
    //     thumb + name + visibility toggle + opacity slider + blend dropdown.
    //   Use `ph2d_painter_brush::{BlendMode, MAX_BLEND_MODES}` for the blend
    //   popover order (T3.3). Register each row's interactive widgets in
    //   `populate.rs` + classify their events in `event.rs` (ToolPanelEvent
    //   canal genérico, mirror do sidebar). Add the per-row NodeIds in
    //   editor-core `ids.rs` alongside `PAINTER_LAYERS_*`.
    let label_font = TypeToken::Base.px();
    let placeholder = match state::current_layers() {
        Some(stack) => {
            let n = stack.len();
            if n == 0 {
                "No layers".to_string()
            } else {
                format!("{n} layer(s) — rows pending (W3.T3.4)")
            }
        }
        None => "No layers".to_string(),
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &placeholder,
        rect.x + PANEL_HEAD_PAD,
        y,
        label_font,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text2, theme),
    );
    y += label_font + Spacing::Md.px();
    // ──────────────────────────────────────────────────────────────────

    let content_h = (y - body_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Bottom-LEFT resize corner dot (mirror canon BR).
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Re-register close button no fim do frame pra scrolled body widgets
    // não shadowarem o close (canon panel_chrome doc).
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LAYERS_CLOSE, panel_close_button_rect(rect));
}

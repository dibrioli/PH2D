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
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{LayerId, LayerKind, LayerStack};

// Read-only layer-row metrics (W3.T3.4 1st pass). Component-specific layout,
// not global Spacing steps.
const LAYER_META_W: f32 = 96.0; // LITERAL-PX-OK: right column for "Blend · NN%" meta text
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children

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

    // Layer rows (W3.T3.4, read-only 1st pass) — render the published runtime
    // `LayerStack` snapshot top→bottom, recursing groups. INTERACTIVE controls
    // (visibility toggle / opacity slider / blend dropdown) + the composite-
    // preview that reflects them are the next block, gated on the tool↔
    // LayerStack integration (see HANDOFF_painter_w3_layerstack_divergence_coord.md).
    match state::current_layers() {
        Some(stack) if !stack.is_empty() => {
            let content_w = rect.w - PANEL_HEAD_PAD * 2.0;
            y = paint_layer_subtree(
                ctx,
                theme,
                &stack,
                stack.root(),
                0,
                rect.x + PANEL_HEAD_PAD,
                content_w,
                y,
            );
        }
        _ => {
            let font = TypeToken::Base.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                "No layers",
                rect.x + PANEL_HEAD_PAD,
                y,
                font,
                rect.w - PANEL_HEAD_PAD * 2.0,
                resolve(ColorToken::Text2, theme),
            );
            y += font + Spacing::Md.px();
        }
    }

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

/// Paint the layers in `ids` (top→bottom) as read-only rows, recursing into
/// groups (indented; a collapsed group hides its children). Returns the `y`
/// advanced past the painted rows.
///
/// Read-only (W3.T3.4 1st pass): no hit registration / event wiring yet —
/// the visibility toggle / opacity slider / blend dropdown interactions land
/// in the next block once the tool↔LayerStack integration is ratified.
#[allow(clippy::too_many_arguments)]
fn paint_layer_subtree(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    stack: &LayerStack,
    ids: &[LayerId],
    depth: usize,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let row_gap = Spacing::Xs.px();
    let cell_gap = Spacing::Sm.px();
    let thumb_w = Spacing::Xl2.px();
    let indent = depth as f32 * LAYER_INDENT_STEP;
    for &id in ids {
        let Some(layer) = stack.get(id) else { continue };
        let row_x = x + indent;
        let row_w = (w - indent).max(0.0);
        let label_y = y + (ROW_H_PX - font) * 0.5;

        // Visibility eye — read-only indicator (Text1 visible / disabled hidden).
        let eye_rect = Rect::new(row_x, y, ROW_H_PX, ROW_H_PX);
        let eye_color = resolve(
            if layer.visible {
                ColorToken::Text1
            } else {
                ColorToken::TextDisabled
            },
            theme,
        );
        let eye_icon = if layer.visible {
            IconId::Eye
        } else {
            IconId::EyeClosed
        };
        paint_icon(ctx.scene, eye_icon, eye_rect, eye_color, StrokeToken::Default.px());

        // Thumb placeholder — no per-layer pixels in the panel yet (the
        // real thumbnail arrives with the tool↔LayerStack integration).
        let thumb_rect = Rect::new(
            eye_rect.x + ROW_H_PX + cell_gap,
            y + (ROW_H_PX - thumb_w) * 0.5,
            thumb_w,
            thumb_w,
        );
        fill_rounded_rect(
            ctx.scene,
            thumb_rect,
            Radius::Sm.px(),
            resolve(ColorToken::BgElev, theme),
        );

        // Name (clipped to leave the meta column on the right).
        let name_x = thumb_rect.x + thumb_w + cell_gap;
        let name_w = (row_x + row_w - LAYER_META_W - name_x).max(0.0);
        paint_text(
            ctx.text_system,
            ctx.scene,
            &layer.name,
            name_x,
            label_y,
            font,
            name_w,
            resolve(ColorToken::Text1, theme),
        );

        // Meta: "Blend  NN%" (dim, right column).
        let meta = format!("{}  {:.0}%", layer.blend_mode.name(), layer.opacity * 100.0);
        paint_text(
            ctx.text_system,
            ctx.scene,
            &meta,
            row_x + row_w - LAYER_META_W,
            label_y,
            font,
            LAYER_META_W,
            resolve(ColorToken::Text2, theme),
        );

        y += ROW_H_PX + row_gap;

        // Recurse into non-collapsed groups.
        if let LayerKind::Group(g) = &layer.kind
            && !g.collapsed
        {
            y = paint_layer_subtree(ctx, theme, stack, &g.children, depth + 1, x, w, y);
        }
    }
    y
}

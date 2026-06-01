//! Painter layers panel paint — W3.T3.4 UI-plumbing (interactive rows).
//!
//! Render canon (mirror do `ph2d-panel-painter-sidebar` paint):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_layers`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Layers" + a
//!   dock-toggle button ("Brush", mode C) + close (X) button
//! - Drag handle + 2 resize handles (Inspector slot shared canon)
//! - One interactive row per layer (top→bottom, recursing groups):
//!   * line 1: visibility eye toggle · thumb · name (click = select) · blend
//!     mode chip (click = cycle to the next mode)
//!   * line 2: opacity slider + numeric chip (`0..1` storage, % display)
//! - "+ Layer" button at the foot of the body
//!
//! Per-row widget ids are derived via
//! [`ph2d_editor_core::ids::painter_layer_widget_id`] and **registered in the
//! `WidgetStore` here in `paint`** (the panel owns `store_mut`), so the normal
//! dispatch path emits `WidgetEvent`s — no hierarchy-style companion-bit
//! dispatcher allowlist needed. `event.rs` classifies those into a tool-
//! agnostic [`PanelEvent`] forwarded over `EditorAction::ToolPanelEvent`; the
//! shell calls `PainterTool::handle_panel_event`.

use crate::PainterLayersPanel;
use crate::state::{self, PainterLayersPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, SliderOrientation, SliderState, TextInputState, paint_button,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{Layer, LayerId, LayerKind, LayerStack, MAX_BLEND_MODES};

// Per-row layout metrics. Component-specific layout (not global Spacing steps),
// hence the single-line LITERAL-PX-OK justifications.
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children
const BLEND_CHIP_W: f32 = 84.0; // LITERAL-PX-OK: blend-mode chip column width
const OPACITY_CHIP_W: f32 = 52.0; // LITERAL-PX-OK: opacity numeric-chip column width
const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
const ADD_BTN_W: f32 = 96.0; // LITERAL-PX-OK: "+ Layer" button width

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
    // slot persistence, mirror do sidebar).
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

    // Dock-toggle button ("Brush") — mode C: swaps the shared dock slot back
    // to the brush sidebar. Sits just left of the close (X) button.
    paint_dock_toggle(ctx, rect, theme);

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
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;

    match state::current_layers() {
        Some(stack) if !stack.is_empty() => {
            let active = stack.active();
            y = paint_layer_subtree(
                ctx,
                theme,
                &stack,
                stack.root(),
                active,
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
                content_w,
                resolve(ColorToken::Text2, theme),
            );
            y += font + Spacing::Md.px();
        }
    }

    // "+ Layer" button — append a transparent raster on top.
    y += Spacing::Xs.px();
    paint_add_button(ctx, rect.x + PANEL_HEAD_PAD, content_w, y, theme);
    y += ROW_H_PX;

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

/// Header dock-toggle ("Brush") — swaps the shared dock slot to the brush
/// sidebar. Placed left of the close button (`PANEL_HEADER_CLOSE_RESERVE`).
fn paint_dock_toggle(ctx: &mut PaintCtx, rect: Rect, theme: ph2d_tokens::Theme) {
    let close = panel_close_button_rect(rect);
    let btn_rect = Rect::new(
        close.x - Spacing::Sm.px() - TOGGLE_BTN_W,
        close.y,
        TOGGLE_BTN_W,
        close.h,
    );
    let st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_LAYERS_TOGGLE_DOCK)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, "Brush").state(st);
    paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, btn_rect);
}

/// "+ Layer" footer button.
fn paint_add_button(ctx: &mut PaintCtx, x: f32, w: f32, y: f32, theme: ph2d_tokens::Theme) {
    let btn_w = ADD_BTN_W.min(w);
    let btn_rect = Rect::new(x, y, btn_w, ROW_H_PX);
    let st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_LAYERS_ADD)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(core_ids::PAINTER_LAYERS_ADD, "+ Layer").state(st);
    paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LAYERS_ADD, btn_rect);
}

/// Paint `ids` (top→bottom) as interactive rows, recursing into non-collapsed
/// groups (indented). Returns the `y` advanced past the painted rows.
#[allow(clippy::too_many_arguments)]
fn paint_layer_subtree(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    stack: &LayerStack,
    ids: &[LayerId],
    active: Option<LayerId>,
    depth: usize,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let indent = depth as f32 * LAYER_INDENT_STEP;
    for &id in ids {
        let Some(layer) = stack.get(id) else { continue };
        let row_x = x + indent;
        let row_w = (w - indent).max(0.0);
        y = paint_layer_row(ctx, theme, id, layer, active == Some(id), row_x, row_w, y);

        if let LayerKind::Group(g) = &layer.kind
            && !g.collapsed
        {
            y = paint_layer_subtree(ctx, theme, stack, &g.children, active, depth + 1, x, w, y);
        }
    }
    y
}

/// Paint one interactive layer row (eye · thumb · name(select) · blend chip,
/// then an opacity slider underneath) and register its per-row widgets. The
/// active row gets a subtle highlight background. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: LayerId,
    layer: &Layer,
    is_active: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let cell_gap = Spacing::Sm.px();
    let thumb_w = Spacing::Xl2.px();
    let row_gap = Spacing::Xs.px();
    let label_y = y + (ROW_H_PX - font) * 0.5;

    // Active-row highlight spanning both lines.
    if is_active {
        let hl = Rect::new(x, y, w, ROW_H_PX * 2.0 + row_gap);
        fill_rounded_rect(ctx.scene, hl, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
    }

    // ── Visibility eye toggle (Button) ──────────────────────────────────
    let eye_rect = Rect::new(x, y, ROW_H_PX, ROW_H_PX);
    let eye_id = painter_layer_widget_id(id.0, PainterLayerWidget::Visibility);
    register_button(ctx.host.store_mut(), eye_id);
    let eye_color = resolve(
        if layer.visible { ColorToken::Text1 } else { ColorToken::TextDisabled },
        theme,
    );
    let eye_icon = if layer.visible { IconId::Eye } else { IconId::EyeClosed };
    paint_icon(ctx.scene, eye_icon, eye_rect, eye_color, StrokeToken::Default.px());
    ctx.host.hit_index_mut().register(eye_id, eye_rect);

    // ── Thumb placeholder ───────────────────────────────────────────────
    let thumb_rect = Rect::new(
        eye_rect.x + ROW_H_PX + cell_gap,
        y + (ROW_H_PX - thumb_w) * 0.5,
        thumb_w,
        thumb_w,
    );
    fill_rounded_rect(ctx.scene, thumb_rect, Radius::Sm.px(), resolve(ColorToken::BgElev, theme));

    // ── Name (click anywhere thumb→blend selects the layer) ─────────────
    let blend_x = x + w - BLEND_CHIP_W;
    let name_x = thumb_rect.x + thumb_w + cell_gap;
    let name_w = (blend_x - cell_gap - name_x).max(0.0);
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
    let row_id = painter_layer_widget_id(id.0, PainterLayerWidget::Row);
    register_button(ctx.host.store_mut(), row_id);
    let select_rect = Rect::new(thumb_rect.x, y, (blend_x - cell_gap - thumb_rect.x).max(0.0), ROW_H_PX);
    ctx.host.hit_index_mut().register(row_id, select_rect);

    // ── Blend-mode chip (click cycles to the next mode) ─────────────────
    let blend_rect = Rect::new(blend_x, y, BLEND_CHIP_W, ROW_H_PX);
    let blend_id = painter_layer_widget_id(id.0, PainterLayerWidget::Blend);
    register_button(ctx.host.store_mut(), blend_id);
    let blend_st = ctx
        .host
        .store()
        .button_state(blend_id)
        .unwrap_or(ButtonState::Normal);
    let blend_btn = Button::new(blend_id, layer.blend_mode.name()).state(blend_st);
    paint_button(&blend_btn, blend_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(blend_id, blend_rect);

    // ── Opacity slider + numeric chip (line 2) ──────────────────────────
    let op_y = y + ROW_H_PX + row_gap;
    let op_slider = painter_layer_widget_id(id.0, PainterLayerWidget::Opacity);
    let op_chip = painter_layer_widget_id(id.0, PainterLayerWidget::OpacityChip);
    let pct = (layer.opacity * 100.0).round(); // LITERAL-PX-OK: opacity fraction→percent for the chip readout, not a design value
    register_opacity(ctx.host.store_mut(), op_slider, op_chip, layer.opacity, pct);
    let op_display = format!("{pct:.0}%");
    let op_rect = Rect::new(x, op_y, w, ROW_H_PX);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let op_h = paint_slider_with_chip_layout_adaptive(
        op_rect,
        "",
        layer.opacity,
        pct as f64,
        Some(&op_display),
        op_slider,
        op_chip,
        0.0,
        OPACITY_CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    op_y + op_h + Spacing::Sm.px()
}

/// `register_if_absent` a per-row Button slot (dispatch needs the store entry
/// to emit `Click`; paint draws the visuals).
fn register_button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register_if_absent(id, InteractiveState::Button { state: ButtonState::Normal });
}

/// `register_if_absent` the per-row opacity slider + chip, seeded from the
/// snapshot, and (re)link them with the `0..1 → 0..100` affine projection
/// (idempotent; canon DIRETRIZ §5.2). Storage is `0..1` on both.
fn register_opacity(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    value: f32,
    pct: f32,
) {
    store.register_if_absent(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register_if_absent(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: pct as f64,
            buffer: format!("{pct:.0}"),
            caret: 0,
            last_committed: pct as f64,
            selection_anchor: None,
        },
    );
    // 0..1 slider storage ↔ 0..100 % chip (integer percent).
    store.link_slider_number_mapped_integer(slider, chip, 100.0, 0.0); // LITERAL-PX-OK: 0..1→0..100% affine scale, not a design value
}

/// Compute the next blend-mode wire discriminant (wraps), for the cycle chip.
pub(crate) fn next_blend_mode(current: u8) -> u8 {
    (current + 1) % MAX_BLEND_MODES
}

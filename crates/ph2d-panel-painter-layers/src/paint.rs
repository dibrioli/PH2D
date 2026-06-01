//! Painter layers panel paint — W3.T3.4 UI-plumbing (interactive rows).
//!
//! Render canon (mirror do `ph2d-panel-painter-sidebar` paint):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_layers`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Layers" + a
//!   dock-toggle button ("Brush", mode C) + close (X) button
//! - One interactive row per layer (top→bottom, recursing groups):
//!   * line 1: visibility eye toggle · name (click = select) · blend-mode
//!     dropdown chip (opens a popover list — canon `Dropdown` widget)
//!   * line 2: opacity slider + numeric chip (`0..1` storage, % display)
//!   * the active layer's row gets an accent outline
//! - "+ Layer" button at the foot of the body
//! - the single open blend dropdown's popover is painted last (on top)
//!
//! Per-row widget ids are derived via
//! [`ph2d_editor_core::ids::painter_layer_widget_id`] and **registered in the
//! `WidgetStore` here in `paint`** (the panel owns `store_mut`), so the normal
//! dispatch path emits `WidgetEvent`s — no hierarchy-style companion-bit
//! dispatcher allowlist needed. `event.rs` classifies those into a tool-
//! agnostic [`PanelEvent`] forwarded over `EditorAction::ToolPanelEvent`.

use crate::PainterLayersPanel;
use crate::state::{self, PainterLayersPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{
    self as core_ids, PainterLayerWidget, painter_layer_blend_option_id, painter_layer_widget_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, Dropdown, DropdownOption, DropdownState, SliderOrientation, SliderState,
    TextInputState, paint_button, paint_dropdown_chip, paint_dropdown_popover_in_viewport,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BlendMode, Layer, LayerId, LayerKind, LayerStack, MAX_BLEND_MODES};

// Per-row layout metrics. Component-specific layout (not global Spacing steps),
// hence the single-line LITERAL-PX-OK justifications.
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children
const BLEND_CHIP_W: f32 = 92.0; // LITERAL-PX-OK: blend-mode dropdown chip column width
const OPACITY_CHIP_W: f32 = 52.0; // LITERAL-PX-OK: opacity numeric-chip column width
const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
const ADD_BTN_W: f32 = 96.0; // LITERAL-PX-OK: "+ Layer" button width
const PCT_SCALE: f32 = 100.0; // LITERAL-PX-OK: opacity fraction→percent scale, not a design value

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
    state::set_pending_blend_dd(None);

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_LAYERS_PANEL, rect);

    // Chrome: dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared canon — Inspector right-dock).
    {
        let drag_rect = panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    let title_size = paint_panel_title(
        rect,
        "Layers",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Dock-toggle ("Brush") + close (X).
    paint_dock_toggle(ctx, rect, theme);
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

    y += Spacing::Xs.px();
    paint_add_button(ctx, rect.x + PANEL_HEAD_PAD, content_w, y, theme);
    y += ROW_H_PX;

    let content_h = (y - body_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LAYERS_CLOSE, panel_close_button_rect(rect));

    // Deferred: the single open blend dropdown's popover, on top of everything.
    if let Some((layer_u64, chip_rect, cur_mode)) = state::take_pending_blend_dd() {
        paint_blend_popover(ctx, theme, layer_u64, chip_rect, cur_mode);
    }
}

/// Header dock-toggle ("Brush") — swaps the shared dock slot to the brush
/// sidebar. Placed left of the close button.
fn paint_dock_toggle(ctx: &mut PaintCtx, rect: Rect, theme: ph2d_tokens::Theme) {
    let close = panel_close_button_rect(rect);
    let btn_rect = Rect::new(close.x - Spacing::Sm.px() - TOGGLE_BTN_W, close.y, TOGGLE_BTN_W, close.h);
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
    let btn_rect = Rect::new(x, y, ADD_BTN_W.min(w), ROW_H_PX);
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

/// Paint one interactive layer row (eye · name(select) · blend dropdown, then
/// an opacity slider underneath) and register its per-row widgets. The active
/// row gets an accent outline. Returns the next `y`.
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
    let row_gap = Spacing::Xs.px();
    let label_y = y + (ROW_H_PX - font) * 0.5;
    let row_total_h = ROW_H_PX * 2.0 + row_gap;

    // Active-row accent outline (spans both lines), slightly outset into the
    // panel padding so it frames the row without clipping the slider.
    if is_active {
        let pad = Spacing::Xs.px();
        let hl = Rect::new(x - pad, y - pad, w + pad * 2.0, row_total_h + pad * 2.0);
        stroke_rounded_rect(ctx.scene, hl, Radius::Sm.px(), StrokeToken::Default.px(), resolve(ColorToken::Accent, theme));
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

    // ── Name (right next to the eye; click anywhere up to the blend chip
    // selects the layer) ────────────────────────────────────────────────
    let blend_x = x + w - BLEND_CHIP_W;
    let name_x = eye_rect.x + ROW_H_PX + cell_gap;
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
    let select_rect = Rect::new(name_x, y, (blend_x - cell_gap - name_x).max(0.0), ROW_H_PX);
    ctx.host.hit_index_mut().register(row_id, select_rect);

    // ── Blend-mode dropdown chip (opens a popover list) ─────────────────
    let blend_rect = Rect::new(blend_x, y, BLEND_CHIP_W, ROW_H_PX);
    let cur_mode = layer.blend_mode.to_u8();
    paint_blend_chip(ctx, theme, id.0, cur_mode, blend_rect);

    // ── Opacity slider + numeric chip (line 2) ──────────────────────────
    let op_y = y + ROW_H_PX + row_gap;
    let op_slider = painter_layer_widget_id(id.0, PainterLayerWidget::Opacity);
    let op_chip = painter_layer_widget_id(id.0, PainterLayerWidget::OpacityChip);
    let pct = (layer.opacity * PCT_SCALE).round();
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

/// All 22 blend modes as `Dropdown` options for the layer with runtime id
/// `layer_u64` (value = wire discriminant, label = display name).
fn blend_options(layer_u64: u64) -> Vec<DropdownOption<u8>> {
    (0..MAX_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                painter_layer_blend_option_id(layer_u64, m),
                m,
                BlendMode::from_u8(m).name(),
            )
        })
        .collect()
}

/// Paint the closed blend dropdown chip for a row, register it, and (if the
/// store says it's open AND no other dropdown already claimed the popover)
/// stash it for the deferred popover pass.
fn paint_blend_chip(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, layer_u64: u64, cur_mode: u8, rect: Rect) {
    let id = painter_layer_widget_id(layer_u64, PainterLayerWidget::Blend);
    let store = ctx.host.store_mut();
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_mode as usize),
        },
    );
    let (dd_state, store_open) = match store.get(id) {
        Some(InteractiveState::Dropdown { state, open, .. }) => (*state, *open),
        _ => (DropdownState::Normal, false),
    };
    // One popover at a time: only the first open dropdown (top→bottom) wins.
    let open = store_open && state::pending_blend_dd().is_none();
    if store_open && !open
        && let Some(InteractiveState::Dropdown { open: o, .. }) = store.get_mut(id)
    {
        *o = false;
    }
    let dd = Dropdown::new(id, "", blend_options(layer_u64))
        .selected(cur_mode)
        .open(open)
        .state(dd_state);
    paint_dropdown_chip(&dd, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    if open {
        state::set_pending_blend_dd(Some((layer_u64, rect, cur_mode)));
    }
}

/// Deferred paint of the single open blend dropdown popover (on top of the
/// rows, clamped to the viewport so it stays on-screen). Registers each option
/// as a Button + its hit rect so option clicks dispatch.
fn paint_blend_popover(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, layer_u64: u64, chip_rect: Rect, cur_mode: u8) {
    let dd = Dropdown::new(
        painter_layer_widget_id(layer_u64, PainterLayerWidget::Blend),
        "",
        blend_options(layer_u64),
    )
    .selected(cur_mode)
    .open(true);
    let viewport = ctx.viewport;
    let panel = dd.popover_rect_clamped(chip_rect, viewport);
    paint_dropdown_popover_in_viewport(&dd, chip_rect, Some(viewport), ctx.scene, ctx.text_system, theme);
    // Register option buttons (mutable store) then their hit rects (mutable
    // hit_index) in separate borrows — `store_and_hit_index_mut` hands back an
    // immutable store, which can't `register_if_absent`.
    {
        let store = ctx.host.store_mut();
        for opt in dd.options.iter() {
            register_button(store, opt.id);
        }
    }
    let hit_index = ctx.host.hit_index_mut();
    for (i, opt) in dd.options.iter().enumerate() {
        hit_index.register(opt.id, dd.option_rect_in(chip_rect, panel, i));
    }
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
    store.link_slider_number_mapped_integer(slider, chip, PCT_SCALE, 0.0); // LITERAL-PX-OK: 0..1→0..100% affine, not a design value
}

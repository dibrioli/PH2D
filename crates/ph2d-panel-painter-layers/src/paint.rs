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
use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget, painter_layer_widget_id};
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
    Button, ButtonState, SliderOrientation, SliderState, TextInputState, paint_button,
    paint_scrollbar, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{Layer, LayerId, LayerKind, LayerStack};

// Per-row layout metrics. Component-specific layout (not global Spacing steps),
// hence the single-line LITERAL-PX-OK justifications.
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children
const BLEND_CHIP_W: f32 = 92.0; // LITERAL-PX-OK: blend-mode dropdown chip column width
const OPACITY_CHIP_W: f32 = 72.0; // LITERAL-PX-OK: opacity numeric-chip column width (= canon number_input MIN_W_PX)
const REORDER_W: f32 = 16.0; // LITERAL-PX-OK: far-right ↑↓ reorder button column width
const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
const ADD_BTN_W: f32 = 96.0; // LITERAL-PX-OK: "+ Layer" button width
const APPLY_BTN_W: f32 = 80.0; // LITERAL-PX-OK: "Apply" CTA button width
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
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
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

    // Dock-toggle (Brush) + close (X).
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

    let scroll_y = ctx
        .host
        .store()
        .panel_scroll(core_ids::PAINTER_LAYERS_PANEL)
        .max(0.0);
    // Body content scrolls: the paint origin is offset up by the scroll
    // position; content_h is measured from it (scroll-independent). Extra top
    // padding so the active-row accent outline is not clipped at the body top
    // and the first row sits clear of the Layers title.
    let body_paint_top = body_top + Spacing::Md.px() - scroll_y;
    let mut y = body_paint_top;
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
    // Footer: "+ Layer" (left) + the "Apply" CTA (right) — Apply commits the
    // composite into the sprite (the only discoverable commit; Cmd/Ctrl+Enter
    // is the hidden shortcut).
    let footer_x = rect.x + PANEL_HEAD_PAD;
    paint_add_button(ctx, footer_x, ADD_BTN_W, y, theme);
    let apply_rect = Rect::new(footer_x + content_w - APPLY_BTN_W, y, APPLY_BTN_W, ROW_H_PX);
    paint_apply_button(ctx, apply_rect, theme);
    y += ROW_H_PX;

    let content_h = (y - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Visual scrollbar (self-gates when the content fits). Wheel/trackpad
    // scrolling already works via the generic `dispatch_wheel` once the bounds
    // below are published. NOTE: thumb-DRAG needs a foundational
    // `scrollbar_panel_for_id` entry + a `widget::*_SCROLLBAR_ID` const
    // (Coord follow-up); the wheel covers the interaction meanwhile.
    paint_scrollbar(body_rect, scroll_y, content_h, body_h, false, ctx.scene, theme);

    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    // Re-register ALL header chrome AFTER the body rows. `HitIndex::hit` is
    // last-registered-wins, and scrolled-up rows have hit rects with `y <
    // body_top` that overlap the header — without this they shadow the
    // dock-toggle / drag / resize handles (audit HIGH). The body paint is
    // clipped so those rows are invisible, but their hits are not clipped.
    {
        let close = panel_close_button_rect(rect);
        let toggle = Rect::new(
            close.x - Spacing::Sm.px() - TOGGLE_BTN_W,
            close.y,
            TOGGLE_BTN_W,
            close.h,
        );
        let hit = ctx.host.hit_index_mut();
        hit.register(
            core_ids::INSP_DRAG_HANDLE,
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE),
        );
        hit.register(core_ids::INSP_RESIZE_HANDLE, panel_resize_handle_rect(rect));
        hit.register(core_ids::INSP_RESIZE_HANDLE_BL, panel_resize_handle_rect_bl(rect));
        hit.register(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, toggle);
        hit.register(core_ids::PAINTER_LAYERS_CLOSE, close);
    }

    // Publish scroll bounds so `dispatch_wheel` scrolls this panel + clamp the
    // offset to the new content (so deleting/collapsing rows snaps back).
    {
        let store = ctx.host.store_mut();
        store.set_panel_content_h(core_ids::PAINTER_LAYERS_PANEL, content_h);
        store.set_panel_visible_h(core_ids::PAINTER_LAYERS_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(core_ids::PAINTER_LAYERS_PANEL) > max_scroll {
            store.set_panel_scroll(core_ids::PAINTER_LAYERS_PANEL, max_scroll);
        }
    }

    // Deferred: the single open blend dropdown popover, on top of everything.
    if let Some((layer_u64, chip_rect, cur_mode)) = state::take_pending_blend_dd() {
        crate::blend::paint_blend_popover(ctx, theme, layer_u64, chip_rect, cur_mode);
    }
}

/// Header dock-toggle ("Brush") — swaps the shared dock slot to the brush
/// sidebar. Placed left of the close button.
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

/// "Apply" footer CTA — commits the live layer composite into the sprite
/// (routes to `PainterTool::request_commit`). Accent-filled for prominence.
fn paint_apply_button(ctx: &mut PaintCtx, rect: Rect, theme: ph2d_tokens::Theme) {
    let st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_APPLY)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(core_ids::PAINTER_APPLY, "Apply")
        .accent()
        .state(st);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_APPLY, rect);
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
    let last = ids.len().saturating_sub(1);
    for (i, &id) in ids.iter().enumerate() {
        let Some(layer) = stack.get(id) else { continue };
        let row_x = x + indent;
        let row_w = (w - indent).max(0.0);
        // The bottom-most root layer IS the sprite image (the base): painting
        // it edits the image directly and it has nothing beneath to blend with,
        // so it gets no blend-mode dropdown (Photoshop "Background" semantics).
        let is_base = depth == 0 && i == last;
        // Reorder ↑↓ availability. The base stays locked at the bottom of the
        // root stack, so root layers can never move INTO its slot (can_down
        // stops one short of `last`); within a group, free reorder.
        let can_up = !is_base && i > 0;
        let can_down = if depth == 0 {
            !is_base && i + 1 < last
        } else {
            i < last
        };
        y = paint_layer_row(
            ctx,
            theme,
            id,
            layer,
            active == Some(id),
            is_base,
            can_up,
            can_down,
            row_x,
            row_w,
            y,
        );

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
    is_base: bool,
    can_up: bool,
    can_down: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let cell_gap = Spacing::Sm.px();
    let row_gap = Spacing::Xs.px();
    let label_y = y + (ROW_H_PX - font) * 0.5;
    let row_total_h = ROW_H_PX * 2.0 + row_gap;
    let op_y = y + ROW_H_PX + row_gap;
    // The far-right column is reserved (on every row, for alignment) for the
    // ↑↓ reorder buttons; the rest of the row content stops at `content_right`.
    let reorder_x = x + w - REORDER_W;
    let content_right = reorder_x - cell_gap;

    // Active-row accent outline (spans both lines), slightly outset into the
    // panel padding so it frames the row without clipping the slider.
    if is_active {
        let pad = Spacing::Xs.px();
        let hl = Rect::new(x - pad, y - pad, w + pad * 2.0, row_total_h + pad * 2.0);
        stroke_rounded_rect(
            ctx.scene,
            hl,
            Radius::Sm.px(),
            StrokeToken::Default.px(),
            resolve(ColorToken::Accent, theme),
        );
    }

    // ── Visibility eye toggle (Button) ──────────────────────────────────
    let eye_rect = Rect::new(x, y, ROW_H_PX, ROW_H_PX);
    let eye_id = painter_layer_widget_id(id.0, PainterLayerWidget::Visibility);
    register_button(ctx.host.store_mut(), eye_id);
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
    paint_icon(
        ctx.scene,
        eye_icon,
        eye_rect,
        eye_color,
        StrokeToken::Default.px(),
    );
    ctx.host.hit_index_mut().register(eye_id, eye_rect);

    // ── Name (right next to the eye; click anywhere up to the blend chip
    // selects the layer). The base layer has no blend chip, so its name +
    // hit-rect run to the row's right edge. ─────────────────────────────
    let name_x = eye_rect.x + ROW_H_PX + cell_gap;
    let blend_x = content_right - BLEND_CHIP_W;
    let name_right = if is_base { content_right } else { blend_x - cell_gap };
    let name_w = (name_right - name_x).max(0.0);
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
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(name_x, y, name_w, ROW_H_PX));

    // ── Blend-mode dropdown chip (opens a popover list) — skipped for the
    // base layer (it IS the image; nothing below to blend with). ────────
    if !is_base {
        let blend_rect = Rect::new(blend_x, y, BLEND_CHIP_W, ROW_H_PX);
        crate::blend::paint_blend_chip(ctx, theme, id.0, layer.blend_mode.to_u8(), blend_rect);
    }

    // ── Reorder ↑↓ buttons (far-right column). Disabled buttons paint dim
    // and are not hit-registered, so they no-op. The base has neither. ──
    let up_id = painter_layer_widget_id(id.0, PainterLayerWidget::MoveUp);
    let down_id = painter_layer_widget_id(id.0, PainterLayerWidget::MoveDown);
    paint_reorder_btn(
        ctx,
        theme,
        up_id,
        Rect::new(reorder_x, y, REORDER_W, ROW_H_PX),
        can_up,
        IconId::ChevronUp,
    );
    paint_reorder_btn(
        ctx,
        theme,
        down_id,
        Rect::new(reorder_x, op_y, REORDER_W, ROW_H_PX),
        can_down,
        IconId::ChevronDown,
    );

    // ── Opacity slider + numeric chip (line 2) ──────────────────────────
    let op_slider = painter_layer_widget_id(id.0, PainterLayerWidget::Opacity);
    let op_chip = painter_layer_widget_id(id.0, PainterLayerWidget::OpacityChip);
    let pct = (layer.opacity * PCT_SCALE).round();
    register_opacity(ctx.host.store_mut(), op_slider, op_chip, layer.opacity, pct);
    let op_display = format!("{pct:.0}%");
    let op_rect = Rect::new(x, op_y, content_right - x, ROW_H_PX);
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

/// Paint one ↑/↓ reorder button. When `enabled`, it draws at full contrast and
/// is registered + hit-mapped (so the click dispatches). When disabled (list
/// edge / base layer) it draws dim and is NOT registered, so it no-ops.
fn paint_reorder_btn(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    enabled: bool,
    icon: IconId,
) {
    let color = resolve(
        if enabled {
            ColorToken::Text2
        } else {
            ColorToken::TextDisabled
        },
        theme,
    );
    paint_icon(ctx.scene, icon, rect, color, StrokeToken::Default.px());
    if enabled {
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, rect);
    }
}

/// `register_if_absent` a per-row Button slot (dispatch needs the store entry
/// to emit `Click`; paint draws the visuals).
pub(crate) fn register_button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
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

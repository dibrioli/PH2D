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
use ph2d_editor_core::interaction::{HierarchyDragState, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, PAINTER_LAYERS_SCROLLBAR_ID, Slider, SliderOrientation, SliderState,
    paint_button, paint_scrollbar, paint_slider, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{Layer, LayerId, LayerKind, LayerStack};

// Per-row layout metrics. Component-specific layout (not global Spacing steps),
// hence the single-line LITERAL-PX-OK justifications.
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children
const BLEND_CHIP_W: f32 = 92.0; // LITERAL-PX-OK: blend-mode dropdown chip column width
const OPACITY_PCT_W: f32 = 44.0; // LITERAL-PX-OK: plain "NN%" readout column right of the bare opacity slider
const REORDER_W: f32 = 16.0; // LITERAL-PX-OK: far-right ↑↓ reorder button column width
const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
const HEADER_ICON_W: f32 = 28.0; // LITERAL-PX-OK: action icon-button square
const TOOLBAR_H: f32 = 36.0; // LITERAL-PX-OK: action toolbar strip height (icon + pad)
const APPLY_BTN_W: f32 = 80.0; // LITERAL-PX-OK: "Apply" CTA button width
const PCT_SCALE: f32 = 100.0; // LITERAL-PX-OK: opacity fraction→percent scale, not a design value
const DROP_BAR_H: f32 = 2.0; // LITERAL-PX-OK: W3.T3.8 drag drop-indicator bar thickness
const GHOST_PAD: f32 = 7.0; // LITERAL-PX-OK: floating drag-ghost pill horizontal text padding
const GHOST_CHAR_ADV: f32 = 0.62; // LITERAL-PX-OK: rough glyph-advance fraction of font px (ghost pill sizing, cosmetic)

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

    // Action toolbar strip (New / Group / Duplicate / Delete) sits one row BELOW
    // the header, between it and the scrollable list — fixed, not scrolled.
    let header_bottom = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let toolbar_rect = Rect::new(rect.x, header_bottom, rect.w, TOOLBAR_H);

    // Body region (clipped), starting below the toolbar.
    let body_top = header_bottom + TOOLBAR_H;
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

    // Row-id set published to the dispatch so it knows which NodeIds are
    // draggable layer rows (Coord drag foundation `1c3411d`). Filled in the
    // layer branch, pushed in the scroll-bounds block below.
    let mut painter_row_ids: std::collections::BTreeSet<ph2d_a11y::NodeId> =
        std::collections::BTreeSet::new();
    // W3.T3.8 drag overlay: the live reparent gesture (id being dragged + the
    // latest cursor). Copied out (state is `Copy`) so no store borrow lingers
    // into the `store_mut()` calls below. `None` unless a drag passed the
    // distance threshold (`active`).
    let dragging = ctx.host.store().painter_layer_drag().filter(|d| d.active);
    // The floating "lifted layer" pill (name + cursor) painted last, unclipped.
    let mut ghost: Option<(String, f32, f32)> = None;
    match state::current_layers() {
        Some(stack) if !stack.is_empty() => {
            painter_row_ids = stack
                .all_ids()
                .map(|l| painter_layer_widget_id(l.0, PainterLayerWidget::Row))
                .collect();
            let active = stack.active();
            // Full-row drop bands collected during the walk (id, full-row rect,
            // is_group) so the indicator below mirrors `find_painter_layer_drop`
            // exactly (same rows, same 30/40/30) — WYSIWYG drop.
            let mut drag_rows: Vec<(ph2d_a11y::NodeId, Rect, bool)> = Vec::new();
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
                dragging,
                &mut drag_rows,
            );
            if let Some(d) = dragging {
                // Live drop indicator, on top of the rows (still inside the body
                // clip so it can't bleed over the header/footer).
                paint_drop_indicator(
                    ctx,
                    theme,
                    &drag_rows,
                    d.cursor_y,
                    d.dragged,
                    rect.x + PANEL_HEAD_PAD,
                    content_w,
                );
                // Decode the dragged NodeId → its layer name for the ghost pill.
                ghost = stack
                    .all_ids()
                    .find(|lid| {
                        painter_layer_widget_id(lid.0, PainterLayerWidget::Row) == d.dragged
                    })
                    .and_then(|lid| stack.get(lid))
                    .map(|l| (l.name.clone(), d.cursor_x, d.cursor_y));
            }
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
    // Footer: the "Apply" CTA (right) — commits the composite into the sprite
    // (the only discoverable commit; Cmd/Ctrl+Enter is the hidden shortcut).
    // "+ Layer" moved to the header icon cluster (New layer / Group / Duplicate /
    // Delete) — `paint_header_actions`, registered post-body (last-wins).
    let footer_x = rect.x + PANEL_HEAD_PAD;
    let apply_rect = Rect::new(footer_x + content_w - APPLY_BTN_W, y, APPLY_BTN_W, ROW_H_PX);
    paint_apply_button(ctx, apply_rect, theme);
    y += ROW_H_PX;

    let content_h = (y - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Visual scrollbar (self-gates when the content fits). Wheel/trackpad
    // scrolling works via the generic `dispatch_wheel` once the bounds below are
    // published; thumb-DRAG works via the foundational
    // `scrollbar_panel_for_id → PAINTER_LAYERS_PANEL` mapping (Coord `d5146b7`)
    // plus the hit-rect registered in the post-body chrome block. `is_active`
    // tints the thumb Accent while dragging (mirror of the Inspector).
    let scrollbar_active = matches!(
        ctx.host.store().scrollbar_drag(),
        Some(d) if d.panel == core_ids::PAINTER_LAYERS_PANEL
    );
    paint_scrollbar(
        body_rect,
        scroll_y,
        content_h,
        body_h,
        scrollbar_active,
        ctx.scene,
        theme,
    );

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
        // Scrollbar thumb-drag: register the thumb hit-rect AFTER the rows (same
        // last-wins reason as the chrome above). Gated to when the scrollbar is
        // actually shown; the rect must match `paint_scrollbar`'s internal
        // `track_rect → thumb_rect` so the grab aligns with the painted thumb.
        if scrollbar_is_needed(content_h, body_h) {
            let track = scrollbar_track_rect(body_rect);
            let thumb = scrollbar_thumb_rect(track, scroll_y, content_h, body_h);
            hit.register(PAINTER_LAYERS_SCROLLBAR_ID, thumb);
        }
    }

    // Action toolbar icons (New layer / Group / Duplicate / Delete) — one row
    // below the header. Painted + registered AFTER the rows (same last-wins
    // reason as the chrome above; scrolled rows must not shadow the toolbar).
    paint_action_toolbar(ctx, toolbar_rect, theme);

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
        // Tell the dispatch which NodeIds are draggable layer rows (Down on one
        // of these begins a `PainterLayerReparent` drag — Coord drag foundation).
        store.set_painter_layer_row_ids(painter_row_ids);
    }

    // Deferred: the single open blend dropdown popover, on top of everything.
    if let Some((layer_u64, chip_rect, cur_mode)) = state::take_pending_blend_dd() {
        crate::blend::paint_blend_popover(ctx, theme, layer_u64, chip_rect, cur_mode);
    }

    // W3.T3.8: the floating drag ghost is the very top layer (a drag and an open
    // blend popover are mutually exclusive, so order vs the popover is moot).
    // Painted after the body clip pop so it tracks the cursor past the list.
    if let Some((name, cx, cy)) = ghost {
        paint_drag_ghost(ctx, theme, &name, cx, cy);
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

/// Action toolbar (one row below the header): New layer · Group · Duplicate ·
/// Delete — the canonical ghost-icon button (same look as the Hierarchy header
/// "+"). Laid out left-to-right inside `toolbar_rect`; painted + registered
/// post-body so scrolled rows don't shadow them.
fn paint_action_toolbar(ctx: &mut PaintCtx, toolbar_rect: Rect, theme: ph2d_tokens::Theme) {
    let mut x = toolbar_rect.x + PANEL_HEAD_PAD;
    // Vertically center the square icon within the toolbar strip.
    let y = toolbar_rect.y + ((toolbar_rect.h - HEADER_ICON_W) * 0.5).max(0.0);
    let specs = [
        (core_ids::PAINTER_LAYERS_ADD, IconId::Add, "New layer"),
        (core_ids::PAINTER_LAYERS_GROUP, IconId::Group, "New group"),
        (core_ids::PAINTER_LAYERS_DUPLICATE, IconId::Duplicate, "Duplicate layer"),
        (core_ids::PAINTER_LAYERS_DELETE, IconId::Trash, "Delete layer"),
    ];
    for (id, icon, label) in specs {
        let btn_rect = Rect::new(x, y, HEADER_ICON_W, HEADER_ICON_W);
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).icon_only(icon).state(st);
        paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, btn_rect);
        x += HEADER_ICON_W + Spacing::Xs.px();
    }
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
    dragging: Option<HierarchyDragState>,
    drag_rows: &mut Vec<(ph2d_a11y::NodeId, Rect, bool)>,
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
            dragging,
            drag_rows,
        );

        if let LayerKind::Group(g) = &layer.kind
            && !g.collapsed
        {
            y = paint_layer_subtree(
                ctx,
                theme,
                stack,
                &g.children,
                active,
                depth + 1,
                x,
                w,
                y,
                dragging,
                drag_rows,
            );
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
    dragging: Option<HierarchyDragState>,
    drag_rows: &mut Vec<(ph2d_a11y::NodeId, Rect, bool)>,
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

    // W3.T3.8 drag overlay: identify this row's draggable handle id once (reused
    // for the hit-rect below). Only while a drag is in flight do we record its
    // FULL-row band for the drop indicator — no per-frame Vec churn when idle.
    let row_id = painter_layer_widget_id(id.0, PainterLayerWidget::Row);
    let is_dragged = dragging.map(|d| d.dragged == row_id).unwrap_or(false);
    if dragging.is_some() {
        drag_rows.push((row_id, Rect::new(x, y, w, row_total_h), layer.is_group()));
    }
    // The row being dragged reads as "lifted": a soft accent wash in place while
    // the real content tracks the cursor as the floating ghost (Procreate-style).
    if is_dragged {
        let pad = Spacing::Xs.px();
        let lift = Rect::new(x - pad, y - pad, w + pad * 2.0, row_total_h + pad * 2.0);
        fill_rounded_rect(
            ctx.scene,
            lift,
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
    }

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
    register_button(ctx.host.store_mut(), row_id);
    // Hit rect spans the FULL row height (both lines), not just the name line, so
    // the whole row is a drag handle + drop target — and the drop indicator's
    // bands (computed off this same geometry) match where the reparent lands.
    // The per-cell widgets (eye registered above; blend/slider/reorder below) are
    // registered with their own rects and win by last-registration, so this wider
    // rect only claims the otherwise-empty row body for select/drag.
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(name_x, y, name_w, row_total_h));

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

    // ── Opacity: a BARE slider (no numeric chip → no per-row Vello clip,
    // which was the panel FPS sink) + a plain "NN%" readout (line 2). ───
    let op_slider = painter_layer_widget_id(id.0, PainterLayerWidget::Opacity);
    register_opacity(ctx.host.store_mut(), op_slider, layer.opacity);
    let pct = (layer.opacity * PCT_SCALE).round();
    let slider_w = (content_right - x - OPACITY_PCT_W - cell_gap).max(0.0);
    let st = ctx
        .host
        .store()
        .slider(op_slider)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let mut slider = Slider::new(op_slider, "").accent(true).state(st);
    slider.value = layer.opacity;
    paint_slider(&slider, Rect::new(x, op_y, slider_w, ROW_H_PX), ctx.scene, theme);
    ctx.host.hit_index_mut().register(op_slider, Rect::new(x, op_y, slider_w, ROW_H_PX));
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{pct:.0}%"),
        x + slider_w + cell_gap,
        op_y + (ROW_H_PX - font) * 0.5,
        font,
        OPACITY_PCT_W,
        resolve(ColorToken::Text2, theme),
    );

    op_y + ROW_H_PX + Spacing::Sm.px()
}

/// Live drop indicator for an in-progress layer drag — painted on top of the
/// rows, mirroring `find_painter_layer_drop`'s 30/40/30 band split so the user
/// sees exactly where the reparent lands (WYSIWYG drop):
///   - top 30% of a row → a bar at the row's top edge (insert before)
///   - middle 40% → an outline box AROUND a group (nest inside); over a leaf the
///     tool falls back to before-sibling, so the top bar shows there too
///   - bottom 30% → a bar at the row's bottom edge (insert after)
///   - below every row → a bar above the base (End → root bottom)
/// Skips the dragged row itself, exactly as the dispatch does.
fn paint_drop_indicator(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rows: &[(ph2d_a11y::NodeId, Rect, bool)],
    cursor_y: f32,
    dragged: ph2d_a11y::NodeId,
    x: f32,
    w: f32,
) {
    let accent = resolve(ColorToken::Accent, theme);
    let bar = |ctx: &mut PaintCtx, y: f32| {
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x, y - DROP_BAR_H * 0.5, w, DROP_BAR_H),
            Radius::Sm.px(),
            accent,
        );
    };
    for &(id, rect, is_group) in rows {
        if id == dragged {
            continue;
        }
        let top = rect.y;
        let bot = rect.y + rect.h;
        if cursor_y < top || cursor_y >= bot {
            continue;
        }
        let inside_top = top + rect.h * 0.3;
        let inside_bot = top + rect.h * 0.7;
        if cursor_y < inside_top {
            bar(ctx, top);
        } else if cursor_y < inside_bot {
            if is_group {
                stroke_rounded_rect(ctx.scene, rect, Radius::Sm.px(), StrokeToken::Thick.px(), accent);
            } else {
                bar(ctx, top);
            }
        } else {
            bar(ctx, bot);
        }
        return;
    }
    // Below every visible row → End (root bottom, just above the base sprite,
    // which is the bottom-most row). Draw the bar at the base's top edge.
    if let Some(&(_, base, _)) = rows.last() {
        bar(ctx, base.y);
    }
}

/// Floating pill that tracks the cursor during a layer drag — the real-time
/// "this layer is moving" cue (Procreate-style). Painted unclipped (after the
/// body clip is popped) so it follows the cursor even past the list bounds.
fn paint_drag_ghost(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    name: &str,
    cursor_x: f32,
    cursor_y: f32,
) {
    let font = TypeToken::Base.px();
    let text_w = (name.chars().count() as f32) * font * GHOST_CHAR_ADV;
    let pill_w = text_w + GHOST_PAD * 2.0;
    let pill_h = ROW_H_PX;
    // Sit just below-right of the cursor so the pointer doesn't cover the label.
    let px = cursor_x + Spacing::Sm.px();
    let py = cursor_y - pill_h * 0.5;
    let pill = Rect::new(px, py, pill_w, pill_h);
    fill_rounded_rect(ctx.scene, pill, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(
        ctx.scene,
        pill,
        Radius::Sm.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Accent, theme),
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        name,
        px + GHOST_PAD,
        py + (pill_h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );
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

/// `register_if_absent` the per-row opacity slider (bare, `0..1` storage). The
/// dispatch maps a drag to a fresh value from the registered hit rect; the
/// panel forwards the resulting `ValueChanged` as `SetValue` to the tool.
fn register_opacity(store: &mut WidgetStore, slider: ph2d_a11y::NodeId, value: f32) {
    store.register_if_absent(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation: SliderOrientation::Horizontal,
        },
    );
}

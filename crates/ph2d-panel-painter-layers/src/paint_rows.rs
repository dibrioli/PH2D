//! Painter layers panel — per-row rendering + W3.T3.8 drag overlay.
//!
//! Split out of `paint.rs` (panel-LOC cap): the orchestrator lives in
//! [`crate::paint`]; this sibling holds the recursive layer-row painters and
//! the live drag-reparent overlay (drop indicator + floating ghost). All entry
//! points are `pub(crate)` so the orchestrator can call them.

use crate::paint::register_button;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::{HierarchyDragState, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Slider, SliderOrientation, SliderState, paint_button,
    paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{Layer, LayerId, LayerKind, LayerStack};
use std::collections::BTreeSet;

// Per-row layout metrics. Component-specific layout (not global Spacing steps),
// hence the single-line LITERAL-PX-OK justifications.
const LAYER_INDENT_STEP: f32 = 14.0; // LITERAL-PX-OK: per-nesting-level indent for group children
const BLEND_CHIP_W: f32 = 92.0; // LITERAL-PX-OK: blend-mode dropdown chip column width
const OPACITY_PCT_W: f32 = 44.0; // LITERAL-PX-OK: plain "NN%" readout column right of the bare opacity slider
const REORDER_W: f32 = 16.0; // LITERAL-PX-OK: far-right reorder button column width
const PCT_SCALE: f32 = 100.0; // LITERAL-PX-OK: opacity fraction-to-percent scale, not a design value
const DROP_BAR_H: f32 = 2.0; // LITERAL-PX-OK: W3.T3.8 drag drop-indicator bar thickness
const GHOST_PAD: f32 = 7.0; // LITERAL-PX-OK: floating drag-ghost pill horizontal text padding
const DROP_BAND_TOP: f32 = 0.3; // LITERAL-PX-OK: top 30% = insert-before band (mirror of find_painter_layer_drop)
const DROP_BAND_BOT: f32 = 0.7; // LITERAL-PX-OK: bottom 30% = insert-after band (mirror of find_painter_layer_drop)
const MASK_INV_W: f32 = 34.0; // LITERAL-PX-OK: mask-row "Inv" toggle button width
const MASK_APPLY_W: f32 = 48.0; // LITERAL-PX-OK: mask-row "Apply" button width

/// Paint `ids` (top-to-bottom) as interactive rows, recursing into
/// non-collapsed groups (indented). Returns the `y` advanced past the rows.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_layer_subtree(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    stack: &LayerStack,
    ids: &[LayerId],
    active: Option<LayerId>,
    selected: &BTreeSet<LayerId>,
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
        // so it gets no blend-mode dropdown (Photoshop Background semantics).
        let is_base = depth == 0 && i == last;
        // Reorder availability. The base stays locked at the bottom of the root
        // stack, so root layers can never move INTO its slot (can_down stops one
        // short of `last`); within a group, free reorder.
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
            selected.contains(&id),
            is_base,
            can_up,
            can_down,
            row_x,
            row_w,
            y,
            dragging,
            drag_rows,
        );

        // T3.5: a layer's attached mask renders as an indented, selectable
        // sub-row directly under it (one nesting level deeper).
        if let Some(mask_id) = layer.mask {
            let inverted = matches!(
                stack.get(mask_id).map(|l| &l.kind),
                Some(LayerKind::Mask(mk)) if mk.inverted
            );
            let mask_indent = (depth + 1) as f32 * LAYER_INDENT_STEP;
            y = paint_mask_row(
                ctx,
                theme,
                mask_id,
                inverted,
                active == Some(mask_id),
                x + mask_indent,
                (w - mask_indent).max(0.0),
                y,
            );
        }

        if let LayerKind::Group(g) = &layer.kind
            && !g.collapsed
        {
            y = paint_layer_subtree(
                ctx,
                theme,
                stack,
                &g.children,
                active,
                selected,
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

/// Paint one interactive layer row (eye, name-select, blend dropdown, then an
/// opacity slider underneath) and register its per-row widgets. The active row
/// gets an accent outline. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: LayerId,
    layer: &Layer,
    is_active: bool,
    is_selected: bool,
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
    // reorder buttons; the rest of the row content stops at `content_right`.
    let reorder_x = x + w - REORDER_W;
    let content_right = reorder_x - cell_gap;

    // W3.T3.8 drag overlay: identify this row draggable handle id once (reused
    // for the hit-rect below). Only while a drag is in flight do we record its
    // FULL-row band for the drop indicator: no per-frame Vec churn when idle.
    let row_id = painter_layer_widget_id(id.0, PainterLayerWidget::Row);
    let is_dragged = dragging.map(|d| d.dragged == row_id).unwrap_or(false);
    if dragging.is_some() {
        drag_rows.push((row_id, Rect::new(x, y, w, row_total_h), layer.is_group()));
    }
    // The row being dragged reads as lifted: a soft accent wash in place while
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

    // Multi-select wash: a selected row that is NOT the active one gets a soft
    // accent fill behind its content; the active row keeps the strong outline
    // below. Skipped while dragged (the lift wash already fills the row).
    if is_selected && !is_active && !is_dragged {
        let pad = Spacing::Xs.px();
        let sel = Rect::new(x - pad, y - pad, w + pad * 2.0, row_total_h + pad * 2.0);
        fill_rounded_rect(
            ctx.scene,
            sel,
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
    // selects the layer). The base layer has no blend chip, so its name plus
    // hit-rect run to the row right edge. ───────────────────────────────
    let name_x = eye_rect.x + ROW_H_PX + cell_gap;
    let blend_x = content_right - BLEND_CHIP_W;
    let name_right = if is_base {
        content_right
    } else {
        blend_x - cell_gap
    };
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
    // the whole row is a drag handle plus drop target, and the drop indicator
    // bands (computed off this same geometry) match where the reparent lands.
    // The per-cell widgets (eye registered above; blend/slider/reorder below) are
    // registered with their own rects and win by last-registration, so this wider
    // rect only claims the otherwise-empty row body for select/drag.
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(name_x, y, name_w, row_total_h));

    // ── Blend-mode dropdown chip (opens a popover list): skipped for the
    // base layer (it IS the image; nothing below to blend with). ─────────
    if !is_base {
        let blend_rect = Rect::new(blend_x, y, BLEND_CHIP_W, ROW_H_PX);
        crate::blend::paint_blend_chip(ctx, theme, id.0, layer.blend_mode.to_u8(), blend_rect);
    }

    // ── Reorder buttons (far-right column). Disabled buttons paint dim and
    // are not hit-registered, so they no-op. The base has neither. ───────
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

    // ── Opacity: a BARE slider (no numeric chip, hence no per-row Vello clip,
    // which was the panel FPS sink) plus a plain "NN%" readout (line 2). ──
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
    paint_slider(
        &slider,
        Rect::new(x, op_y, slider_w, ROW_H_PX),
        ctx.scene,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(op_slider, Rect::new(x, op_y, slider_w, ROW_H_PX));
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

/// Paint a layer's attached mask (§2.7) as an indented, selectable sub-row: a
/// "Mask" label (+ "· Inverted") with the active accent outline, plus the
/// Invert toggle + Apply (destructive bake) affordances on the right. Selecting
/// the row routes through the normal Row path → the tool makes the mask the edit
/// target (grayscale paint). Single line. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn paint_mask_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    mask_id: LayerId,
    inverted: bool,
    is_active: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let cell_gap = Spacing::Xs.px();
    if is_active {
        stroke_rounded_rect(
            ctx.scene,
            Rect::new(x, y, w, ROW_H_PX),
            Radius::Sm.px(),
            StrokeToken::Default.px(),
            resolve(ColorToken::Accent, theme),
        );
    }

    // Right-aligned affordances: [ Inv ] [ Apply ]. The label runs up to them.
    let apply_rect = Rect::new(x + w - MASK_APPLY_W, y, MASK_APPLY_W, ROW_H_PX);
    let inv_rect = Rect::new(apply_rect.x - cell_gap - MASK_INV_W, y, MASK_INV_W, ROW_H_PX);

    let label = if inverted { "Mask · Inverted" } else { "Mask" };
    let label_x = x + Spacing::Sm.px();
    let label_y = y + (ROW_H_PX - font) * 0.5;
    let label_w = (inv_rect.x - cell_gap - label_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        label_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text2, theme),
    );

    // Register the row-select hit rect FIRST, spanning the FULL row. The two
    // buttons register AFTER and win their own cells (HitIndex is last-
    // registration-wins), so the whole row selects EXCEPT the button cells —
    // robust even when deep nesting shrinks the row (no dead/negative select
    // band, and the buttons' left edges are not shadowed by the row rect).
    let row_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::Row);
    register_button(ctx.host.store_mut(), row_id);
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(x, y, w, ROW_H_PX));

    // Invert toggle — accent-filled when on (mirror of the modifier toolbar).
    let inv_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::MaskInvert);
    register_button(ctx.host.store_mut(), inv_id);
    let inv_st = ctx
        .host
        .store()
        .button_state(inv_id)
        .unwrap_or(ButtonState::Normal);
    let mut inv_btn = Button::new(inv_id, "Inv").state(inv_st);
    if inverted {
        inv_btn.kind = ButtonKind::Accent;
    }
    paint_button(&inv_btn, inv_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(inv_id, inv_rect);

    // Apply — destructive bake into the parent alpha, then remove the mask.
    let apply_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::MaskApply);
    register_button(ctx.host.store_mut(), apply_id);
    let apply_st = ctx
        .host
        .store()
        .button_state(apply_id)
        .unwrap_or(ButtonState::Normal);
    let apply_btn = Button::new(apply_id, "Apply").state(apply_st);
    paint_button(&apply_btn, apply_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(apply_id, apply_rect);

    y + ROW_H_PX + Spacing::Xs.px()
}

/// Live drop indicator for an in-progress layer drag, painted on top of the
/// rows, mirroring `find_painter_layer_drop`'s 30/40/30 band split so the user
/// sees exactly where the reparent lands (WYSIWYG drop). Skips the dragged row
/// itself, exactly as the dispatch does. Bands:
///
/// - top 30% of a row: a bar at the row top edge (insert before)
/// - middle 40%: an outline box around a group (nest inside); over a leaf the
///   tool falls back to before-sibling, so the top bar shows there too
/// - bottom 30%: a bar at the row bottom edge (insert after)
/// - below every row: a bar above the base (End to root bottom)
pub(crate) fn paint_drop_indicator(
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
        let inside_top = top + rect.h * DROP_BAND_TOP;
        let inside_bot = top + rect.h * DROP_BAND_BOT;
        if cursor_y < inside_top {
            bar(ctx, top);
        } else if cursor_y < inside_bot {
            if is_group {
                stroke_rounded_rect(
                    ctx.scene,
                    rect,
                    Radius::Sm.px(),
                    StrokeToken::Thick.px(),
                    accent,
                );
            } else {
                bar(ctx, top);
            }
        } else {
            bar(ctx, bot);
        }
        return;
    }
    // Below every visible row: End (root bottom, just above the base sprite,
    // which is the bottom-most row). Draw the bar at the base top edge.
    if let Some(&(_, base, _)) = rows.last() {
        bar(ctx, base.y);
    }
}

/// Floating pill that tracks the cursor during a layer drag: the real-time
/// "this layer is moving" cue (Procreate-style). Painted unclipped (after the
/// body clip is popped) so it follows the cursor even past the list bounds.
pub(crate) fn paint_drag_ghost(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    name: &str,
    cursor_x: f32,
    cursor_y: f32,
) {
    let font = TypeToken::Base.px();
    // True pixel advance (Inter is proportional, so char count is not width).
    let text_w = ctx.text_system.prefix_width(name, font);
    let pill_w = text_w + GHOST_PAD * 2.0;
    let pill_h = ROW_H_PX;
    // Sit just below-right of the cursor so the pointer does not cover the label.
    let px = cursor_x + Spacing::Sm.px();
    let py = cursor_y - pill_h * 0.5;
    let pill = Rect::new(px, py, pill_w, pill_h);
    fill_rounded_rect(
        ctx.scene,
        pill,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
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

/// Paint one reorder button. When `enabled`, it draws at full contrast and is
/// registered plus hit-mapped (so the click dispatches). When disabled (list
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

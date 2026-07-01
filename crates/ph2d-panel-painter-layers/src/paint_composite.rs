//! The **Composite Brush** card (below Strength, above Accumulate). A bordered box with a "Composite
//! Brush" checkbox; when on, the single Strength slider hides and the card shows the 3-layer stack —
//! Brush · Smear · Blur — as reorderable rows modelled on the Layers panel: `N Name` label, a **bare**
//! Strength slider + plain readout (no numeric chip — the Layers-opacity pattern, which never stacks in
//! a narrow panel), and dim-at-the-ends ↑/↓ reorder buttons. The position number `N` is FIXED 1/2/3;
//! the tool at each position moves with the arrows. Split from `paint_brush` for the LOC cap;
//! registration of its fixed-id widgets is [`register_composite_widgets`] (called from `crate::populate`).

use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxValue, Slider, SliderOrientation, SliderState, paint_checkbox,
    paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Fixed-width label column for `N Name` (fits "2 Smear" at the Base font).
const LABEL_W: f32 = 54.0; // LITERAL-PX-OK: layer label column ("N ToolName")
/// Plain "0.50" strength readout column, right of the bare slider.
const READOUT_W: f32 = 34.0; // LITERAL-PX-OK: strength readout column
/// Reorder ↑/↓ button column width (mirrors the Layers panel's `REORDER_W`).
const ARROW_W: f32 = 16.0; // LITERAL-PX-OK: reorder button column (matches paint_rows)

/// The name shown in each layer row for a composite op wire discriminant (`0` Brush · `1` Smear · `2` Blur).
fn op_name(op: u8) -> &'static str {
    match op {
        1 => "Smear",
        2 => "Blur",
        _ => "Brush",
    }
}

/// Paint the Composite Brush card and return the next `y`. Draws its own bordered background so it reads
/// as a distinct card. The Strength-slider hide is the caller's job (it owns the row order); this only
/// renders the card.
pub(crate) fn paint_composite_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Xs.px();
    let checked = brush.composite_enabled;
    // Single-line rows (bare slider never stacks), so the height is exact: padding + the checkbox row +
    // (when on) a gap and the 3 layer rows (each ROW_H + gap).
    let layers_h = if checked {
        gap + 3.0 * (ROW_H_PX + gap)
    } else {
        0.0
    };
    let card_h = pad + ROW_H_PX + layers_h + pad;
    let card = Rect::new(x, y, content_w, card_h);
    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, card, radius, resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        ctx.scene,
        card,
        radius,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );

    let ix = x + pad;
    let iw = content_w - 2.0 * pad;
    let mut iy = y + pad;

    // The enable checkbox (forwards a plain Click → the tool's `toggle_composite`).
    let cb = Checkbox::new(core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE, "Composite Brush").value(
        if checked {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        },
    );
    let cb_rect = Rect::new(ix, iy, iw, ROW_H_PX);
    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE, cb_rect);
    iy += ROW_H_PX;

    if checked {
        iy += gap;
        for pos in 0..3usize {
            iy = paint_layer_row(ctx, theme, ix, iw, iy, pos, brush);
        }
    }
    y + card_h + Spacing::Sm.px()
}

/// One composite layer row: `N Name` label · bare Strength slider · "0.50" readout · ↑/↓ reorder. The
/// number `N` = `pos + 1` is the FIXED position; the tool name follows the reordered stack. The top
/// row's ↑ and the bottom row's ↓ paint dim and are inert (list edges), keeping every row aligned.
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let down_x = x + row_w - ARROW_W;
    let up_x = down_x - gap - ARROW_W;
    let readout_x = up_x - gap - READOUT_W;
    let slider_x = x + LABEL_W + gap;
    let slider_w = (readout_x - gap - slider_x).max(24.0);

    // "N Name" label.
    let label = format!("{} {}", pos + 1, op_name(brush.composite_ops[pos]));
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text1, theme),
    );

    // Bare Strength slider (value from the snapshot; state from the store) + plain readout.
    let sid = core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos];
    let val = brush.composite_strength[pos].clamp(0.0, 1.0);
    let st = ctx
        .host
        .store()
        .slider(sid)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let mut slider = Slider::new(sid, "").accent(true).state(st);
    slider.value = val;
    let slider_rect = Rect::new(slider_x, y, slider_w, ROW_H_PX);
    paint_slider(&slider, slider_rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(sid, slider_rect);
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{val:.2}"),
        readout_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        READOUT_W,
        resolve(ColorToken::Text2, theme),
    );

    // Reorder arrows (dim + inert at the list edges) — shared with the Layers rows.
    crate::paint_rows::paint_reorder_btn(
        ctx,
        theme,
        core_ids::PAINTER_BRUSH_COMPOSITE_UP[pos],
        Rect::new(up_x, y, ARROW_W, ROW_H_PX),
        pos > 0,
        IconId::ChevronUp,
    );
    crate::paint_rows::paint_reorder_btn(
        ctx,
        theme,
        core_ids::PAINTER_BRUSH_COMPOSITE_DOWN[pos],
        Rect::new(down_x, y, ARROW_W, ROW_H_PX),
        pos < 2,
        IconId::ChevronDown,
    );

    y + ROW_H_PX + gap
}

/// Register the Composite-card fixed-id widgets in the `WidgetStore` (from `crate::populate`): the 3
/// per-position bare Strength sliders + the enable checkbox and the 6 reorder buttons — all forwarded
/// over the existing Click / ValueChanged channels to the tool's `route_composite_event`.
pub(crate) fn register_composite_widgets(store: &mut WidgetStore) {
    for sid in core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH {
        store.register(
            sid,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
    for id in core_ids::PAINTER_BRUSH_COMPOSITE_BUTTONS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

//! The **Composite Brush** card (below Strength, above Accumulate). A bordered box with a "Composite
//! Brush" checkbox; when on, the single Strength slider hides and the card shows the 3-layer stack —
//! Brush · Smear · Blur — as reorderable rows (like the Layers panel). Each row is a Strength
//! slider-with-chip labelled `N ToolName` (the position number `N` is FIXED 1/2/3; the tool moves via
//! the up/down buttons), so the stroke runs all three ops with per-layer Strength. Split from
//! `paint_brush` for the LOC cap. Registration of its fixed-id widgets lives in
//! [`register_composite_widgets`] (called from `crate::populate`).

use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxValue, IconButtonStyle, IconGlyph, SliderOrientation,
    SliderState, TextInputState, paint_checkbox, paint_icon_button, paint_slider_with_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken};
use ph2d_tool_painter::BrushSettings;

/// The name shown in each layer row for a composite op wire discriminant (`0` Brush · `1` Smear · `2` Blur).
fn op_name(op: u8) -> &'static str {
    match op {
        1 => "Smear",
        2 => "Blur",
        _ => "Brush",
    }
}

/// Paint the Composite Brush card and return the next `y`. Draws its own bordered background so it reads
/// as a distinct card (like the Jitter/Randomize grouping). The Strength-slider hide is the caller's job
/// (it owns the row order); this only renders the card.
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
    // Card height: padding + the checkbox row + (when on) a gap and the 3 layer rows (each ROW_H + gap).
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
            iy += gap;
        }
    }
    y + card_h + Spacing::Sm.px()
}

/// One composite layer row: a Strength slider-with-chip labelled `N ToolName`, then up/down reorder
/// buttons on the right. The number `N` = `pos + 1` is the FIXED position; the tool name follows the
/// reordered stack. The top row hides its up button and the bottom row its down button (inert at the
/// ends), keeping the slots reserved so all rows align.
fn paint_layer_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    pos: usize,
    brush: BrushSettings,
) -> f32 {
    let arrow_w = ROW_H_PX;
    let gap = Spacing::Xxs.px();
    let ctrl_w = (row_w - 2.0 * (arrow_w + gap)).max(40.0);
    let slider_rect = Rect::new(x, y, ctrl_w, ROW_H_PX);
    let up_rect = Rect::new(x + ctrl_w + gap, y, arrow_w, ROW_H_PX);
    let down_rect = Rect::new(up_rect.x + arrow_w + gap, y, arrow_w, ROW_H_PX);

    let label = format!("{} {}", pos + 1, op_name(brush.composite_ops[pos]));
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_slider_with_chip(
            slider_rect,
            &label,
            brush.composite_strength[pos],
            core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos],
            core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos],
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
    // Reorder arrows (top row has no "up", bottom row no "down").
    if pos > 0 {
        paint_arrow(
            ctx,
            theme,
            up_rect,
            IconId::ChevronUp,
            core_ids::PAINTER_BRUSH_COMPOSITE_UP[pos],
        );
    }
    if pos < 2 {
        paint_arrow(
            ctx,
            theme,
            down_rect,
            IconId::ChevronDown,
            core_ids::PAINTER_BRUSH_COMPOSITE_DOWN[pos],
        );
    }
    y + ROW_H_PX
}

/// Paint + hit-register one reorder arrow button (frameless glyph, state-tinted).
fn paint_arrow(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    icon: IconId,
    id: ph2d_a11y::NodeId,
) {
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        rect,
        IconGlyph::Builtin(icon),
        IconButtonStyle::Plain,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// Register the Composite-card fixed-id widgets in the `WidgetStore` (called from `crate::populate`):
/// the 3 per-position Strength sliders + their linked/ranged numeric chips (canonical slider-with-chip),
/// plus the enable checkbox and the 6 reorder buttons — all forwarded over the existing Click / SetValue
/// channels to the tool's `route_composite_event`.
pub(crate) fn register_composite_widgets(store: &mut WidgetStore) {
    // The chip stepper / drag increment on the `0..1` track (a behaviour value, not a design token).
    const STEP: f64 = 0.01; // LITERAL-PX-OK: chip 0..1 track step (non-design behaviour value)
    for i in 0..3 {
        let slider = core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH[i];
        let chip = core_ids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[i];
        store.register(
            slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider, chip);
        store.set_number_range(chip, 0.0, 1.0, STEP);
    }
    let enable = core_ids::PAINTER_BRUSH_COMPOSITE_ENABLE;
    for id in std::iter::once(enable)
        .chain(core_ids::PAINTER_BRUSH_COMPOSITE_UP)
        .chain(core_ids::PAINTER_BRUSH_COMPOSITE_DOWN)
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

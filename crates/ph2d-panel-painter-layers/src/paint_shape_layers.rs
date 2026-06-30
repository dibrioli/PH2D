//! The Shape **Per-Layer Color** UI (multi-layer Shape): a mode toggle below the Texture dropdown, plus
//! — when on — one row per captured layer: a "Layer N Color" checkbox + a right cluster of `[B blend]
//! [colour box] [opacity]`. Texture Color is the DEFAULT (the layer paints its own captured colours); the
//! checkbox opts into a CUSTOM colour, revealing the colour box. The "B" chip picks the layer's blend
//! mode; the opacity box scales its tip contribution (brush-only). The cluster wraps to a second row when
//! the panel is too narrow. Shown only for a multi-layer (`> 1`) Shape; while on, the caller hides the
//! Shape Color ramp section. Split from `paint_shape` for the workspace LOC cap.

use crate::paint::register_button;
use crate::paint_brush_top::paint_checkbox_row;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    Button, Checkbox, CheckboxValue, ColorSwatch, DropdownOption, DropdownState, SwatchSize,
    SwatchState, paint_button, paint_checkbox, paint_color_swatch,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BlendMode, BrushSettings, MAX_BLEND_MODES};

const SWATCH_W: f32 = ROW_H_PX; // LITERAL-PX-OK: per-layer colour box — SQUARE (Enio 2026-06-29)
const BLEND_LABEL_W: f32 = 38.0; // LITERAL-PX-OK: "Blend" text label column (second row)
const OPACITY_W: f32 = 48.0; // LITERAL-PX-OK: per-layer opacity number box width
const SHAPE_BLEND_POPOVER_W: f32 = 132.0; // LITERAL-PX-OK: open blend list width (long mode names fit one line)
const OPACITY_PCT: f32 = 100.0; // LITERAL-PX-OK: opacity shown/edited as 0..100 percent (not a design value)

/// sRGB 8-bit normalize for the picker round-trip.
fn enc(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 // LITERAL-PX-OK: sRGB 8-bit normalize
}

/// Paint the "Use Document Layers" button — captures the active document's visible raster layers as a
/// multi-layer Shape (each becomes a colourable layer). Always shown in the Shape section. Returns `y`.
pub(crate) fn paint_use_layers_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let id = core_ids::PAINTER_SHAPE_USE_LAYERS;
    let btn = Button::new(id, "Use Document Layers");
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

/// Paint the **Per-Layer Color** toggle + (when on) the per-layer "Layer N Color" checkbox + swatch
/// rows. A no-op for a single-layer / no Shape. Returns the next `y`.
pub(crate) fn paint_shape_per_layer_color(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    let count = brush.shape_layer_count as usize;
    if count <= 1 {
        return y; // single-layer image (or none) ⇒ no per-layer-colour UI
    }
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SHAPE_PER_LAYER_COLOR,
        "Per-Layer Color",
        brush.shape_per_layer_color,
    );
    if !brush.shape_per_layer_color {
        return y;
    }
    let n = count.min(brush.shape_layer_color_on.len());
    let gap = Spacing::Xs.px();
    for i in 0..n {
        let iu = i as u8;
        let check_id = core_ids::painter_shape_layer_color_check_id(iu);
        // Texture Color is the DEFAULT (`color_on` off ⇒ the layer paints its own captured colours); the
        // "Layer N Color" checkbox turns on a CUSTOM colour, which reveals the colour box.
        let color_on = brush.shape_layer_color_on[i];

        // ── Row 1: the "Layer N Color" checkbox (fills the left) + the square colour box (far right),
        //    shown only when the layer has a custom colour. ──
        let box_rect = color_on.then(|| Rect::new(x + content_w - SWATCH_W, y, SWATCH_W, ROW_H_PX));
        let cb_w = if color_on {
            (content_w - SWATCH_W - gap).max(0.0)
        } else {
            content_w
        };
        let cb = Checkbox::new(check_id, format!("Layer {} Color", i + 1)).value(if color_on {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
        let cb_rect = Rect::new(x, y, cb_w, ROW_H_PX);
        paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
        register_button(ctx.host.store_mut(), check_id);
        ctx.host.hit_index_mut().register(check_id, cb_rect);
        if let Some(sr) = box_rect {
            let sw_id = core_ids::painter_shape_layer_color_swatch_id(iu);
            let c = brush.shape_layer_color[i];
            let open = ctx.host.store().picker_target() == Some(sw_id);
            paint_color_swatch(
                &ColorSwatch {
                    id: sw_id,
                    label: String::new(),
                    rgba: [enc(c[0]), enc(c[1]), enc(c[2]), 255],
                    state: if open {
                        SwatchState::Focused
                    } else {
                        SwatchState::Normal
                    },
                    size: SwatchSize::Sm,
                },
                sr,
                ctx.scene,
                theme,
            );
            register_button(ctx.host.store_mut(), sw_id);
            ctx.host.hit_index_mut().register(sw_id, sr);
            layer_color_readback(ctx, sw_id, iu, c);
        }
        y += ROW_H_PX;

        // ── Row 2: "Blend" label + the named blend dropdown (like the Layers panel) + the opacity box.
        //    The BOTTOM layer (index 0, the base) has NO blend — Photoshop-Background semantics, mirroring
        //    the Layers panel — so only its opacity box shows. ──
        let op_rect = Rect::new(x + content_w - OPACITY_W, y, OPACITY_W, ROW_H_PX);
        if i > 0 {
            let font = TypeToken::Sm.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                "Blend",
                x,
                y + (ROW_H_PX - font) * 0.5,
                font,
                BLEND_LABEL_W,
                resolve(ColorToken::Text2, theme),
            );
            let chip_x = x + BLEND_LABEL_W + gap;
            let chip_w = (op_rect.x - gap - chip_x).max(0.0);
            paint_shape_blend_chip(
                ctx,
                theme,
                iu,
                brush.shape_layer_blend[i],
                Rect::new(chip_x, y, chip_w, ROW_H_PX),
            );
        }
        crate::number_field::chip(
            ctx,
            theme,
            op_rect,
            core_ids::painter_shape_layer_opacity_id(iu),
            brush.shape_layer_opacity[i] * OPACITY_PCT,
            0.0,
            OPACITY_PCT,
            1.0,
            0,
        );
        y += ROW_H_PX + Spacing::Sm.px();
    }
    y
}

/// All 22 blend modes as dropdown options for Shape layer `i` (value = wire discriminant, label = name).
fn shape_blend_options(i: u8) -> Vec<DropdownOption<u8>> {
    (0..MAX_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                core_ids::painter_shape_layer_blend_option_id(i, m),
                m,
                BlendMode::from_u8(m).name(),
            )
        })
        .collect()
}

/// Paint the per-layer **blend** ("B") dropdown chip (registered as an `InteractiveState::Dropdown` for the
/// generic open/close dispatch). One popover open at a time; when open it is stashed for the deferred pass.
fn paint_shape_blend_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    i: u8,
    cur_mode: u8,
    rect: Rect,
) {
    let id = core_ids::painter_shape_layer_blend_id(i);
    let store = ctx.host.store_mut();
    store.register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_mode as usize),
        },
    );
    let store_open = matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    // One popover at a time: only the first open Shape-blend chip (top→bottom) wins.
    let open = store_open && crate::state::pending_shape_blend_dd().is_none();
    if store_open
        && !open
        && let Some(InteractiveState::Dropdown { open: o, .. }) = store.get_mut(id)
    {
        *o = false;
    }
    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::Bg1, theme));
    let border = if open {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );
    // The current blend-mode NAME (left, clipped to the text column) + chevron (right) — the same shape as
    // the Layers-panel blend chip, so the per-layer blend reads the way Layers does.
    let font = TypeToken::Sm.px();
    let pad = Spacing::Xs.px();
    let chevron = Spacing::Md.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let text_w = (chevron_rect.x - Spacing::Xxs.px() - (rect.x + pad)).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        BlendMode::from_u8(cur_mode).name(),
        rect.x + pad,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    ctx.host.hit_index_mut().register(id, rect);
    if open {
        crate::state::set_pending_shape_blend_dd(Some((i, rect, cur_mode)));
    }
}

/// Deferred paint of the open Shape-layer blend popover (on top of the rows). Drained by the Brush-section
/// popover pass via [`crate::state::take_pending_shape_blend_dd`].
pub(crate) fn paint_shape_blend_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    i: u8,
    chip_rect: Rect,
    cur_mode: u8,
) {
    // Ensure the popover is at least wide enough for the longest mode names, anchored at the chip's LEFT
    // edge so it extends rightward into the panel (the chip sits after the "Blend" label, left of centre);
    // the shared renderer clamps it to the viewport.
    let pop_w = SHAPE_BLEND_POPOVER_W.max(chip_rect.w);
    let pop_chip = Rect::new(chip_rect.x, chip_rect.y, pop_w, chip_rect.h);
    crate::paint_brush::paint_dropdown_popover(
        ctx,
        theme,
        core_ids::painter_shape_layer_blend_id(i),
        shape_blend_options(i),
        pop_chip,
        cur_mode,
    );
}

/// When the picker targets layer `i`'s swatch, forward its live colour as `"i,r,g,b"` (u8) once it
/// differs — the tool decodes it to `set_brush_shape_layer_color(i, rgb)`.
fn layer_color_readback(ctx: &mut PaintCtx, swatch_id: NodeId, i: u8, cur: [f32; 3]) {
    if ctx.host.store().picker_target() != Some(swatch_id) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(swatch_id) else {
        return;
    };
    if [enc(cur[0]), enc(cur[1]), enc(cur[2])] == [picked[0], picked[1], picked[2]] {
        return;
    }
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            swatch_id,
            format!("{i},{},{},{}", picked[0], picked[1], picked[2]),
        )));
}

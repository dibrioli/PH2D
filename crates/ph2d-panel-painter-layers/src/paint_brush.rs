//! The Painter dock's **Brush-properties** view (header toggle → "Brush"): the
//! active brush's Size slider, blend-mode chip, and a colour swatch that opens
//! the shared Blender colour picker (`INSP_BLENDER_PICKER`, the same rich picker
//! the Inspector uses — only one is ever open, so they share the slot).
//!
//! The brush is tool-global, so these are FIXED-id widgets (registered in
//! [`crate::populate`]). The panel reads the published [`BrushSettings`] snapshot
//! to position them and forwards edits over the frozen `PanelEvent` channel. The
//! colour round-trip is a per-frame read-back: when the floating picker targets
//! our swatch, its live value (mirrored by the hero loop into
//! `widget_color(target)`) is forwarded to the tool.

use crate::paint::register_button;
use crate::state;
use ph2d_editor_core::IconId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{self as core_ids, painter_brush_blend_option_id};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::panel_chrome::PANEL_HEAD_PAD;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Dropdown, DropdownOption, DropdownState, Slider, SliderState,
    paint_button, paint_dropdown_popover_in_viewport, paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BrushBlend, BrushSettings, MAX_BRUSH_BLEND_MODES};

const LABEL_W: f32 = 60.0; // LITERAL-PX-OK: brush row label column ("Hardness"/"Strength")
const READOUT_W: f32 = 30.0; // LITERAL-PX-OK: param readout column

/// Brush used before the first snapshot publish (Painter just activated). In
/// practice the bridge publishes every frame the panel is visible, so this is
/// only a defensive default.
const FALLBACK_BRUSH: BrushSettings = BrushSettings {
    size_px: 25.0,
    size_norm: 0.217,
    hardness: 0.0,
    flow: 1.0,
    strength: 1.0,
    color: [0.0, 0.0, 0.0],
    blend: 0,
    eraser: false,
};

/// Paint the Brush-properties body below `header_bottom` (the Painter dock in
/// Brush mode). Terminal for the panel — owns its own popover pass.
pub(crate) fn paint_brush_mode(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    header_bottom: f32,
) {
    let x = rect.x + PANEL_HEAD_PAD;
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;
    let brush = state::current_brush().unwrap_or(FALLBACK_BRUSH);
    // If the shared picker is editing our swatch, forward its live colour.
    brush_color_readback(ctx, brush);

    let gap = Spacing::Sm.px();
    let mut y = header_bottom + Spacing::Md.px();
    // Hardness/Flow/Strength read as percent; Size reads raw px.
    let pct = |v: f32| format!("{:.0}", v * 100.0);

    // ── Slider rows: Size (px) + Hardness / Flow / Strength (%) ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Size",
        id: core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        value: brush.size_norm,
        readout: &format!("{:.0}", brush.size_px),
    });
    for (lbl, id, value) in [
        ("Hardness", core_ids::PAINTER_BRUSH_HARDNESS_SLIDER, brush.hardness),
        ("Flow", core_ids::PAINTER_BRUSH_FLOW_SLIDER, brush.flow),
        ("Strength", core_ids::PAINTER_BRUSH_STRENGTH_SLIDER, brush.strength),
    ] {
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: lbl,
            id,
            value,
            readout: &pct(value),
        });
    }

    // ── Blend: label + dropdown chip ──
    label(ctx, theme, "Blend", x, y, TypeToken::Sm.px());
    let chip_w = (content_w - LABEL_W - gap).max(0.0);
    paint_brush_blend_chip(
        ctx,
        theme,
        brush.blend,
        Rect::new(x + LABEL_W + gap, y, chip_w, ROW_H_PX),
    );
    y += ROW_H_PX + Spacing::Sm.px();

    // ── Eraser: full-width mode toggle (Accent while erasing) ──
    y = paint_eraser_toggle(ctx, theme, x, content_w, y, brush.eraser);

    // ── Colour: label + swatch (click opens the shared Blender picker) ──
    paint_color_swatch_row(ctx, theme, x, content_w, y, brush);

    // Deferred: the brush blend popover, on top of the body.
    if let Some((chip_rect, cur_mode)) = state::take_pending_brush_blend_dd() {
        paint_brush_blend_popover(ctx, theme, chip_rect, cur_mode);
    }
}

/// Args for [`paint_param_row`] (grouped to dodge the too-many-arguments lint).
struct ParamRow<'a, 'b> {
    ctx: &'a mut PaintCtx<'b>,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &'a str,
    id: ph2d_a11y::NodeId,
    value: f32,
    readout: &'a str,
}

/// Paint one "label · slider · readout" brush param row. Returns the next `y`.
fn paint_param_row(r: ParamRow) -> f32 {
    let ParamRow { ctx, theme, x, content_w, y, label: label_txt, id, value, readout } = r;
    let font = TypeToken::Sm.px();
    let gap = Spacing::Sm.px();
    label(ctx, theme, label_txt, x, y, font);
    let slider_w = (content_w - LABEL_W - gap - READOUT_W - gap).max(0.0);
    paint_brush_slider(
        ctx,
        theme,
        id,
        Rect::new(x + LABEL_W + gap, y, slider_w, ROW_H_PX),
        value,
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        readout,
        x + LABEL_W + gap + slider_w + gap,
        y + (ROW_H_PX - font) * 0.5,
        font,
        READOUT_W,
        resolve(ColorToken::Text2, theme),
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Paint the full-width Eraser mode toggle (Accent while `on`). Returns next `y`.
fn paint_eraser_toggle(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    on: bool,
) -> f32 {
    let id = core_ids::PAINTER_BRUSH_ERASER;
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    let st = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    let mut btn = Button::new(id, "Eraser").state(st);
    if on {
        btn.kind = ButtonKind::Accent;
    }
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

/// When the shared Blender picker targets the brush swatch, the hero loop mirrors
/// its live value into `widget_color(PAINTER_COLOR_THUMB)`. Forward that colour to
/// the tool (as `"r,g,b"`) when it differs from the brush's current colour — this
/// makes the picker drive the brush live.
fn brush_color_readback(ctx: &mut PaintCtx, brush: BrushSettings) {
    if ctx.host.store().picker_target() != Some(core_ids::PAINTER_COLOR_THUMB) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(core_ids::PAINTER_COLOR_THUMB) else {
        return;
    };
    let cur = encode_rgb(brush.color);
    if [picked[0], picked[1], picked[2]] == cur {
        return; // already applied — don't spam the bus
    }
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            core_ids::PAINTER_COLOR_THUMB,
            format!("{},{},{}", picked[0], picked[1], picked[2]),
        )));
}

/// Paint the colour preview swatch (a full-width bar). Registered as a button:
/// clicking it toggles the shared Blender picker (see `event.rs`). The accent
/// border shows when the picker is currently editing it.
fn paint_color_swatch_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) {
    let font = TypeToken::Sm.px();
    label(ctx, theme, "Color", x, y, font);
    let sx = x + LABEL_W + Spacing::Sm.px();
    let sw = (content_w - LABEL_W - Spacing::Sm.px()).max(0.0);
    let rect = Rect::new(sx, y, sw, ROW_H_PX);
    register_button(ctx.host.store_mut(), core_ids::PAINTER_COLOR_THUMB);

    let [r, g, b] = encode_rgb(brush.color);
    let col = ph2d_vector::Color::from_rgba8(r, g, b, 255); // LITERAL-COLOR-OK: brush colour (data)
    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, col);
    let open = ctx.host.store().picker_target() == Some(core_ids::PAINTER_COLOR_THUMB);
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
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_COLOR_THUMB, rect);
}

/// Deferred paint of the open brush blend popover (clamped to the viewport).
/// Mirror of [`crate::blend::paint_blend_popover`] but for the single fixed brush
/// chip + the 24 `BrushBlend` modes.
pub(crate) fn paint_brush_blend_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    chip_rect: Rect,
    cur_mode: u8,
) {
    let dd = Dropdown::new(core_ids::PAINTER_BRUSH_BLEND, "", brush_blend_options())
        .selected(cur_mode)
        .open(true);
    let viewport = ctx.viewport;
    let panel = dd.popover_rect_clamped(chip_rect, viewport);
    paint_dropdown_popover_in_viewport(
        &dd,
        chip_rect,
        Some(viewport),
        ctx.scene,
        ctx.text_system,
        theme,
    );
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

/// A left-aligned, vertically-centred row label in a `ROW_H_PX` cell.
fn label(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, text: &str, x: f32, y: f32, font: f32) {
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
}

/// Paint one accent brush slider showing `value` (`0..1`) and register its hit
/// rect. The store value is driven by the drag dispatch; the display tracks the
/// snapshot (mirror of the per-row opacity slider).
fn paint_brush_slider(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    value: f32,
) {
    let st = ctx
        .host
        .store()
        .slider(id)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let mut slider = Slider::new(id, "").accent(true).state(st);
    slider.value = value;
    paint_slider(&slider, rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, rect);
}

/// The brush blend chip (registered as a `Dropdown` for the generic open/close
/// dispatch); when open, stash it for the deferred popover pass.
fn paint_brush_blend_chip(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, cur_mode: u8, rect: Rect) {
    let id = core_ids::PAINTER_BRUSH_BLEND;
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_mode as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );

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

    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
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

    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        BrushBlend::from_u8(cur_mode).name(),
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    if open {
        state::set_pending_brush_blend_dd(Some((rect, cur_mode)));
    }
}

/// The 24 brush blend modes as `Dropdown` options (value = wire discriminant,
/// label = display name).
fn brush_blend_options() -> Vec<DropdownOption<u8>> {
    (0..MAX_BRUSH_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                painter_brush_blend_option_id(m),
                m,
                BrushBlend::from_u8(m).name(),
            )
        })
        .collect()
}

/// Encode a straight-RGB colour in `[0, 1]` (native space) to 8-bit for display.
fn encode_rgb(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

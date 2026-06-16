//! Brush Studio row + label helpers — the leaf painters (`*_row`, `section_header`)
//! and label maps the section painters in `sections.rs` call. Split out of `sections.rs`
//! to keep each file under the panel-file LOC cap (the section orchestrator stays there;
//! these carry no orchestration). All `pub(crate)`, used only within this crate.

use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonState, Checkbox, CheckboxState, CheckboxValue, IconButtonStyle, IconGlyph,
    SectionHeader, paint_button, paint_checkbox, paint_icon_button, paint_section_header,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

const LABEL_W: f32 = 88.0; // LITERAL-PX-OK: studio slider label column width
const CHIP_W: f32 = 56.0; // LITERAL-PX-OK: studio slider value-chip column width

#[allow(clippy::too_many_arguments)]
pub(crate) fn section_header(
    ctx: &mut PaintCtx,
    id: NodeId,
    reset_id: NodeId,
    label: &str,
    x: f32,
    w: f32,
    y: f32,
    theme: Theme,
) -> (f32, bool) {
    let header_h = ROW_H_PX;
    let collapsed = ctx.host.store().is_collapsed(id);
    let header = SectionHeader::new(id, label).collapsible(!collapsed);
    let hrect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, hrect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, hrect);
    // Per-section "reset to default" button at the header's right edge. Registered AFTER the header
    // so the HitIndex (topmost-wins, `iter().rev()`) routes a click in this sub-rect to the reset,
    // not to the header's collapse toggle.
    let btn = (header_h * 0.7).clamp(12.0, 18.0); // LITERAL-PX-OK: mirrors the chevron icon sizing
    let reset_rect = Rect::new(
        x + w - btn - Spacing::Sm.px(),
        y + (header_h - btn) * 0.5,
        btn,
        btn,
    );
    let st = ctx
        .host
        .store()
        .button_state(reset_id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        st,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(reset_id, reset_rect);
    (y + header_h + Spacing::Xs.px(), collapsed)
}

/// Percent slider row (0..1 → 0..100%, integer display).
#[allow(clippy::too_many_arguments)]
pub(crate) fn pct_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    value01: f32,
    slider_id: NodeId,
    chip_id: NodeId,
    theme: Theme,
) -> f32 {
    let display = format!("{:.0}%", value01 * 100.0); // LITERAL-PX-OK: percent display scale (x100), not a px dimension
    mapped_row(
        ctx,
        x,
        w,
        y,
        label,
        value01,
        (value01 * 100.0) as f64, // LITERAL-PX-OK: percent display scale (x100), not a px dimension
        &display,
        slider_id,
        chip_id,
        theme,
    )
}

/// Slider row with an explicit chip numeric + display string (for non-percent
/// params — `shape_count`, `grain_scale`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn mapped_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    value01: f32,
    chip_value: f64,
    display: &str,
    slider_id: NodeId,
    chip_id: NodeId,
    theme: Theme,
) -> f32 {
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let h = paint_slider_with_chip_layout_adaptive(
        rect,
        label,
        value01,
        chip_value,
        Some(display),
        slider_id,
        chip_id,
        LABEL_W,
        CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    y + h + Spacing::Sm.px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn checkbox_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    checked: bool,
    id: NodeId,
    theme: Theme,
) -> f32 {
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .checkbox(id)
        .map(|(st, _)| st)
        .unwrap_or(CheckboxState::Normal);
    let value = if checked {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let cb = Checkbox::new(id, label).state(state).value(value);
    paint_checkbox(&cb, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

/// A full-width cycling button (grain type, rendering mode). `pressed` shows the
/// active (non-default) state.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cycler_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    pressed: bool,
    id: NodeId,
    theme: Theme,
) -> f32 {
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = if pressed {
        ButtonState::Pressed
    } else {
        ctx.host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal)
    };
    let btn = Button::new(id, label).state(state);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

pub(crate) fn rendering_mode_label(mode: u8) -> &'static str {
    match mode {
        0 => "Light Glaze",
        1 => "Uniform Glaze",
        2 => "Intense Glaze",
        3 => "Heavy Glaze",
        4 => "Uniform Blend",
        5 => "Intense Blend",
        _ => "Light Glaze",
    }
}

pub(crate) fn grain_type_label(grain: u8) -> &'static str {
    match grain {
        1 => "Simplex",
        2 => "Gabor",
        3 => "Weave",
        4 => "Spray",
        _ => "Off",
    }
}

//! **The transport bar's reusable pieces** — an icon-button, a number chip, a toggle, a
//! label.
//!
//! Split out of `transport.rs` under the 600-LOC panel cap (HR-18), and a unit in its own
//! right: nothing here knows what a playhead is. They are the bar's vocabulary; `transport.rs`
//! is the sentence it makes out of them.

// The moved bodies keep the parent's imports: `transport.rs` is where this bar's
// widget vocabulary is declared, and re-listing it here is a second list to drift.
use super::*;

/// Paint one square icon-button; register its hit rect. Returns the right edge.
pub(crate) fn icon_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    glyph: IconId,
) -> f32 {
    let rect = Rect::new(x, y, BTN_W, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Compact,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + BTN_W
}

/// Paint one number chip (value mirrored from the snapshot when unfocused);
/// register its `[0, ∞)` range + hit. Returns the right edge.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    value: f64,
    step: f64,
    decimals: usize,
) -> f32 {
    let rect = Rect::new(x, y, CHIP_W, ROW_H_PX);
    {
        let store = ctx.host.store_mut();
        mirror_number(store, id, value, decimals);
        store.set_number_range(id, 0.0, f64::from(u16::MAX), step);
    }
    let (state, _v, buf, caret, anchor) = read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", value).step(step).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + CHIP_W
}

/// Paint a `label | switch` toggle in the exact Widget-Gallery form: a label
/// column then a `TypeToken::Xl3`-wide, `Density::Compact`-tall pill switch
/// (proper 2:1 track + thumb), vertically centred in the row. Register the
/// switch hit. Returns the right edge.
pub(crate) fn toggle(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    text: &str,
    on: bool,
) -> f32 {
    let sw = TypeToken::Xl3.px();
    let sh = Density::Compact.row_h_px();
    let pad = Spacing::Xs.px();
    // Outlined cell grouping [label | switch] so each toggle is demarcated from
    // its neighbours (Enio 2026-07-08).
    let cell_w = pad + TOGGLE_LABEL_W + pad + sw + pad;
    let cell = Rect::new(x, y, cell_w, ROW_H_PX);
    stroke_rounded_rect(
        ctx.scene,
        cell,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    label(ctx, theme, text, x + pad, y, TOGGLE_LABEL_W);
    let sx = x + pad + TOGGLE_LABEL_W + pad;
    let rect = Rect::new(sx, y + (ROW_H_PX - sh) * 0.5, sw, sh);
    // Mirror the snapshot's on-state into the store (when not focused) so the
    // painted switch reflects the document and the edit baseline stays correct.
    {
        let store = ctx.host.store_mut();
        if store.focus_id() != Some(id)
            && let Some(InteractiveState::Toggle { on: store_on, .. }) = store.get_mut(id)
        {
            *store_on = on;
        }
    }
    let state = ctx
        .host
        .store()
        .toggle(id)
        .map(|(s, _)| s)
        .unwrap_or(ToggleState::Normal);
    // Gallery builder form (label rendered separately, above).
    let widget = Toggle::new(id, "").on(on).state(state);
    paint_toggle(&widget, rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, rect);
    x + cell_w
}

/// A short left-aligned, vertically-centred label of width `w`.
pub(crate) fn label(ctx: &mut PaintCtx, theme: Theme, text: &str, x: f32, y: f32, w: f32) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Mirror a committed numeric `value` into the store's chip when it isn't being
/// edited (mirror of the Inspector/painter number-field sync).
pub(crate) fn mirror_number(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    value: f64,
    decimals: usize,
) {
    if store.focus_id() == Some(id) {
        return;
    }
    let text = if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    };
    if let Some(InteractiveState::NumberInput {
        value: v,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *v = value;
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = value;
    }
}

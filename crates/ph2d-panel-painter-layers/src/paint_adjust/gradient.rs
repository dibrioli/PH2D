//! **O editor do GRADIENT MAP do painel de ajustes** (+ o encode sRGB do preview) — filho cortado
//! do `paint_adjust.rs` por responsabilidade quando a porta da moldura do tema (wave 4 do
//! redesenho, 2026-09-05) o fez passar o teto de LOC do painel.

use super::*;

/// IEC sRGB encode (linear-light `0..1` → display byte) for the Gradient Map
/// preview, which samples the LINEAR `gradient_map_lut` the canvas uses.
fn lin_to_srgb8(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let cutoff = 0.003_130_8; // LITERAL-PX-OK: IEC sRGB linear-segment cutoff
    let s = if v <= cutoff {
        v * 12.92 // LITERAL-PX-OK: IEC sRGB transfer constant
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055 // LITERAL-PX-OK: IEC sRGB transfer constants
    };
    (s * 255.0).round() as u8 // LITERAL-PX-OK: 8-bit byte range
}

/// Bespoke Gradient Map editor: a +/− stop-button row, then a gradient preview
/// bar with its draggable stop handles (each a 1-D `CurvePoint` whose `x` is the
/// offset, tinted by the stop's color), then the SELECTED stop's RGB sliders, then
/// the interpolation segment. The selected stop is panel-local VIEW state
/// ([`state::selected_gradient_stop`]); drags/colors forward `PAINTER_GRADIENT_*`
/// from `event.rs`. Returns the next `y`.
pub(super) fn paint_gradient_map(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let AdjustmentParams::GradientMap(g) = params else {
        return y;
    };
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let n_stops = g.stops.len();
    let selected = state::selected_gradient_stop(layer_id).min(n_stops.saturating_sub(1));

    // ── + / − stop buttons (right-aligned) ──
    let add_rect = Rect::new(x + w - CURVE_BTN_W * 2.0 - gap, y, CURVE_BTN_W, ROW_H_PX);
    let rem_rect = Rect::new(x + w - CURVE_BTN_W, y, CURVE_BTN_W, ROW_H_PX);
    for (brect, label, id) in [
        (add_rect, "+", painter_gradient_add_id(layer_id)),
        (rem_rect, "−", painter_gradient_remove_id(layer_id)),
    ] {
        fill_rounded_rect(
            ctx.scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            brect,
            font,
            resolve(ColorToken::Text1, theme),
        );
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, brect);
    }
    y += ph2d_tokens::row_pitch_px();

    // ── Preview bar (samples the LINEAR LUT the canvas uses) ──
    let bar = Rect::new(x, y, w.max(1.0), GRAD_BAR_H);
    let lut = ph2d_tool_painter::gradient_map_lut(g);
    let slices = (bar.w as usize).clamp(16, 256);
    let slice_w = bar.w / slices as f32;
    for i in 0..slices {
        let off = i as f32 / (slices - 1).max(1) as f32;
        let c = lut[(off * 255.0).round() as usize]; // LITERAL-PX-OK: 8-bit LUT index
        let [r, g, b] = [lin_to_srgb8(c[0]), lin_to_srgb8(c[1]), lin_to_srgb8(c[2])];
        let col = ph2d_vector::Color::from_rgba8(r, g, b, 255); // LITERAL-COLOR-OK: gradient data
        let srect = Rect::new(bar.x + i as f32 * slice_w, bar.y, slice_w + 1.0, bar.h);
        fill_rounded_rect(ctx.scene, srect, 0.0, col);
    }
    // ⭐ Pela porta do TEMA: a barra é plana num tema moderno.
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        bar,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0, // LITERAL-PX-OK: 1px hairline border
        resolve(ColorToken::TextDisabled, theme),
    );

    // ── Draggable stop handles on the bar's bottom edge ──
    let parent = painter_gradient_editor_id(layer_id);
    let cy = bar.y + bar.h;
    for (index, stop) in g.stops.iter().enumerate().take(MAX_GRADIENT_STOPS) {
        let id = painter_gradient_stop_id(layer_id, index as u8);
        let cx = bar.x + stop.offset.clamp(0.0, 1.0) * bar.w;
        // Overwrite each frame so the carried `canvas` (the bar) tracks resizes.
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel: 0,
                index: index as u8,
                canvas: bar,
            },
        );
        let grab = Rect::new(
            cx - CURVE_GRAB_R,
            cy - CURVE_GRAB_R,
            CURVE_GRAB_R * 2.0,
            CURVE_GRAB_R * 2.0,
        );
        ctx.host.hit_index_mut().register(id, grab);
        // LITERAL-COLOR-OK: the stop's own color (data).
        let scol = ph2d_vector::Color::from_rgba8(stop.color[0], stop.color[1], stop.color[2], 255); // LITERAL-COLOR-OK: stop's own color (data)
        let ring = if index == selected {
            ColorToken::Accent
        } else {
            ColorToken::TextDisabled
        };
        fill_circle(ctx.scene, cx, cy, GRAD_HANDLE_R + 1.5, resolve(ring, theme)); // LITERAL-PX-OK: 1.5px ring outline
        fill_circle(ctx.scene, cx, cy, GRAD_HANDLE_R, scol);
    }
    y += GRAD_BAR_H + GRAD_HANDLE_R + gap;

    // ── Selected stop's RGB sliders ──
    for (slot, (label, val01)) in ph2d_tool_painter::gradient_stop_color_params(g, selected)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ph2d_tokens::row_pitch_px();
    }

    // ── Interpolation segment (Linear / Smooth) ──
    paint_segment_rack(ctx, theme, layer_id, params, x, w, y)
}

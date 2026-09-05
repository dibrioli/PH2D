//! **O editor de CURVAS do painel de ajustes** — filho cortado do `paint_adjust.rs` por
//! responsabilidade (a folga de 823 LOC que ele carregava desde a W4 do Painter morreu aqui) quando
//! a porta da moldura do tema (wave 4 do redesenho, 2026-09-05) o fez passar o teto.

use super::*;

/// Bespoke Curves editor: a row of channel tabs (RGB/R/G/B) + add/remove-point
/// buttons, over a canvas plotting the ACTIVE channel's tone curve with its FREE
/// 2-D draggable control points. Each handle registers an
/// `InteractiveState::CurvePoint` (carrying the `canvas` rect, so the dispatch
/// normalizes the drag against the full plotting area, not the small grab box) +
/// a grab rect in the `HitIndex`. The drag result is drained in `event.rs` and
/// forwarded to `PainterTool::set_curve_point`.
pub(super) fn paint_curve_editor(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    c: &CurvesParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let channel = state::active_curve_channel(layer_id);

    // ── Tab + button row: [RGB][R][G][B] … [+][−] ──
    let buttons_w = CURVE_BTN_W * 2.0 + gap;
    let tabs_w = (w - buttons_w - gap).max(0.0);
    let tab_w = tabs_w / 4.0; // LITERAL-PX-OK: 4 channel tabs (RGB + master)
    for ch in 0u8..4 {
        let trect = Rect::new(x + ch as f32 * tab_w, y, (tab_w - 2.0).max(0.0), ROW_H_PX);
        let active = ch == channel;
        let (bg, fg) = if active {
            (ColorToken::AccentSoft, ColorToken::Text1)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(ctx.scene, trect, Radius::Sm.px(), resolve(bg, theme));
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            CURVE_TAB_LABELS[ch as usize],
            trect,
            font,
            resolve(fg, theme),
        );
        let id = painter_curve_tab_id(layer_id, ch);
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, trect);
    }
    let add_rect = Rect::new(x + tabs_w + gap, y, CURVE_BTN_W, ROW_H_PX);
    let rem_rect = Rect::new(add_rect.x + CURVE_BTN_W + gap, y, CURVE_BTN_W, ROW_H_PX);
    for (brect, label, id) in [
        (add_rect, "+", painter_curve_add_id(layer_id)),
        (rem_rect, "−", painter_curve_remove_id(layer_id)),
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
    y += ROW_H_PX + gap;

    // ── Canvas: the active channel's curve + draggable points ──
    let canvas = Rect::new(x, y, w.max(1.0), CURVE_CANVAS_H);
    // ⭐ Raio e moldura pela porta do TEMA: o canvas da curva é plano num tema moderno.
    let canvas_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(
        ctx.scene,
        canvas,
        canvas_radius,
        resolve(ColorToken::Bg2, theme),
    );
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        canvas,
        canvas_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0, // LITERAL-PX-OK: 1px hairline border
        resolve(ColorToken::TextDisabled, theme),
    );
    // Quarter grid + the identity diagonal (so a curve's deviation from no-op is
    // readable) — the canonical curve-editor backdrop, behind the curve.
    let grid = resolve(ColorToken::GridLine, theme);
    for i in 1..4 {
        let f = i as f32 / 4.0; // LITERAL-PX-OK: 4 grid divisions (curve editor thirds)
        let gx = canvas.x + f * canvas.w;
        let gy = canvas.y + f * canvas.h;
        stroke_polyline(
            ctx.scene,
            &[(gx, canvas.y), (gx, canvas.y + canvas.h)],
            CURVE_GRID_W,
            grid,
        );
        stroke_polyline(
            ctx.scene,
            &[(canvas.x, gy), (canvas.x + canvas.w, gy)],
            CURVE_GRID_W,
            grid,
        );
    }
    stroke_polyline(
        ctx.scene,
        &[
            (canvas.x, canvas.y + canvas.h), // bottom-left (in 0 → out 0)
            (canvas.x + canvas.w, canvas.y), // top-right (in 1 → out 1)
        ],
        CURVE_GRID_W,
        resolve(ColorToken::GridAxis, theme),
    );
    let pts = match channel {
        1 => &c.points_r,
        2 => &c.points_g,
        3 => &c.points_b,
        _ => &c.points_rgb,
    };
    // Tint the curve + handle rings by the active channel (master = accent), so
    // it's obvious which channel you're editing.
    let curve_color = resolve(
        match channel {
            1 => ColorToken::CurveR,
            2 => ColorToken::CurveG,
            3 => ColorToken::CurveB,
            _ => ColorToken::Accent,
        },
        theme,
    );
    // Plot the channel's own tone curve (the monotone spline the GPU also bakes)
    // as a smooth polyline through its sampled output.
    let samples = (canvas.w * 0.5).clamp(48.0, 256.0) as usize; // LITERAL-PX-OK: curve sample-count clamp
    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(samples + 1);
    for k in 0..=samples {
        let t = k as f32 / samples as f32; // display x 0..1
        let yv = ph2d_tool_painter::curve_value_at(&pts.points, t);
        poly.push((canvas.x + t * canvas.w, canvas.y + (1.0 - yv) * canvas.h));
    }
    stroke_polyline(ctx.scene, &poly, CURVE_STROKE_W, curve_color);

    let parent = painter_curve_editor_id(layer_id);
    let ring = curve_color;
    let fill = resolve(ColorToken::Text1, theme);
    for (index, pt) in pts.points.iter().enumerate().take(MAX_CURVE_POINTS) {
        let id = painter_curve_point_id(layer_id, channel, index as u8);
        let cx = canvas.x + pt[0].clamp(0.0, 1.0) * canvas.w;
        let cy = canvas.y + (1.0 - pt[1].clamp(0.0, 1.0)) * canvas.h;
        // Overwrite each frame so the carried `canvas` (and active `channel`)
        // track panel resizes / tab switches (CurvePoint has no per-frame drag
        // state — the result lands in the store's `curve_point_drag` slot).
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel,
                index: index as u8,
                canvas,
            },
        );
        let grab = Rect::new(
            cx - CURVE_GRAB_R,
            cy - CURVE_GRAB_R,
            CURVE_GRAB_R * 2.0,
            CURVE_GRAB_R * 2.0,
        );
        ctx.host.hit_index_mut().register(id, grab);
        // Ring + fill (no stroke-circle helper — a slightly larger ring circle
        // under the fill gives a 1.5px outline).
        fill_circle(ctx.scene, cx, cy, CURVE_HANDLE_R + 1.5, ring); // LITERAL-PX-OK: 1.5px ring outline
        fill_circle(ctx.scene, cx, cy, CURVE_HANDLE_R, fill);
    }
    y += CURVE_CANVAS_H + gap;
    y
}

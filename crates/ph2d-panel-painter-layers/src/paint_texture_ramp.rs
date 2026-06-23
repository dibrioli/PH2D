//! The Texture section's **Color Ramp** sub-editor — maps the texture's scalar to a colour gradient
//! (the reusable `ph2d_color::ColorRamp`). Mirrors Blender's Color Ramp panel: an enable toggle, the
//! RGB/HSV/HSL **Mode** + interpolation **Interp** dropdowns, a live gradient **bar** with a marker
//! per stop, and **+ / −** to add (largest-gap) / remove (last) stops.
//!
//! Painted off the Copy [`BrushSettings`] snapshot (the tool owns the ramp); edits forward over the
//! frozen `PanelEvent` channel to `PainterTool::*_texture_ramp*`. Per-stop colour editing + stop
//! dragging are follow-ups; this v1 makes the ramp reachable + structurally editable.

use crate::paint::register_button;
use crate::paint_brush::{paint_dropdown_popover, paint_dropdown_row, paint_toggle_row};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_texture_ramp_interp_option_id,
    painter_brush_texture_ramp_mode_option_id,
};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{BrushSettings, RampColorMode, RampInterp};
use ph2d_vector::Color;

const BAR_H: f32 = 28.0; // LITERAL-PX-OK: gradient bar height
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− stop-button width
const STRIPS: usize = 64; // gradient-preview strip count
const MARK_R: f32 = 4.0; // LITERAL-PX-OK: stop marker radius
const OUTLINE_W: f32 = 1.0; // LITERAL-PX-OK: bar outline stroke width

/// Paint the Color Ramp editor at `y`, returning the next `y`. The Mode / Interp dropdowns stash
/// their open rects for the deferred [`paint_texture_ramp_popovers`] pass.
pub(crate) fn paint_texture_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // ── Enable toggle ──
    let mut y = paint_toggle_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        "Color Ramp",
        brush.texture_ramp_enabled,
    );
    if !brush.texture_ramp_enabled {
        return y; // hide the editor when off (no dead controls)
    }

    // ── Mode (RGB/HSV/HSL) + Interpolation dropdowns ──
    let mode = brush.texture_ramp_mode;
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Mode",
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        mode,
        RampColorMode::from_u8(mode).name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_ramp_mode_dd(Some((r, mode)));
    }
    let interp = brush.texture_ramp_interp;
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Interpolation",
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        interp,
        RampInterp::from_u8(interp).name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_ramp_interp_dd(Some((r, interp)));
    }

    // ── + / − stop buttons (right-aligned, above the bar) ──
    let gap = Spacing::Xs.px();
    for (brect, label, id) in [
        (
            Rect::new(x + content_w - BTN_W * 2.0 - gap, y, BTN_W, ROW_H_PX),
            "+",
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        ),
        (
            Rect::new(x + content_w - BTN_W, y, BTN_W, ROW_H_PX),
            "−",
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
        ),
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
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        );
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, brect);
    }
    y += ROW_H_PX + gap;

    // ── Gradient bar + a marker per stop ──
    let bar = Rect::new(x, y, content_w, BAR_H);
    paint_ramp_bar(ctx, theme, bar, brush);
    y + BAR_H + gap
}

/// Paint the live gradient (linear preview between stops) + a marker per stop along the bottom.
fn paint_ramp_bar(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, bar: Rect, brush: BrushSettings) {
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    let stops = &brush.texture_ramp_stops[..count];
    let strip_w = bar.w / STRIPS as f32;
    for i in 0..STRIPS {
        let t = (i as f32 + 0.5) / STRIPS as f32;
        let strip = Rect::new(bar.x + i as f32 * strip_w, bar.y, strip_w + 1.0, bar.h);
        fill_rounded_rect(ctx.scene, strip, 0.0, ramp_color_at(stops, t));
    }
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Sm.px(),
        OUTLINE_W,
        resolve(ColorToken::Border, theme),
    );
    for s in stops {
        let mx = bar.x + s[0].clamp(0.0, 1.0) * bar.w;
        fill_circle(
            ctx.scene,
            mx,
            bar.y + bar.h,
            MARK_R,
            resolve(ColorToken::Text1, theme),
        );
    }
}

/// Linear-interpolated colour at `t` for the bar preview (the real paint uses the ramp's interp mode;
/// this is a quick visual). Stops are `(pos, r, g, b, a)` in display sRGB.
fn ramp_color_at(stops: &[[f32; 5]], t: f32) -> Color {
    match stops {
        [] => rgba_color([0.0, 0.0, 0.0, 0.0, 1.0]),
        [only] => rgba_color(*only),
        _ => {
            if t <= stops[0][0] {
                return rgba_color(stops[0]);
            }
            for w in stops.windows(2) {
                let (a, b) = (w[0], w[1]);
                if t >= a[0] && t <= b[0] {
                    let f = if b[0] > a[0] {
                        (t - a[0]) / (b[0] - a[0])
                    } else {
                        0.0
                    };
                    let mix = |i: usize| a[i] + (b[i] - a[i]) * f;
                    return rgba_color([t, mix(1), mix(2), mix(3), mix(4)]);
                }
            }
            rgba_color(stops[stops.len() - 1])
        }
    }
}

/// `(pos, r, g, b, a)` (sRGB `[0,1]`) → a vello colour. LITERAL-COLOR-OK: a user-authored ramp stop
/// colour, not a theme token.
fn rgba_color(s: [f32; 5]) -> Color {
    let u = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    Color::from_rgba8(u(s[1]), u(s[2]), u(s[3]), u(s[4]))
}

/// Deferred paint of the ramp's open Mode / Interpolation dropdown popovers — drained at the end of
/// the Brush body so they sit above every row.
pub(crate) fn paint_texture_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_ramp_mode_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
            ramp_mode_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_ramp_interp_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
            ramp_interp_options(),
            chip,
            cur,
        );
    }
}

fn ramp_mode_options() -> Vec<DropdownOption<u8>> {
    (0..RampColorMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_brush_texture_ramp_mode_option_id(m),
                m,
                RampColorMode::from_u8(m).name(),
            )
        })
        .collect()
}

fn ramp_interp_options() -> Vec<DropdownOption<u8>> {
    (0..RampInterp::COUNT)
        .map(|i| {
            DropdownOption::new(
                painter_brush_texture_ramp_interp_option_id(i),
                i,
                RampInterp::from_u8(i).name(),
            )
        })
        .collect()
}

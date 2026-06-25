//! The **Shape Tone** section — the Shape's B&W *value ramp* (`ph2d_color::ValueRamp`) that remaps the
//! silhouette's tone (orthogonal to the Grain colour ramp). A grayscale twin of [`crate::paint_texture_ramp`]:
//! an enable checkbox, a `+ − [Interp] [Invert]` controls line, the grayscale bar with a value-filled
//! draggable handle per stop, and a bottom row of the selected stop's editable index / position / value
//! chips (a **value** chip, not a colour box). Edits forward over the frozen `PanelEvent` channel to
//! `PainterTool::*_shape_ramp*`.

use crate::paint::register_button;
use crate::paint_brush::paint_dropdown_chip;
use crate::paint_texture_ramp::paint_ramp_chip;
use crate::state;
use ph2d_editor_core::ids::{self as core_ids, painter_shape_ramp_handle_id};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{BrushSettings, RampInterp};
use ph2d_vector::Color;

const BAR_H: f32 = 30.0; // LITERAL-PX-OK: gradient bar height
const STRIPS: usize = 64; // gradient-preview strip count
const MARK_R: f32 = 6.0; // LITERAL-PX-OK: stop handle radius
const OUTLINE_W: f32 = 1.0; // LITERAL-PX-OK: stroke width
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a stop handle's grab box
const IDX_W: f32 = 34.0; // LITERAL-PX-OK: stop-index chip width
const POS_W: f32 = 64.0; // LITERAL-PX-OK: position chip width
const VAL_W: f32 = 64.0; // LITERAL-PX-OK: value chip width

/// Paint the collapsible **Shape Tone** section at `y`, returning the next `y`. The Interp chip stashes
/// its open rect for the deferred [`paint_shape_ramp_popovers`] pass.
pub(crate) fn paint_shape_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, collapsed) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Shape Tone",
        core_ids::PAINTER_SHAPE_RAMP_SECTION,
        core_ids::PAINTER_SHAPE_RAMP_SECTION_COLOR,
        core_ids::PAINTER_SHAPE_RAMP_RESET,
    );
    if collapsed {
        return y;
    }
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SHAPE_RAMP_ENABLE,
        "Use Shape Tone",
        brush.shape_ramp_enabled,
    );
    if !brush.shape_ramp_enabled {
        return y;
    }
    let count = (brush.shape_ramp_stop_count as usize).min(brush.shape_ramp_stops.len());
    let sel_id = state::selected_shape_ramp_stop();
    let sel = brush.shape_ramp_stops[..count]
        .iter()
        .position(|s| s[2] as u8 == sel_id)
        .unwrap_or(0);

    // ── Controls line: + − [Interp] [Invert] (no labels) ──
    let gap = Spacing::Xs.px();
    let mut cx = x;
    for (label, id) in [
        ("+", core_ids::PAINTER_SHAPE_RAMP_ADD),
        ("−", core_ids::PAINTER_SHAPE_RAMP_REMOVE),
    ] {
        cx = icon_button(ctx, theme, cx, y, label, id) + gap;
    }
    let interp = brush.shape_ramp_interp;
    let interp_w = (x + content_w - cx - gap - ROW_H_PX).max(0.0);
    let interp_rect = Rect::new(cx, y, interp_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        core_ids::PAINTER_SHAPE_RAMP_INTERP,
        interp,
        RampInterp::from_u8(interp).name(),
        interp_rect,
    ) {
        state::set_pending_shape_ramp_interp_dd(Some((interp_rect, interp)));
    }
    icon_button(
        ctx,
        theme,
        x + content_w - ROW_H_PX,
        y,
        "I",
        core_ids::PAINTER_SHAPE_RAMP_INVERT,
    );
    y += ROW_H_PX + gap;

    // ── Grayscale bar + a value-filled draggable handle per stop ──
    let bar = Rect::new(x, y, content_w, BAR_H);
    paint_bar(ctx, theme, bar, brush, sel);
    y += BAR_H + MARK_R + Spacing::Sm.px();

    // ── Bottom row: the selected stop's editable index / position / value chips ──
    paint_bottom(ctx, theme, x, content_w, y, brush, sel);
    y + ROW_H_PX + Spacing::Xs.px()
}

/// One square icon button (`+` / `−` / `I`); registers it + its hit rect. Returns its right edge `x`.
fn icon_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    y: f32,
    label: &str,
    id: ph2d_a11y::NodeId,
) -> f32 {
    let r = Rect::new(x, y, ROW_H_PX, ROW_H_PX);
    fill_rounded_rect(ctx.scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        label,
        r,
        TypeToken::Base.px(),
        resolve(ColorToken::Text1, theme),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
    x + ROW_H_PX
}

/// The grayscale gradient + a value-filled circular handle per stop (selected one ringed accent).
fn paint_bar(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, bar: Rect, brush: BrushSettings, sel: usize) {
    let count = (brush.shape_ramp_stop_count as usize).min(brush.shape_ramp_stops.len());
    let stops = &brush.shape_ramp_stops[..count];
    let strip_w = bar.w / STRIPS as f32;
    for i in 0..STRIPS {
        let t = (i as f32 + 0.5) / STRIPS as f32;
        let strip = Rect::new(bar.x + i as f32 * strip_w, bar.y, strip_w + 1.0, bar.h);
        fill_rounded_rect(ctx.scene, strip, 0.0, gray(value_at(stops, t)));
    }
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Sm.px(),
        OUTLINE_W,
        resolve(ColorToken::Border, theme),
    );
    let my = bar.y + bar.h;
    for (i, s) in stops.iter().enumerate() {
        let mx = bar.x + s[0].clamp(0.0, 1.0) * bar.w;
        let stop_id = s[2] as u8;
        let id = painter_shape_ramp_handle_id(stop_id);
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent: core_ids::PAINTER_SHAPE_RAMP_EDIT,
                channel: 0,
                index: stop_id,
                canvas: bar,
            },
        );
        ctx.host.hit_index_mut().register(
            id,
            Rect::new(mx - GRAB_R, my - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0),
        );
        let ring = if i == sel {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        fill_circle(ctx.scene, mx, my, MARK_R, resolve(ring, theme));
        fill_circle(ctx.scene, mx, my, MARK_R - OUTLINE_W, gray(s[1]));
    }
}

/// The bottom row: index selector + position chip + the selected stop's **value** chip (`0..1`).
fn paint_bottom(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    _content_w: f32,
    y: f32,
    brush: BrushSettings,
    sel: usize,
) {
    let count = (brush.shape_ramp_stop_count as usize).min(brush.shape_ramp_stops.len());
    if count == 0 {
        return;
    }
    let s = brush.shape_ramp_stops[sel.min(count - 1)];
    let gap = Spacing::Xs.px();
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x, y, IDX_W, ROW_H_PX),
        core_ids::PAINTER_SHAPE_RAMP_STOP_INDEX,
        sel as f64,
        &format!("{sel}"),
    );
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x + IDX_W + gap, y, POS_W, ROW_H_PX),
        core_ids::PAINTER_SHAPE_RAMP_STOP_POS,
        f64::from(s[0]),
        &format!("{:.3}", s[0]),
    );
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x + IDX_W + POS_W + gap * 2.0, y, VAL_W, ROW_H_PX),
        core_ids::PAINTER_SHAPE_RAMP_STOP_VALUE,
        f64::from(s[1]),
        &format!("{:.3}", s[1]),
    );
}

/// Deferred paint of the Shape ramp's open Interpolation dropdown popover.
pub(crate) fn paint_shape_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_shape_ramp_interp_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_RAMP_INTERP,
            (0..RampInterp::COUNT)
                .map(|i| {
                    ph2d_editor_core::widget::DropdownOption::new(
                        core_ids::painter_shape_ramp_interp_option_id(i),
                        i,
                        RampInterp::from_u8(i).name(),
                    )
                })
                .collect(),
            chip,
            cur,
        );
    }
}

/// Linear-interpolated grayscale value at `t` for the bar preview (the real paint uses the ramp's
/// interp mode; this is a quick visual). Stops are `(pos, value, id)`.
fn value_at(stops: &[[f32; 3]], t: f32) -> f32 {
    match stops {
        [] => 0.0,
        [only] => only[1],
        _ => {
            if t <= stops[0][0] {
                return stops[0][1];
            }
            for w in stops.windows(2) {
                let (a, b) = (w[0], w[1]);
                if t >= a[0] && t <= b[0] {
                    let f = if b[0] > a[0] {
                        (t - a[0]) / (b[0] - a[0])
                    } else {
                        0.0
                    };
                    return a[1] + (b[1] - a[1]) * f;
                }
            }
            stops[stops.len() - 1][1]
        }
    }
}

/// A grayscale `[0,1]` value → an opaque vello colour. LITERAL-COLOR-OK: user-authored ramp value.
fn gray(v: f32) -> Color {
    let u = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: 8-bit value
    Color::from_rgba8(u, u, u, 255) // LITERAL-COLOR-OK: ramp value is user data, not a theme token
}

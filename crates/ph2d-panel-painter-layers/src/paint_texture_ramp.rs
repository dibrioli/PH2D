//! The Texture section's **Color Ramp** sub-editor — maps the texture's scalar to a colour gradient
//! (the reusable `ph2d_color::ColorRamp`). Blender-style layout: the enable toggle; one compact
//! controls line (`+` `−` + the RGB/HSV/HSL **Mode** and interpolation chips, no labels); the live
//! gradient **bar** with a colour-filled draggable handle per stop; and a bottom row of the selected
//! stop's **editable** index selector + position chips and one **colour box** showing the final
//! colour with alpha (over a checker) that opens the shared picker.
//!
//! The bar handles reuse the foundational `CurvePoint` drag (as the falloff editor) — drag = position,
//! click = select. The index / position chips are `NumberInput`s (type-to-edit) registered in the
//! store; the selected stop drives the bottom row + the colour box. Edits forward over the frozen
//! `PanelEvent` channel to `PainterTool::*_texture_ramp*`.

use crate::paint::register_button;
use crate::paint_brush::{paint_dropdown_chip, paint_dropdown_popover, paint_toggle_row};
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_texture_ramp_handle_id,
    painter_brush_texture_ramp_interp_option_id, painter_brush_texture_ramp_mode_option_id,
};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    ColorSwatch, DropdownOption, SwatchSize, SwatchState, TextInputState, paint_color_swatch,
    paint_number_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{BrushSettings, RampColorMode, RampInterp};
use ph2d_vector::Color;

const BAR_H: f32 = 30.0; // LITERAL-PX-OK: gradient bar height
const STRIPS: usize = 64; // gradient-preview strip count
const MARK_R: f32 = 6.0; // LITERAL-PX-OK: stop handle radius (colour-filled circle)
const OUTLINE_W: f32 = 1.0; // LITERAL-PX-OK: stroke width
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a stop handle's pointer grab box
const IDX_W: f32 = 34.0; // LITERAL-PX-OK: stop-index chip width
const POS_W: f32 = 70.0; // LITERAL-PX-OK: position chip width

/// Paint the Color Ramp editor at `y`, returning the next `y`. The Mode / Interp chips stash their
/// open rects for the deferred [`paint_texture_ramp_popovers`] pass.
pub(crate) fn paint_texture_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
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
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    // The selection is a STABLE id; resolve it to the current sorted index (so it follows a stop that
    // was dragged past a neighbour).
    let sel_id = state::selected_ramp_stop();
    let sel = brush.texture_ramp_stops[..count]
        .iter()
        .position(|s| s[5] as u8 == sel_id)
        .unwrap_or(0);

    // ── Controls line: + − [Mode] [Interp] (no labels) ──
    let gap = Spacing::Xs.px();
    let mut cx = x;
    for (label, id) in [
        ("+", core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD),
        ("−", core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE),
    ] {
        let r = Rect::new(cx, y, ROW_H_PX, ROW_H_PX);
        fill_rounded_rect(
            ctx.scene,
            r,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
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
        cx += ROW_H_PX + gap;
    }
    let chip_w = ((x + content_w - cx - gap) * 0.5).max(0.0);
    let mode = brush.texture_ramp_mode;
    let mode_rect = Rect::new(cx, y, chip_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        mode,
        RampColorMode::from_u8(mode).name(),
        mode_rect,
    ) {
        state::set_pending_ramp_mode_dd(Some((mode_rect, mode)));
    }
    let interp = brush.texture_ramp_interp;
    let interp_rect = Rect::new(cx + chip_w + gap, y, chip_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        interp,
        RampInterp::from_u8(interp).name(),
        interp_rect,
    ) {
        state::set_pending_ramp_interp_dd(Some((interp_rect, interp)));
    }
    y += ROW_H_PX + gap;

    // ── Gradient bar + a colour-filled draggable handle per stop ──
    let bar = Rect::new(x, y, content_w, BAR_H);
    paint_ramp_bar(ctx, theme, bar, brush, sel);
    // Clear the handles that hang MARK_R below the bar, plus breathing room before the bottom row.
    y += BAR_H + MARK_R + Spacing::Sm.px();

    // ── Bottom row: editable index + position chips + the final-colour box (edits the selected stop) ──
    y = paint_ramp_bottom(ctx, theme, x, content_w, y, brush, sel);
    ramp_color_readback(ctx, brush, sel);
    y
}

/// The gradient preview + a colour-filled circular handle per stop (the selected one ringed accent).
fn paint_ramp_bar(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    bar: Rect,
    brush: BrushSettings,
    sel: usize,
) {
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
    // Each marker is a colour-filled handle: a `CurvePoint` whose x → the stop position (y ignored).
    let my = bar.y + bar.h;
    for (i, s) in stops.iter().enumerate() {
        let mx = bar.x + s[0].clamp(0.0, 1.0) * bar.w;
        let stop_id = s[5] as u8; // the stable stop id (so a drag can cross neighbours)
        let id = painter_brush_texture_ramp_handle_id(stop_id);
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT,
                channel: 0,
                index: stop_id,
                canvas: bar,
            },
        );
        ctx.host.hit_index_mut().register(
            id,
            Rect::new(mx - GRAB_R, my - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0),
        );
        // White outer ring (accent if selected) so the marker reads on any gradient; filled colour.
        let ring = if i == sel {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        fill_circle(ctx.scene, mx, my, MARK_R, resolve(ring, theme));
        fill_circle(ctx.scene, mx, my, MARK_R - OUTLINE_W, rgba_color(*s));
    }
}

/// The bottom row: the selected stop's **editable** index selector + position chips, then one colour
/// box showing the final colour **with alpha** (a checker reads the translucency) that opens the picker.
fn paint_ramp_bottom(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
    sel: usize,
) -> f32 {
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    if count == 0 {
        return y;
    }
    let s = brush.texture_ramp_stops[sel.min(count - 1)];
    let gap = Spacing::Xs.px();
    // Editable index selector (whole number → selects that sorted stop) + position (0..1 → moves it).
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x, y, IDX_W, ROW_H_PX),
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX,
        sel as f64,
        &format!("{sel}"),
    );
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x + IDX_W + gap, y, POS_W, ROW_H_PX),
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS,
        f64::from(s[0]),
        &format!("{:.3}", s[0]),
    );
    // One colour box: the final colour WITH alpha (the swatch checkers behind translucency).
    let bx = x + IDX_W + POS_W + gap * 2.0;
    let box_rect = Rect::new(bx, y, (x + content_w - bx).max(0.0), ROW_H_PX);
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let open =
        ctx.host.store().picker_target() == Some(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH);
    paint_color_swatch(
        &ColorSwatch {
            id: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
            label: String::new(),
            rgba: [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])],
            // Focused emphasizes the swatch border — our "picker open" affordance.
            state: if open {
                SwatchState::Focused
            } else {
                SwatchState::Normal
            },
            size: SwatchSize::Sm,
        },
        box_rect,
        ctx.scene,
        theme,
    );
    register_button(
        ctx.host.store_mut(),
        core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
    );
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH, box_rect);
    y + ROW_H_PX + gap
}

/// Paint one editable [`paint_number_chip`] driven by a `NumberInput` in the store: register it once
/// (`register_if_absent`, so live typing isn't clobbered), mirror `value`/`text` into it while it
/// isn't focused, then render the in-progress buffer + caret. The global dispatch handles focus +
/// keys + commit; the panel's `apply_event` reacts to the `ValueChanged`.
fn paint_ramp_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    chip_id: NodeId,
    value: f64,
    text: &str,
) {
    let store = ctx.host.store_mut();
    let _ = store.register_if_absent(
        chip_id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: text.to_string(),
            caret: text.len(),
            last_committed: value,
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(chip_id)
        && let Some(InteractiveState::NumberInput {
            value: v,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(chip_id)
    {
        *v = value;
        buffer.clear();
        buffer.push_str(text);
        *caret = buffer.len();
        *last_committed = value;
    }
    let (st, buf, caret, anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, buffer.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    paint_number_chip(
        rect,
        st,
        value,
        None,
        Some(&buf),
        caret,
        anchor,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(chip_id, rect);
}

/// When the shared picker targets the colour box, forward its live colour (incl. **alpha**) to the
/// **selected** stop (as `"id,r,g,b,a"`) once it differs. Mirrors `paint_brush::brush_color_readback`.
fn ramp_color_readback(ctx: &mut PaintCtx, brush: BrushSettings, sel: usize) {
    if ctx.host.store().picker_target() != Some(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH) {
        return;
    }
    let Some(picked) = ctx
        .host
        .store()
        .widget_color(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH)
    else {
        return;
    };
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    if sel >= count {
        return;
    }
    let s = brush.texture_ramp_stops[sel];
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    if [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])] == picked {
        return; // already applied (RGBA) — don't spam the bus
    }
    let stop_id = s[5] as u8; // forward by stable id (the tool resolves the current index)
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
            format!(
                "{stop_id},{},{},{},{}",
                picked[0], picked[1], picked[2], picked[3]
            ),
        )));
}

/// Linear-interpolated colour at `t` for the bar preview (the real paint uses the ramp's interp mode;
/// this is a quick visual). Stops are `(pos, r, g, b, a)` in display sRGB.
fn ramp_color_at(stops: &[[f32; 6]], t: f32) -> Color {
    match stops {
        [] => rgba_color([0.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
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
                    return rgba_color([t, mix(1), mix(2), mix(3), mix(4), 0.0]);
                }
            }
            rgba_color(stops[stops.len() - 1])
        }
    }
}

/// `(pos, r, g, b, a)` (sRGB `[0,1]`) → a vello colour. LITERAL-COLOR-OK: a user-authored ramp stop
/// colour, not a theme token.
fn rgba_color(s: [f32; 6]) -> Color {
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

//! The shared **Colour Ramp** editor body — one reusable widget driven by an id-bundle ([`RampIds`])
//! plus a snapshot view ([`RampView`]), so the Grain ramp ([`crate::paint_texture_ramp`]) and the
//! Shape ramp ([`crate::paint_shape_ramp`]) render from the same code, each keying its own widgets.
//!
//! Layout (Blender-style): the "Use Color Ramp" enable checkbox; a compact controls line of
//! `+ − I B&W` square buttons + the colour **Mode** / **Interpolation** dropdowns (which split onto a
//! second row when the panel is narrow); the gradient **bar** with a colour-filled draggable handle
//! per stop; and a bottom row of the selected stop's editable index / position chips + a colour box.
//! The **B&W** filter, when on, desaturates the displayed gradient (a pure display + paint filter; the
//! authored stop colours are untouched, so toggling it back restores them).

use crate::paint::register_button;
use crate::paint_brush::paint_dropdown_chip;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, SwatchSize, SwatchState, TextInputState, flat_button_surface,
    paint_color_swatch, paint_number_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{
    ColorRamp, RampAlphaMode, RampColorMode, RampInterp, RampStop, linear_to_srgb_byte,
    srgb_to_linear_byte,
};
use ph2d_vector::Color;

const BAR_H: f32 = 30.0; // LITERAL-PX-OK: gradient bar height
const STRIPS: usize = 64; // gradient-preview strip count
const MARK_R: f32 = 6.0; // LITERAL-PX-OK: stop handle radius (colour-filled circle)
const OUTLINE_W: f32 = 1.0; // LITERAL-PX-OK: stroke width
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a stop handle's pointer grab box
const IDX_W: f32 = 34.0; // LITERAL-PX-OK: stop-index chip width
const POS_W: f32 = 70.0; // LITERAL-PX-OK: position chip width
const BW_W: f32 = 42.0; // LITERAL-PX-OK: "B&W" toggle button width (wider than a square icon button)
const DD_MIN_W: f32 = 56.0; // LITERAL-PX-OK: min usable width of one Mode/Interp dropdown chip

/// The fixed-id widget set + per-instance state hooks for one Colour Ramp editor (Grain or Shape).
#[derive(Clone, Copy)]
pub(crate) struct RampIds {
    pub section: NodeId,
    pub section_color: NodeId,
    pub reset: NodeId,
    pub enable: NodeId,
    pub mode: NodeId,
    pub interp: NodeId,
    pub alpha_mode: NodeId,
    pub bw: NodeId,
    pub add: NodeId,
    pub remove: NodeId,
    pub invert: NodeId,
    pub edit: NodeId,
    pub swatch: NodeId,
    pub stop_index: NodeId,
    pub stop_pos: NodeId,
    /// Stable handle id for stop `i` on the bar.
    pub handle: fn(u8) -> NodeId,
    /// Stash the open Mode / Interp / Alpha dropdown rect for this instance's deferred popover pass.
    pub set_pending_mode: fn(Option<(Rect, u8)>),
    pub set_pending_interp: fn(Option<(Rect, u8)>),
    pub set_pending_alpha: fn(Option<(Rect, u8)>),
}

/// A snapshot view of one ramp's published fields. `stops` is the valid slice `(pos, r, g, b, a, id)`
/// in display sRGB; `selected_id` is the stable id of the selected stop (panel-local selection state).
#[derive(Clone, Copy)]
pub(crate) struct RampView<'a> {
    pub enabled: bool,
    pub bw: bool,
    pub mode: u8,
    pub interp: u8,
    pub alpha_mode: u8,
    pub stops: &'a [[f32; 6]],
    pub selected_id: u8,
}

/// Paint the Colour Ramp editor at `y`, returning the next `y`. `title` names the collapsible section.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_color_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    title: &str,
    ids: &RampIds,
    view: RampView,
) -> f32 {
    let (mut y, collapsed) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        title,
        ids.section,
        ids.section_color,
        ids.reset,
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
        ids.enable,
        "Use Color Ramp",
        view.enabled,
    );
    if !view.enabled {
        return y; // hide the editor when off (no dead controls)
    }
    let sel = view
        .stops
        .iter()
        .position(|s| s[5] as u8 == view.selected_id)
        .unwrap_or(0);

    y = paint_controls(ctx, theme, x, content_w, y, ids, view);

    // Gradient bar + a colour-filled draggable handle per stop.
    let bar = Rect::new(x, y, content_w, BAR_H);
    paint_ramp_bar(ctx, theme, bar, ids, view, sel);
    y += BAR_H + MARK_R + Spacing::Sm.px();

    // Bottom row: editable index + position chips + the final-colour box (edits the selected stop).
    y = paint_ramp_bottom(ctx, theme, x, content_w, y, ids, view, sel);
    ramp_color_readback(ctx, ids, view, sel);

    // Alpha action dropdown (Off / Reduce Strength / Sprite Alpha).
    let alpha_rect = Rect::new(x, y, content_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        ids.alpha_mode,
        view.alpha_mode,
        RampAlphaMode::from_u8(view.alpha_mode).name(),
        alpha_rect,
    ) {
        (ids.set_pending_alpha)(Some((alpha_rect, view.alpha_mode)));
    }
    y + ROW_H_PX + Spacing::Xs.px()
}

/// The controls line: `+ − I [B&W]` square/toggle buttons, then the Mode / Interp dropdowns. When the
/// panel is too narrow to fit both groups on one line, the dropdowns wrap to a second row.
fn paint_controls(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    ids: &RampIds,
    view: RampView,
) -> f32 {
    let gap = Spacing::Xs.px();
    // Buttons: + − I (square) then B&W (wide toggle).
    let mut cx = x;
    for (label, id) in [("+", ids.add), ("−", ids.remove), ("I", ids.invert)] {
        icon_button(
            ctx,
            theme,
            Rect::new(cx, y, ROW_H_PX, ROW_H_PX),
            label,
            id,
            false,
        );
        cx += ROW_H_PX + gap;
    }
    icon_button(
        ctx,
        theme,
        Rect::new(cx, y, BW_W, ROW_H_PX),
        "B&W",
        ids.bw,
        view.bw,
    );
    cx += BW_W + gap;
    let buttons_right = cx;
    // One row only if the two dropdowns still fit to the right of the buttons; else wrap to row 2.
    let one_row = (x + content_w) - buttons_right >= DD_MIN_W * 2.0 + gap;
    let (dd_x, dd_y, dd_w) = if one_row {
        let total = (x + content_w) - buttons_right;
        (buttons_right, y, (total - gap) * 0.5)
    } else {
        (x, y + ROW_H_PX + gap, (content_w - gap) * 0.5)
    };
    let mode_rect = Rect::new(dd_x, dd_y, dd_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        ids.mode,
        view.mode,
        RampColorMode::from_u8(view.mode).name(),
        mode_rect,
    ) {
        (ids.set_pending_mode)(Some((mode_rect, view.mode)));
    }
    let interp_rect = Rect::new(dd_x + dd_w + gap, dd_y, dd_w, ROW_H_PX);
    if paint_dropdown_chip(
        ctx,
        theme,
        ids.interp,
        view.interp,
        RampInterp::from_u8(view.interp).name(),
        interp_rect,
    ) {
        (ids.set_pending_interp)(Some((interp_rect, view.interp)));
    }
    let rows = if one_row { 1.0 } else { 2.0 };
    y + rows * ROW_H_PX + (rows - 1.0) * gap + gap
}

/// One control button (`+` / `−` / `I` / `B&W`); `active` fills it accent (a toggle's on-state).
fn icon_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    r: Rect,
    label: &str,
    id: NodeId,
    active: bool,
) {
    // Active toggles stay accent; otherwise the fill follows the button's ButtonState (hover/press) via
    // the central `flat_button_surface`, so every control button shows mouse feedback.
    let (bg, fg) = if active {
        (ColorToken::Accent, ColorToken::Bg0)
    } else {
        let state = match ctx.host.store().get(id) {
            Some(InteractiveState::Button { state }) => *state,
            _ => ButtonState::Normal,
        };
        (flat_button_surface(state), ColorToken::Text1)
    };
    fill_rounded_rect(ctx.scene, r, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        label,
        r,
        TypeToken::Base.px(),
        resolve(fg, theme),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
}

/// The gradient preview + a colour-filled circular handle per stop (selected one ringed accent). The
/// strips desaturate when `view.bw` is on (the filter's preview); the handles keep their authored colour.
fn paint_ramp_bar(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    bar: Rect,
    ids: &RampIds,
    view: RampView,
    sel: usize,
) {
    let strip_w = bar.w / STRIPS as f32;
    for i in 0..STRIPS {
        let t = (i as f32 + 0.5) / STRIPS as f32;
        let strip = Rect::new(bar.x + i as f32 * strip_w, bar.y, strip_w + 1.0, bar.h);
        fill_rounded_rect(ctx.scene, strip, 0.0, ramp_color_at(view.stops, t, view.bw));
    }
    stroke_rounded_rect(
        ctx.scene,
        bar,
        Radius::Sm.px(),
        OUTLINE_W,
        resolve(ColorToken::Border, theme),
    );
    let my = bar.y + bar.h;
    for (i, s) in view.stops.iter().enumerate() {
        let mx = bar.x + s[0].clamp(0.0, 1.0) * bar.w;
        let stop_id = s[5] as u8;
        let id = (ids.handle)(stop_id);
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent: ids.edit,
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
        fill_circle(ctx.scene, mx, my, MARK_R - OUTLINE_W, rgba_color(*s));
    }
}

/// The bottom row: the selected stop's editable index + position chips, then one colour box (the final
/// colour with alpha over a checker) that opens the picker.
#[allow(clippy::too_many_arguments)]
fn paint_ramp_bottom(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    ids: &RampIds,
    view: RampView,
    sel: usize,
) -> f32 {
    if view.stops.is_empty() {
        return y;
    }
    let s = view.stops[sel.min(view.stops.len() - 1)];
    let gap = Spacing::Xs.px();
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x, y, IDX_W, ROW_H_PX),
        ids.stop_index,
        sel as f64,
        &format!("{sel}"),
    );
    paint_ramp_chip(
        ctx,
        theme,
        Rect::new(x + IDX_W + gap, y, POS_W, ROW_H_PX),
        ids.stop_pos,
        f64::from(s[0]),
        &format!("{:.3}", s[0]),
    );
    let bx = x + IDX_W + POS_W + gap * 2.0;
    let box_rect = Rect::new(bx, y, (x + content_w - bx).max(0.0), ROW_H_PX);
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let open = ctx.host.store().picker_target() == Some(ids.swatch);
    paint_color_swatch(
        &ColorSwatch {
            id: ids.swatch,
            label: String::new(),
            rgba: [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])],
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
    register_button(ctx.host.store_mut(), ids.swatch);
    ctx.host.hit_index_mut().register(ids.swatch, box_rect);
    y + ROW_H_PX + gap
}

/// Paint one editable [`paint_number_chip`] driven by a `NumberInput`: register once, mirror
/// `value`/`text` while unfocused, then render the in-progress buffer + caret. Shared by both ramps.
pub(crate) fn paint_ramp_chip(
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

/// When the picker targets this ramp's colour box, forward its live colour (incl. alpha) to the
/// selected stop (as `"id,r,g,b,a"`) once it differs. The tool's swatch decode sets that stop's colour.
fn ramp_color_readback(ctx: &mut PaintCtx, ids: &RampIds, view: RampView, sel: usize) {
    if ctx.host.store().picker_target() != Some(ids.swatch) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(ids.swatch) else {
        return;
    };
    if sel >= view.stops.len() {
        return;
    }
    let s = view.stops[sel];
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    if [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])] == picked {
        return;
    }
    let stop_id = s[5] as u8;
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids.swatch,
            format!(
                "{stop_id},{},{},{},{}",
                picked[0], picked[1], picked[2], picked[3]
            ),
        )));
}

/// Linear-interpolated colour at `t` for the bar preview (a quick visual; the real paint uses the
/// ramp's interp mode). Stops are `(pos, r, g, b, a)` display sRGB; `bw` desaturates to luminance.
fn ramp_color_at(stops: &[[f32; 6]], t: f32, bw: bool) -> Color {
    let mk = |s: [f32; 6]| {
        if bw {
            let l = 0.2126 * s[1] + 0.7152 * s[2] + 0.0722 * s[3]; // LITERAL-PX-OK: Rec.709 luma weights
            rgba_color([s[0], l, l, l, s[4], s[5]])
        } else {
            rgba_color(s)
        }
    };
    match stops {
        [] => Color::from_rgba8(0, 0, 0, 0), // LITERAL-COLOR-OK: empty-ramp transparent fallback (a ramp always keeps ≥1 stop)
        [only] => mk(*only),
        _ => {
            if t <= stops[0][0] {
                return mk(stops[0]);
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
                    return mk([t, mix(1), mix(2), mix(3), mix(4), 0.0]);
                }
            }
            mk(stops[stops.len() - 1])
        }
    }
}

/// `(pos, r, g, b, a)` (sRGB `[0,1]`) → a vello colour. LITERAL-COLOR-OK: a user-authored ramp stop
/// colour, not a theme token.
fn rgba_color(s: [f32; 6]) -> Color {
    let u = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    Color::from_rgba8(u(s[1]), u(s[2]), u(s[3]), u(s[4])) // LITERAL-COLOR-OK: ramp stop colour is user data, not a theme token
}

/// Bake the **exact** `ColorRamp` from `stops` (display sRGB → linear), honouring `mode` / `interp`,
/// then the `bw` luminance filter, into `out` as a 256-entry **sRGB-straight RGBA** LUT — the same bake
/// the tool paints with, so a preview is faithful. Returns `false` (→ grayscale scalar) when off / empty.
/// Shared by the Grain ([`crate::paint_texture`]) + Shape ([`crate::paint_shape`]) previews.
pub(crate) fn build_preview_lut(
    enabled: bool,
    bw: bool,
    mode: u8,
    interp: u8,
    stops: &[[f32; 6]],
    out: &mut [[f32; 4]; 256],
) -> bool {
    if !enabled || stops.is_empty() {
        return false;
    }
    let s2l = |c: f32| srgb_to_linear_byte((c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8); // LITERAL-PX-OK: sRGB 8-bit normalize
    let rs: Vec<RampStop> = stops
        .iter()
        .map(|s| RampStop::new(s[0], [s2l(s[1]), s2l(s[2]), s2l(s[3]), s[4]]))
        .collect();
    let ramp = ColorRamp::new(
        rs,
        RampColorMode::from_u8(mode),
        RampInterp::from_u8(interp),
    );
    ramp.bake_into(out); // linear RGBA in the chosen interp/colour space
    for c in out.iter_mut() {
        if bw {
            let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]; // LITERAL-PX-OK: Rec.709 luma weights
            c[0] = l;
            c[1] = l;
            c[2] = l;
        }
        c[0] = f32::from(linear_to_srgb_byte(c[0])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[1] = f32::from(linear_to_srgb_byte(c[1])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
        c[2] = f32::from(linear_to_srgb_byte(c[2])) / 255.0; // LITERAL-PX-OK: sRGB 8-bit normalize
    }
    true
}

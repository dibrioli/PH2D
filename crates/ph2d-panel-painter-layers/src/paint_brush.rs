//! The Painter dock's **Brush-properties** view (header toggle → "Brush"): the
//! active brush's Size slider, blend-mode chip, and a colour swatch that opens
//! the shared Blender colour picker (`INSP_BLENDER_PICKER`, the same rich picker
//! the Inspector uses — only one is ever open, so they share the slot).
//!
//! The brush is tool-global, so these are FIXED-id widgets (registered in
//! [`crate::populate`]). The panel reads the published [`BrushSettings`] snapshot
//! to position them and forwards edits over the frozen `PanelEvent` channel. The
//! colour round-trip is a per-frame read-back: when the floating picker targets our
//! swatch, its live value (mirrored by the hero loop into `widget_color(target)`) is forwarded.

use crate::paint::register_button;
use crate::state;
use ph2d_editor_core::IconId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_blend_option_id, painter_brush_falloff_option_id,
};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::panel_chrome::PANEL_HEAD_PAD;
use ph2d_editor_core::widget::{DropdownOption, DropdownState, Slider, SliderState, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{
    BrushBlend, BrushSettings, Falloff, FalloffPoint, HandleType, MAX_BRUSH_BLEND_MODES,
    MAX_FALLOFF,
};

const LABEL_W: f32 = 60.0; // LITERAL-PX-OK: brush row label column ("Hardness"/"Strength")
const READOUT_W: f32 = 30.0; // LITERAL-PX-OK: param readout column

/// Padding entry for the fixed-size `falloff_points` array (only the first
/// `falloff_len` are read).
const FALLOFF_PT_NIL: FalloffPoint = FalloffPoint {
    id: 0,
    x: 0.0,
    y: 0.0,
    handle: HandleType::Auto,
};

/// Brush used before the first snapshot publish (Painter just activated; the bridge then publishes
/// every frame). `pub(crate)` so the Stroke-section paint tests reuse it as a Blender-default base
/// (struct-update the one field under test).
pub(crate) const FALLBACK_BRUSH: BrushSettings = BrushSettings {
    size_px: 25.0,    // LITERAL-PX-OK: defensive pre-publish fallback default
    size_norm: 0.217, // LITERAL-PX-OK: defensive pre-publish fallback default
    strength: 1.0,
    falloff: 0,
    // Default Custom curve = linear ramp (0,1)→(1,0); only read when falloff == Custom.
    falloff_points: [
        FalloffPoint {
            id: 0,
            x: 0.0,
            y: 1.0,
            handle: HandleType::Auto,
        },
        FalloffPoint {
            id: 1,
            x: 1.0,
            y: 0.0,
            handle: HandleType::Auto,
        },
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
    ],
    falloff_len: 2,
    color: [0.0, 0.0, 0.0],
    blend: 0,
    eraser: false,
    tiling: [false, false],
    repeat_image: false,
    // Stroke section (mirrors BrushSpec::default / Blender defaults).
    stroke_method: 3,         // Space
    spacing: 0.10,            // LITERAL-PX-OK: Blender brush default (mirrors BrushSpec::default)
    space_attenuation: false, // Adjust Strength off by default (Enio 2026-06-24; mirrors BrushSpec::default)
    accumulate: false,
    jitter: 0.0,
    jitter_absolute_px: 0.0,
    jitter_unit: 0, // Brush
    dash_ratio: 1.0,
    dash_samples: 20,
    input_samples: 1,
    stabilizer: 0.5,
    airbrush_rate_s: 0.1, // LITERAL-PX-OK: Blender default (DNA_brush_types.h:232)
    edge_to_edge: false,
    // Texture section (mirrors TextureSettings::default — no texture assigned).
    texture_kind: 0,    // None
    texture_mapping: 0, // View Plane
    texture_angle_deg: 0,
    texture_rake: false,
    texture_random: false,
    texture_offset: [0.0, 0.0],
    texture_size: [1.0, 1.0],
    texture_params: [0.5; ph2d_tool_painter::MAX_TEX_PARAMS],
    grain_depth: 1.0,
    // Shape section (mirrors BrushSpec::default — kind None, no Shape image, silhouette = falloff).
    shape_kind: 0,
    shape_has_image: false,
    shape_angle_deg: 0,
    shape_rake: false,
    shape_random: false,
    shape_offset: [0.0, 0.0],
    shape_size: [1.0, 1.0],
    shape_params: [0.5; ph2d_tool_painter::MAX_TEX_PARAMS],
    texture_ramp_enabled: false,
    texture_ramp_mode: 0,
    texture_ramp_interp: 2,
    texture_ramp_stops: [[0.0; 6]; ph2d_tool_painter::PANEL_RAMP_STOPS],
    texture_ramp_stop_count: 0,
    texture_ramp_alpha_mode: 0,
    color_jitter_enabled: false,
    color_jitter: [0.0, 0.0, 0.0],
    jitter_scale: 0.0,
    jitter_rotate: 0.0,
    jitter_spacing: 0.0,
};

/// Paint the Brush-properties body rows from `top_y` (already panel-scroll-offset), returning the
/// content-bottom `y`. The caller clips to the body viewport, measures the height for the scrollbar,
/// and drains the dropdown popovers via [`paint_brush_popovers`] AFTER popping the clip.
pub(crate) fn paint_brush_body(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    top_y: f32,
) -> f32 {
    let x = rect.x + PANEL_HEAD_PAD;
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;
    let brush = state::current_brush().unwrap_or(FALLBACK_BRUSH);
    // If the shared picker is editing our swatch, forward its live colour.
    brush_color_readback(ctx, brush);

    let mut y = top_y;
    use crate::paint_brush_top::{
        paint_checkbox_row, paint_randomize_section, paint_slider_chip_row,
    };

    // ── TOP basics (no section), reordered (Enio 2026-06-24):
    //    Blend · Color · Size · Strength · Accumulate · Falloff ──

    // 1. Blend
    let (ny, blend_open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Blend",
        core_ids::PAINTER_BRUSH_BLEND,
        brush.blend,
        BrushBlend::from_u8(brush.blend).name(),
    );
    y = ny;
    if let Some(r) = blend_open {
        state::set_pending_brush_blend_dd(Some((r, brush.blend)));
    }

    // 2. Color
    y = paint_color_swatch_row(ctx, theme, x, content_w, y, brush);

    // 3. Size + 4. Strength — canonical slider + editable numeric chip (Widget-Gallery look).
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_BRUSH_SIZE_SLIDER,
        core_ids::PAINTER_BRUSH_SIZE_CHIP,
        brush.size_norm,
    );
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Strength",
        core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
        core_ids::PAINTER_BRUSH_STRENGTH_CHIP,
        brush.strength,
    );

    // 5. Accumulate (checkbox — caps the stroke at Strength when off).
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_ACCUMULATE,
        "Accumulate",
        brush.accumulate,
    );

    // ── Section 6: Randomize Color (collapsible, collapsed by default; activates on amount > 0) ──
    y = paint_randomize_section(ctx, theme, x, content_w, y, brush);

    // ── Section 7: Shape — the dab silhouette. Hosts the Falloff (the procedural default tip) + its
    //    curve preview + a source picker; once a Shape image is assigned the Falloff goes inactive
    //    (replaced by the image + its preview). Sits ABOVE Grain (Enio 2026-06-25). ──
    y = crate::paint_shape::paint_shape_section(ctx, theme, x, content_w, y, brush);

    // ── Section 8: Grain — the texture inside the silhouette (was "Texture", Enio 2026-06-25) ──
    y = crate::paint_texture::paint_texture_section(ctx, theme, x, content_w, y, brush, false);

    // ── Section 9: Stroke · Section 10: Tiling (last section, collapsed by default) ──
    y = crate::paint_stroke::paint_stroke_section(ctx, theme, x, content_w, y, brush);
    y = crate::paint_stroke::paint_tiling_section(ctx, theme, x, content_w, y, brush);

    // ── Eraser checkbox (standalone, very bottom) ──
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_ERASER,
        "Eraser",
        brush.eraser,
    );
    y
}

/// Drain the Brush-properties dropdown popovers (Blend, Falloff, Stroke Method, Jitter Unit) — one
/// at a time, on TOP of the body. Painted by the caller AFTER the body clip is popped, so an open
/// dropdown is never clipped to the scrolling viewport. The chip rects were stashed during
/// [`paint_brush_body`] (already at their scrolled positions).
pub(crate) fn paint_brush_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_blend_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_BLEND,
            brush_blend_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_falloff_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_FALLOFF,
            falloff_options(),
            chip_rect,
            cur,
        );
    }
    // Shape-section source picker (None / Image).
    if let Some((chip_rect, cur)) = state::take_pending_brush_shape_kind_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_KIND,
            crate::paint_shape::shape_kind_options(),
            chip_rect,
            cur,
        );
    }
    // Stroke-section dropdowns (Method + Jitter Unit) — drained last so they sit
    // on top of every body row, same as the Blend/Falloff chips above.
    crate::paint_stroke::paint_stroke_popovers(ctx, theme);
    // Texture-section dropdowns (Kind picker + Mapping).
    crate::paint_texture::paint_texture_popovers(ctx, theme);
}

/// Args for [`paint_param_row`] (grouped to dodge the too-many-arguments lint).
/// `pub(crate)` so the Stroke section ([`crate::paint_stroke`]) reuses the same row.
pub(crate) struct ParamRow<'a, 'b> {
    pub(crate) ctx: &'a mut PaintCtx<'b>,
    pub(crate) theme: ph2d_tokens::Theme,
    pub(crate) x: f32,
    pub(crate) content_w: f32,
    pub(crate) y: f32,
    pub(crate) label: &'a str,
    pub(crate) id: ph2d_a11y::NodeId,
    pub(crate) value: f32,
    pub(crate) readout: &'a str,
}

/// Paint one "label · slider · readout" brush param row. Returns the next `y`.
pub(crate) fn paint_param_row(r: ParamRow) -> f32 {
    let ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: label_txt,
        id,
        value,
        readout,
    } = r;
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

/// When the shared Blender picker targets the brush swatch, the hero loop mirrors its live value into
/// `widget_color(PAINTER_COLOR_THUMB)`. Forward that colour to the tool (as `"r,g,b"`) when it differs
/// from the brush's current colour, so the picker drives the brush live.
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
) -> f32 {
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
    y + ROW_H_PX + Spacing::Sm.px()
}

/// The shared scrollable dropdown-popover renderer (moved to its own module for the LOC cap).
/// Re-exported so the many `crate::paint_brush::paint_dropdown_popover` callers keep resolving.
pub(crate) use crate::dropdown_popover::paint_dropdown_popover;

/// A left-aligned, vertically-centred row label in a `ROW_H_PX` cell.
pub(crate) fn label(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    x: f32,
    y: f32,
    font: f32,
) {
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

/// Paint a "label + dropdown chip" row. Returns `(next_y, Some(chip_rect))` when
/// the chip is open (the caller stashes the rect into the matching pending slot).
/// `pub(crate)` so the Stroke section reuses it for Method + Jitter Unit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
) -> (f32, Option<Rect>) {
    let gap = Spacing::Sm.px();
    label(ctx, theme, label_txt, x, y, TypeToken::Sm.px());
    let chip_w = (content_w - LABEL_W - gap).max(0.0);
    let rect = Rect::new(x + LABEL_W + gap, y, chip_w, ROW_H_PX);
    let open = paint_dropdown_chip(ctx, theme, id, cur_value, cur_label, rect);
    (y + ROW_H_PX + Spacing::Sm.px(), open.then_some(rect))
}

/// Paint a dropdown chip (registered as a `Dropdown` for the generic open/close
/// dispatch). Returns whether it is open. Shared by the Blend + Falloff chips.
pub(crate) fn paint_dropdown_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
) -> bool {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_value as usize),
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
        cur_label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    open
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

/// The falloff presets as `Dropdown` options (value = wire discriminant, label =
/// Blender's preset name).
fn falloff_options() -> Vec<DropdownOption<u8>> {
    (0..MAX_FALLOFF)
        .map(|p| {
            DropdownOption::new(
                painter_brush_falloff_option_id(p),
                p,
                Falloff::from_u8(p).name(),
            )
        })
        .collect()
}

/// Encode a straight-RGB colour in `[0, 1]` (native space) to 8-bit for display.
fn encode_rgb(c: [f32; 3]) -> [u8; 3] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8, // LITERAL-PX-OK: sRGB 8-bit normalize
    ]
}

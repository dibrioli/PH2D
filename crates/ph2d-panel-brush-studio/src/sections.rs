//! Brush Studio section + row painters — split out of `paint.rs` to keep each
//! file under the panel-file LOC cap. `paint_sections` is the entry the
//! orchestrator (`paint::paint`) calls; everything else is module-private.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{
    Button, ButtonState, Checkbox, CheckboxState, CheckboxValue, SectionHeader, paint_button,
    paint_checkbox, paint_section_header, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_painter::BrushStudioSnapshot;

const LABEL_W: f32 = 88.0; // LITERAL-PX-OK: studio slider label column width
const CHIP_W: f32 = 56.0; // LITERAL-PX-OK: studio slider value-chip column width

/// Paint the five sections in order, separated by dividers. Returns the final
/// `y` (content bottom). Split out of `paint` to keep it under the panel-fn LOC cap.
pub(crate) fn paint_sections(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    body_paint_top: f32,
    snapshot: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    let mut y = body_paint_top;
    y = paint_stroke_section(ctx, x, w, y, snapshot, theme);
    y = paint_section_separator(ctx.scene, theme, x, w, y);
    y = paint_shape_section(ctx, x, w, y, snapshot, theme);
    y = paint_section_separator(ctx.scene, theme, x, w, y);
    y = paint_rendering_section(ctx, x, w, y, snapshot, theme);
    y = paint_section_separator(ctx.scene, theme, x, w, y);
    y = paint_color_dynamics_section(ctx, x, w, y, snapshot, theme);
    y = paint_section_separator(ctx.scene, theme, x, w, y);
    y = paint_dynamics_section(ctx, x, w, y, snapshot, theme);
    y
}

fn paint_stroke_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    let (hy, collapsed) = section_header(ctx, ids::SEC_STROKE, "Stroke Path", x, w, y, theme);
    y = hy;
    if collapsed {
        return y;
    }
    // All Stroke Path rows are 0..1 percent sliders → a data table + loop (keeps the
    // section under the LOC cap). "Jitter" = Procreate POSITIONAL jitter (perp.
    // offset); "Spacing Jit" = gap variation; "Taper" = live start taper (D5);
    // "Falloff" = ink depletion; Streamline/Stabilize/Motion = input smoothing
    // (Motion Filt/Expr = One-Euro, ADR-0077 D10).
    let rows = [
        ("Spacing", s.spacing, ids::SPACING_SLIDER, ids::SPACING_CHIP),
        (
            "Jitter",
            s.jitter_lateral,
            ids::JITTER_LATERAL_SLIDER,
            ids::JITTER_LATERAL_CHIP,
        ),
        (
            "Spacing Jit",
            s.spacing_jitter,
            ids::SPACING_JITTER_SLIDER,
            ids::SPACING_JITTER_CHIP,
        ),
        ("Falloff", s.falloff, ids::FALLOFF_SLIDER, ids::FALLOFF_CHIP),
        ("Taper", s.taper_length, ids::TAPER_SLIDER, ids::TAPER_CHIP),
        (
            "Streamline",
            s.streamline_amount,
            ids::STREAMLINE_SLIDER,
            ids::STREAMLINE_CHIP,
        ),
        (
            "Stabilize",
            s.stabilization,
            ids::STABILIZATION_SLIDER,
            ids::STABILIZATION_CHIP,
        ),
        (
            "Motion Filt",
            s.motion_filtering_amount,
            ids::MOTION_FILTER_SLIDER,
            ids::MOTION_FILTER_CHIP,
        ),
        (
            "Motion Expr",
            s.motion_filtering_expression,
            ids::MOTION_EXPR_SLIDER,
            ids::MOTION_EXPR_CHIP,
        ),
    ];
    for (label, val, sld, chip) in rows {
        y = pct_row(ctx, x, w, y, label, val, sld, chip, theme);
    }
    y
}

fn paint_shape_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    use crate::populate::count_to_slider01;
    let (hy, collapsed) = section_header(ctx, ids::SEC_SHAPE, "Shape", x, w, y, theme);
    y = hy;
    if collapsed {
        return y;
    }
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Scatter",
        s.shape_scatter,
        ids::SHAPE_SCATTER_SLIDER,
        ids::SHAPE_SCATTER_CHIP,
        theme,
    );
    y = mapped_row(
        ctx,
        x,
        w,
        y,
        "Count",
        count_to_slider01(s.shape_count),
        s.shape_count as f64,
        &format!("{}", s.shape_count),
        ids::SHAPE_COUNT_SLIDER,
        ids::SHAPE_COUNT_CHIP,
        theme,
    );
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Count Jit",
        s.shape_count_jitter,
        ids::SHAPE_COUNT_JITTER_SLIDER,
        ids::SHAPE_COUNT_JITTER_CHIP,
        theme,
    );
    // NOTE (W5): Roundness not exposed — the stamp is round-symmetric; squashing
    // needs a roundness field in the FROZEN 96B Stamp ABI (Coord+ADR) + render
    // support. Dormant until then.
    for (label, on, id) in [
        (
            "Follow Rotation",
            s.shape_rotation_follow,
            ids::SHAPE_ROTATION_FOLLOW,
        ),
        ("Randomized", s.shape_randomized, ids::SHAPE_RANDOMIZED),
        ("Flip X", s.shape_flip_x, ids::SHAPE_FLIP_X),
        ("Flip Y", s.shape_flip_y, ids::SHAPE_FLIP_Y),
    ] {
        y = checkbox_row(ctx, x, w, y, label, on, id, theme);
    }
    y
}

fn paint_rendering_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    use crate::populate::grain_scale_to_slider01;
    let (hy, collapsed) = section_header(ctx, ids::SEC_RENDERING, "Rendering", x, w, y, theme);
    y = hy;
    if collapsed {
        return y;
    }
    let mode_label = format!("Mode: {}", rendering_mode_label(s.rendering_mode));
    y = cycler_row(ctx, x, w, y, &mode_label, false, ids::RENDERING_MODE, theme);
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Flow",
        s.flow,
        ids::FLOW_SLIDER,
        ids::FLOW_CHIP,
        theme,
    );
    // Rendering toggles. A local closure collapses the identical checkbox rows
    // (ctx/x/w/theme constant) — reads as a table and keeps the section under the
    // LOC cap. Wet/Burnt Edges are LIVE: real pigment-transport (watercolor dark
    // rim at the receding water boundary) / dry-media tooth (charcoal granulated
    // edge), settled on pen-up — see `cpu_render::apply_wash_settle`.
    let toggle = |ctx: &mut PaintCtx, y: f32, label: &str, on: bool, id: NodeId| {
        checkbox_row(ctx, x, w, y, label, on, id, theme)
    };
    y = toggle(ctx, y, "Pigment", s.pigment_enabled, ids::PIGMENT);
    y = toggle(ctx, y, "Accumulate", s.accumulate_enabled, ids::ACCUMULATE);
    y = toggle(ctx, y, "Wet Edges", s.wet_edges, ids::WET_EDGES);
    y = toggle(ctx, y, "Burnt Edges", s.burnt_edges, ids::BURNT_EDGES);
    // Live watercolor fluid diffusion (ADR-0049 / ADR-0077 D11): stamps splat
    // into a per-stroke wet field advected each frame by `Tool::on_tick`.
    y = toggle(ctx, y, "Fluid", s.fluid_enabled, ids::FLUID);
    // Edge Intensity scales the wet/burnt settle; shown only when an edge is on.
    if s.wet_edges || s.burnt_edges {
        y = pct_row(
            ctx,
            x,
            w,
            y,
            "Edge Intensity",
            s.edge_intensity,
            ids::EDGE_INTENSITY_SLIDER,
            ids::EDGE_INTENSITY_CHIP,
            theme,
        );
    }
    let grain_label = format!("Grain: {}", grain_type_label(s.grain_type));
    y = cycler_row(
        ctx,
        x,
        w,
        y,
        &grain_label,
        s.grain_type != 0,
        ids::GRAIN_TYPE,
        theme,
    );
    if s.grain_type != 0 {
        y = mapped_row(
            ctx,
            x,
            w,
            y,
            "Grain Scale",
            grain_scale_to_slider01(s.grain_scale),
            s.grain_scale as f64,
            &format!("{:.2}", s.grain_scale),
            ids::GRAIN_SCALE_SLIDER,
            ids::GRAIN_SCALE_CHIP,
            theme,
        );
        y = pct_row(
            ctx,
            x,
            w,
            y,
            "Grain Depth",
            s.grain_depth,
            ids::GRAIN_DEPTH_SLIDER,
            ids::GRAIN_DEPTH_CHIP,
            theme,
        );
    }
    // Paper tooth — world-space substrate texture (0 = crisp ink, 1 = heavy
    // paper). Always visible; independent of the brush grain source above.
    pct_row(
        ctx,
        x,
        w,
        y,
        "Paper",
        s.paper_grain,
        ids::PAPER_SLIDER,
        ids::PAPER_CHIP,
        theme,
    )
}

fn paint_color_dynamics_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    let (hy, collapsed) = section_header(ctx, ids::SEC_COLOR, "Color Dynamics", x, w, y, theme);
    y = hy;
    if collapsed {
        return y;
    }
    // Per-stamp OKLab jitter (live): each dab varies in colour for natural,
    // hand-mixed strokes. Honored by the scheduler `apply_stamp_color_jitter`.
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Hue",
        s.stamp_hue_jitter,
        ids::HUE_JITTER_SLIDER,
        ids::HUE_JITTER_CHIP,
        theme,
    );
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Saturation",
        s.stamp_saturation_jitter,
        ids::SAT_JITTER_SLIDER,
        ids::SAT_JITTER_CHIP,
        theme,
    );
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Lightness",
        s.stamp_lightness_jitter,
        ids::LIGHT_JITTER_SLIDER,
        ids::LIGHT_JITTER_CHIP,
        theme,
    );
    y = pct_row(
        ctx,
        x,
        w,
        y,
        "Darkness",
        s.stamp_darkness_jitter,
        ids::DARK_JITTER_SLIDER,
        ids::DARK_JITTER_CHIP,
        theme,
    );
    y
}

fn paint_dynamics_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    let (hy, collapsed) = section_header(ctx, ids::SEC_DYNAMICS, "Dynamics", x, w, y, theme);
    y = hy;
    if collapsed {
        return y;
    }
    // Per-stamp size/opacity jitter (0..1, det_random 0xD1/0xD2) as a percent table,
    // then bipolar velocity dynamics (−1..1: stroke speed → size/opacity/spacing,
    // ADR-0077 D10) as a signed-percent table — data loops keep this under the cap.
    for (label, val, sld, chip) in [
        (
            "Size Jit",
            s.jitter_size,
            ids::SIZE_JITTER_SLIDER,
            ids::SIZE_JITTER_CHIP,
        ),
        (
            "Opacity Jit",
            s.jitter_opacity,
            ids::OPACITY_JITTER_SLIDER,
            ids::OPACITY_JITTER_CHIP,
        ),
    ] {
        y = pct_row(ctx, x, w, y, label, val, sld, chip, theme);
    }
    for (label, v, sld, chip) in [
        (
            "Speed->Size",
            s.speed_size,
            ids::SPEED_SIZE_SLIDER,
            ids::SPEED_SIZE_CHIP,
        ),
        (
            "Speed->Opac",
            s.speed_opacity,
            ids::SPEED_OPACITY_SLIDER,
            ids::SPEED_OPACITY_CHIP,
        ),
        (
            "Speed->Space",
            s.speed_spacing,
            ids::SPEED_SPACING_SLIDER,
            ids::SPEED_SPACING_CHIP,
        ),
    ] {
        let disp = format!("{:+.0}%", v * 100.0); // LITERAL-PX-OK: percent display scale (x100), not a px dimension
        y = mapped_row(
            ctx,
            x,
            w,
            y,
            label,
            (v + 1.0) * 0.5,
            (v * 100.0) as f64, // LITERAL-PX-OK: percent display scale (x100), not a px dimension
            &disp,
            sld,
            chip,
            theme,
        );
    }
    y
}

// ── Row helpers ──────────────────────────────────────────────────────────────

/// Paint a collapsible section header + register its hit so a left-click folds
/// it (generic `is_collapsible_section` dispatch — marked in `populate`). Returns
/// `(y after the header, collapsed)`; the caller skips its body when collapsed.
fn section_header(
    ctx: &mut PaintCtx,
    id: NodeId,
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
    (y + header_h + Spacing::Xs.px(), collapsed)
}

/// Percent slider row (0..1 → 0..100%, integer display).
#[allow(clippy::too_many_arguments)]
fn pct_row(
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
fn mapped_row(
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
fn checkbox_row(
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
fn cycler_row(
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

fn rendering_mode_label(mode: u8) -> &'static str {
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

fn grain_type_label(grain: u8) -> &'static str {
    match grain {
        1 => "Simplex",
        2 => "Gabor",
        3 => "Weave",
        4 => "Spray",
        _ => "Off",
    }
}

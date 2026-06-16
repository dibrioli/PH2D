//! Brush Studio section + row painters — split out of `paint.rs` to keep each
//! file under the panel-file LOC cap. `paint_sections` is the entry the
//! orchestrator (`paint::paint`) calls; everything else is module-private.

use crate::ids;
use crate::section_rows::{
    checkbox_row, cycler_row, grain_type_label, mapped_row, pct_row, rendering_mode_label,
    section_header,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_tokens::Theme;
use ph2d_tool_painter::BrushStudioSnapshot;

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
    y = paint_section_separator(ctx.scene, theme, x, w, y);
    y = paint_wet_mix_section(ctx, x, w, y, snapshot, theme);
    y
}

/// **Wet Mix** (mixer-brush reservoir, W7/ADR-0097) — engages on the Blending
/// rendering modes. Five 0..1 sliders: Dilution, Charge, Attack, Pull, Wetness
/// Jitter. Grade/Blur are W7 phase 2 (not yet engine-wired) so they are omitted.
fn paint_wet_mix_section(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    mut y: f32,
    s: &BrushStudioSnapshot,
    theme: Theme,
) -> f32 {
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_WET_MIX,
        ids::RESET_WET_MIX,
        "Wet Mix",
        x,
        w,
        y,
        theme,
    );
    y = hy;
    if collapsed {
        return y;
    }
    // Master enable — the reservoir also auto-engages on Blending rendering modes,
    // but this lets any brush opt in (and makes the sliders visibly "live").
    y = checkbox_row(
        ctx,
        x,
        w,
        y,
        "Wet Mix",
        s.wet_mix_enabled,
        ids::WET_MIX_ENABLED,
        theme,
    );
    for (label, val, sld, chip) in [
        ("Dilution", s.dilution, ids::DILUTION_SLIDER, ids::DILUTION_CHIP),
        ("Charge", s.charge, ids::CHARGE_SLIDER, ids::CHARGE_CHIP),
        ("Attack", s.attack, ids::ATTACK_SLIDER, ids::ATTACK_CHIP),
        ("Pull", s.pull, ids::PULL_SLIDER, ids::PULL_CHIP),
        (
            "Wetness Jit",
            s.wetness_jitter,
            ids::WETNESS_JITTER_SLIDER,
            ids::WETNESS_JITTER_CHIP,
        ),
        ("Grade", s.grade, ids::GRADE_SLIDER, ids::GRADE_CHIP),
        ("Blur", s.blur, ids::BLUR_SLIDER, ids::BLUR_CHIP),
        (
            "Blur Jit",
            s.blur_jitter,
            ids::BLUR_JITTER_SLIDER,
            ids::BLUR_JITTER_CHIP,
        ),
    ] {
        y = pct_row(ctx, x, w, y, label, val, sld, chip, theme);
    }
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
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_STROKE,
        ids::RESET_STROKE,
        "Stroke Path",
        x,
        w,
        y,
        theme,
    );
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
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_SHAPE,
        ids::RESET_SHAPE,
        "Shape",
        x,
        w,
        y,
        theme,
    );
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
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_RENDERING,
        ids::RESET_RENDERING,
        "Rendering",
        x,
        w,
        y,
        theme,
    );
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
    // LOC cap.
    let toggle = |ctx: &mut PaintCtx, y: f32, label: &str, on: bool, id: NodeId| {
        checkbox_row(ctx, x, w, y, label, on, id, theme)
    };
    y = toggle(ctx, y, "Pigment", s.pigment_enabled, ids::PIGMENT);
    y = toggle(ctx, y, "Accumulate", s.accumulate_enabled, ids::ACCUMULATE);
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
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_COLOR,
        ids::RESET_COLOR,
        "Color Dynamics",
        x,
        w,
        y,
        theme,
    );
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
    let (hy, collapsed) = section_header(
        ctx,
        ids::SEC_DYNAMICS,
        ids::RESET_DYNAMICS,
        "Dynamics",
        x,
        w,
        y,
        theme,
    );
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

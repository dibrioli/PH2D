//! Brush Studio paint — chrome + three scrollable sections (Stroke Path /
//! Shape / Rendering), mirroring the Inspector section layout and the
//! sidebar dock chrome.
//!
//! Render canon (mirror sidebar / layers dock pattern):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect from `ctx.layout.painter_sidebar` (shared slot — only one
//!   of sidebar / layers / studio is visible at a time)
//! - Chrome: dark-glass surface + corner dots + title "Brush Studio" + close X
//!   + drag/resize handles (Inspector slot shared canon)
//! - Scrollable body: sliders via `paint_slider_with_chip_layout_adaptive`,
//!   checkboxes via `paint_checkbox`, enum dials as cycling buttons; sections
//!   separated by `paint_section_separator`; scrollbar + content_h publish

use crate::BrushStudioPanel;
use crate::ids;
use crate::state::{self, BrushStudioPanelState, set_last_content_h, set_last_visible_h};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, Checkbox, CheckboxState, CheckboxValue, PAINTER_BRUSH_STUDIO_SCROLLBAR_ID,
    SectionHeader, paint_button, paint_checkbox, paint_section_header,
    paint_slider_with_chip_layout_adaptive, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_painter::BrushStudioSnapshot;

const LABEL_W: f32 = 88.0; // LITERAL-PX-OK: studio slider label column width
const CHIP_W: f32 = 56.0; // LITERAL-PX-OK: studio slider value-chip column width

pub(crate) fn paint(_state: &mut BrushStudioPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(BrushStudioPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_BRUSH_STUDIO_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_sidebar;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_BRUSH_STUDIO_PANEL, rect);

    // Chrome.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }
    let title_size = paint_panel_title(
        rect,
        "Brush Studio",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // Body region (clipped) + scroll.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let scroll_y = ctx
        .host
        .store()
        .panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL)
        .max(0.0);
    let x = rect.x + PANEL_HEAD_PAD;
    let w = rect.w - PANEL_HEAD_PAD * 2.0;
    let body_paint_top = body_top + Spacing::Sm.px() - scroll_y;
    let y = paint_sections(ctx, x, w, body_paint_top, &snapshot, theme);

    let content_h = (y - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Visual scrollbar (self-gates when the content fits) + drag thumb id.
    let scrollbar_active = matches!(
        ctx.host.store().scrollbar_drag(),
        Some(d) if d.panel == core_ids::PAINTER_BRUSH_STUDIO_PANEL
    );
    paint_scrollbar(
        body_rect,
        scroll_y,
        content_h,
        body_h,
        scrollbar_active,
        ctx.scene,
        theme,
    );
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll_y, content_h, body_h);
        ctx.host
            .hit_index_mut()
            .register(PAINTER_BRUSH_STUDIO_SCROLLBAR_ID, thumb);
    }

    // Corner dots last so they sit visually atop any body drift.
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    // Re-register close after the body so scrolled widgets cannot shadow it.
    ctx.host
        .hit_index_mut()
        .register(ids::CLOSE, panel_close_button_rect(rect));

    // Publish scroll bounds + clamp right after paint (next-event correctness).
    let store = ctx.host.store_mut();
    store.set_panel_content_h(core_ids::PAINTER_BRUSH_STUDIO_PANEL, content_h);
    store.set_panel_visible_h(core_ids::PAINTER_BRUSH_STUDIO_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL) > max_scroll {
        store.set_panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL, max_scroll);
    }
}

/// Paint the five sections in order, separated by dividers. Returns the final
/// `y` (content bottom). Split out of `paint` to keep it under the panel-fn LOC cap.
fn paint_sections(
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
    y = pct_row(ctx, x, w, y, "Spacing", s.spacing, ids::SPACING_SLIDER, ids::SPACING_CHIP, theme);
    // "Jitter" = Procreate's Stroke Path Jitter: a POSITIONAL offset perpendicular
    // to the stroke (our `jitter_lateral`) — visible even on a solid stroke
    // (roughens the edge). The gap-variation `spacing_jitter` is "Spacing Jit"
    // below (only visible on a spaced-out brush).
    y = pct_row(ctx, x, w, y, "Jitter", s.jitter_lateral, ids::JITTER_LATERAL_SLIDER, ids::JITTER_LATERAL_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Spacing Jit", s.spacing_jitter, ids::SPACING_JITTER_SLIDER, ids::SPACING_JITTER_CHIP, theme);
    // T1.7 falloff (live): ink-depletion opacity taper along the stroke.
    y = pct_row(ctx, x, w, y, "Falloff", s.falloff, ids::FALLOFF_SLIDER, ids::FALLOFF_CHIP, theme);
    // T1.7 input smoothing (live): streamline = lazy-mouse lag, stabilize =
    // moving-average jitter rejection. Both honored by the scheduler.
    y = pct_row(ctx, x, w, y, "Streamline", s.streamline_amount, ids::STREAMLINE_SLIDER, ids::STREAMLINE_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Stabilize", s.stabilization, ids::STABILIZATION_SLIDER, ids::STABILIZATION_CHIP, theme);
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
    y = pct_row(ctx, x, w, y, "Scatter", s.shape_scatter, ids::SHAPE_SCATTER_SLIDER, ids::SHAPE_SCATTER_CHIP, theme);
    y = mapped_row(ctx, x, w, y, "Count", count_to_slider01(s.shape_count), s.shape_count as f64, &format!("{}", s.shape_count), ids::SHAPE_COUNT_SLIDER, ids::SHAPE_COUNT_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Count Jit", s.shape_count_jitter, ids::SHAPE_COUNT_JITTER_SLIDER, ids::SHAPE_COUNT_JITTER_CHIP, theme);
    // NOTE (W5): Roundness not exposed — the stamp is round-symmetric; squashing
    // needs a roundness field in the FROZEN 96B Stamp ABI (Coord+ADR) + render
    // support. Dormant until then.
    y = checkbox_row(ctx, x, w, y, "Follow Rotation", s.shape_rotation_follow, ids::SHAPE_ROTATION_FOLLOW, theme);
    y = checkbox_row(ctx, x, w, y, "Randomized", s.shape_randomized, ids::SHAPE_RANDOMIZED, theme);
    y = checkbox_row(ctx, x, w, y, "Flip X", s.shape_flip_x, ids::SHAPE_FLIP_X, theme);
    y = checkbox_row(ctx, x, w, y, "Flip Y", s.shape_flip_y, ids::SHAPE_FLIP_Y, theme);
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
    y = pct_row(ctx, x, w, y, "Flow", s.flow, ids::FLOW_SLIDER, ids::FLOW_CHIP, theme);
    y = checkbox_row(ctx, x, w, y, "Pigment", s.pigment_enabled, ids::PIGMENT, theme);
    y = checkbox_row(ctx, x, w, y, "Accumulate", s.accumulate_enabled, ids::ACCUMULATE, theme);
    // NOTE (W5): Wet/Burnt Edges + Alpha Floor are engine-pending. Wet/Burnt need
    // a STROKE-silhouette edge pass (over the coverage mask), NOT the per-dab rim
    // I first tried (that darkened every circle → scalloped/low-res). Alpha Floor
    // needs a new Stamp ABI field (= Coord). Plumbing stays dormant + ready.
    let grain_label = format!("Grain: {}", grain_type_label(s.grain_type));
    y = cycler_row(ctx, x, w, y, &grain_label, s.grain_type != 0, ids::GRAIN_TYPE, theme);
    if s.grain_type != 0 {
        y = mapped_row(ctx, x, w, y, "Grain Scale", grain_scale_to_slider01(s.grain_scale), s.grain_scale as f64, &format!("{:.2}", s.grain_scale), ids::GRAIN_SCALE_SLIDER, ids::GRAIN_SCALE_CHIP, theme);
        y = pct_row(ctx, x, w, y, "Grain Depth", s.grain_depth, ids::GRAIN_DEPTH_SLIDER, ids::GRAIN_DEPTH_CHIP, theme);
    }
    y
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
    y = pct_row(ctx, x, w, y, "Hue", s.stamp_hue_jitter, ids::HUE_JITTER_SLIDER, ids::HUE_JITTER_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Saturation", s.stamp_saturation_jitter, ids::SAT_JITTER_SLIDER, ids::SAT_JITTER_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Lightness", s.stamp_lightness_jitter, ids::LIGHT_JITTER_SLIDER, ids::LIGHT_JITTER_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Darkness", s.stamp_darkness_jitter, ids::DARK_JITTER_SLIDER, ids::DARK_JITTER_CHIP, theme);
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
    // Per-stamp size + opacity jitter (live): each dab varies for a dry-brush /
    // textured feel. Honored by the scheduler (det_random axes 0xD1 / 0xD2).
    y = pct_row(ctx, x, w, y, "Size Jit", s.jitter_size, ids::SIZE_JITTER_SLIDER, ids::SIZE_JITTER_CHIP, theme);
    y = pct_row(ctx, x, w, y, "Opacity Jit", s.jitter_opacity, ids::OPACITY_JITTER_SLIDER, ids::OPACITY_JITTER_CHIP, theme);
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
    let display = format!("{:.0}%", value01 * 100.0);
    mapped_row(ctx, x, w, y, label, value01, (value01 * 100.0) as f64, &display, slider_id, chip_id, theme)
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

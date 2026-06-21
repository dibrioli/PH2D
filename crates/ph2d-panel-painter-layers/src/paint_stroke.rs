//! The Painter dock's **Stroke** sub-section (clean-room port of Blender's 2D-paint "Stroke"
//! panel): Stroke Method, Spacing, Adjust-Strength, Jitter + Jitter Unit, Dash Ratio/Length,
//! and Input Samples. (Smoothing is automatic — a Catmull-Rom spline in the engine — so there is
//! no Stabilize toggle.)
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this
//! module only paints them off the published [`BrushSettings`] snapshot and reuses the row/chip
//! helpers from [`crate::paint_brush`]. The slider tracks are `0..1`; the tool maps each onto its
//! real range (the `BRUSH_*_MAX` constants are the single source). Edits forward over the frozen
//! `PanelEvent` channel (drained in [`crate::event`]).

use crate::paint_brush::{ParamRow, paint_dropdown_row, paint_param_row, paint_toggle_row};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_jitter_unit_option_id, painter_brush_stroke_method_option_id,
};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::{
    BRUSH_COUNT_SLIDER_MAX, BRUSH_JITTER_ABS_MAX_PX, BrushSettings, JitterUnit, StrokeMethod,
};

/// Paint the Stroke section starting at `y`, returning the next `y`. The two dropdowns (Method,
/// Jitter Unit) stash their open rects for the deferred [`paint_stroke_popovers`] pass.
pub(crate) fn paint_stroke_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let mut y = section_header(ctx, theme, x, content_w, y, "Stroke");

    // Each param is gated by the method that actually consumes it — Blender hides
    // the irrelevant rows rather than showing dead controls. A row that is not
    // painted registers no hit rect this frame, so it can't be clicked; its stored
    // value persists in the WidgetStore, so switching methods round-trips it (DIRETIVA
    // §2: no silent no-ops). The predicate set is locked by `stroke_panel_visibility_
    // matches_blender` in `ph2d-painter-brush`.
    let method = StrokeMethod::from_u8(brush.stroke_method);

    // ── Method dropdown (always) ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Method",
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        brush.stroke_method,
        stroke_method_name(brush.stroke_method),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_stroke_method_dd(Some((r, brush.stroke_method)));
    }

    // ── Spacing (% of diameter) + Adjust Strength — only the spacing-driven methods ──
    if method.uses_spacing() {
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Spacing",
            id: core_ids::PAINTER_BRUSH_SPACING,
            value: brush.spacing,
            readout: &format!("{:.0}%", brush.spacing * 100.0),
        });
        y = paint_toggle_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_SPACE_ATTEN,
            "Adjust Strength",
            brush.space_attenuation,
        );
    }

    // ── Jitter (unit-aware track + readout) + Jitter Unit dropdown — all but Drag Dot/Anchored ──
    if method.allows_jitter() {
        let view = brush.jitter_unit == JitterUnit::View.to_u8();
        let (jval, jread) = if view {
            (
                brush.jitter_absolute_px / BRUSH_JITTER_ABS_MAX_PX,
                format!("{:.0}px", brush.jitter_absolute_px),
            )
        } else {
            (brush.jitter, format!("{:.2}", brush.jitter))
        };
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Jitter",
            id: core_ids::PAINTER_BRUSH_JITTER,
            value: jval,
            readout: &jread,
        });
        let (ny, open) = paint_dropdown_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Unit",
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
            brush.jitter_unit,
            jitter_unit_name(brush.jitter_unit),
        );
        y = ny;
        if let Some(r) = open {
            state::set_pending_brush_jitter_unit_dd(Some((r, brush.jitter_unit)));
        }
    }

    // ── Dash ratio + length — only the spacing-driven methods ──
    if method.uses_dash() {
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Dash",
            id: core_ids::PAINTER_BRUSH_DASH_RATIO,
            value: brush.dash_ratio,
            readout: &format!("{:.2}", brush.dash_ratio),
        });
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Length",
            id: core_ids::PAINTER_BRUSH_DASH_LENGTH,
            value: count_to_norm(brush.dash_samples),
            readout: &brush.dash_samples.to_string(),
        });
    }

    // ── Input samples (always) ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Samples",
        id: core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        value: count_to_norm(brush.input_samples),
        readout: &brush.input_samples.to_string(),
    });
    y
}

/// Deferred paint of the Stroke section's open dropdown popovers (Method + Jitter Unit), drained
/// at the very end of the Brush body so they sit above every row.
pub(crate) fn paint_stroke_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_stroke_method_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            stroke_method_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_jitter_unit_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
            jitter_unit_options(),
            chip_rect,
            cur,
        );
    }
}

/// A faint, left-aligned section divider label in a `ROW_H_PX` cell.
fn section_header(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    text: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        content_w,
        resolve(ColorToken::Text3, theme),
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Map a count (`1..=BRUSH_COUNT_SLIDER_MAX`) onto the slider's `0..1` track. Inverse of the
/// tool's `count_from_norm`.
fn count_to_norm(n: u32) -> f32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    (n.saturating_sub(1) as f32 / span).clamp(0.0, 1.0)
}

/// Display name for a stroke-method wire discriminant.
fn stroke_method_name(m: u8) -> &'static str {
    match StrokeMethod::from_u8(m) {
        StrokeMethod::Dots => "Dots",
        StrokeMethod::Airbrush => "Airbrush",
        StrokeMethod::Anchored => "Anchored",
        StrokeMethod::Space => "Space",
        StrokeMethod::DragDot => "Drag Dot",
        StrokeMethod::Line => "Line",
        StrokeMethod::Curve => "Curve",
    }
}

/// Display name for a jitter-unit wire discriminant.
fn jitter_unit_name(u: u8) -> &'static str {
    match JitterUnit::from_u8(u) {
        JitterUnit::Brush => "Brush",
        JitterUnit::View => "View",
    }
}

/// The 7 stroke methods as dropdown options, in Blender's menu order
/// (Dots, Drag Dot, Space, Airbrush, Anchored, Line, Curve).
fn stroke_method_options() -> Vec<DropdownOption<u8>> {
    [0u8, 4, 3, 1, 2, 5, 6]
        .into_iter()
        .map(|m| {
            DropdownOption::new(
                painter_brush_stroke_method_option_id(m),
                m,
                stroke_method_name(m),
            )
        })
        .collect()
}

/// The two jitter units as dropdown options.
fn jitter_unit_options() -> Vec<DropdownOption<u8>> {
    [0u8, 1]
        .into_iter()
        .map(|u| {
            DropdownOption::new(
                painter_brush_jitter_unit_option_id(u),
                u,
                jitter_unit_name(u),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    //! Paint-level proof that the per-method gate actually drops the irrelevant
    //! rows. The existing `tests/seam.rs` checks bypass hit-testing (they inject a
    //! `WidgetEvent` for a fixed id straight into `apply_event`), so they prove the
    //! wire works *when shown* but cannot prove a row is *hidden*. Here we drive the
    //! real `paint_stroke_section` and read the per-frame `HitIndex`: a row that is
    //! not painted registers no hit rect, so no real pointer click can reach it
    //! (the DIRETIVA §2 "no silent no-op" guarantee). The value still lives in the
    //! WidgetStore, so switching methods round-trips it.

    use super::*;
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
    use ph2d_editor_core::screens::HeroLayout;
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_ui_testkit::MockPanelHost;
    use ph2d_vector::VectorScene;

    /// Paint the Stroke section for `method` (all other fields at the Blender
    /// defaults) and return every widget id that registered a hit rect this frame.
    fn painted_hit_ids(method: StrokeMethod) -> Vec<NodeId> {
        let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        // Tall viewport so no row is clipped out for a reason other than the gate.
        let viewport = Rect::new(0.0, 0.0, 360.0, 4000.0);
        let layout = HeroLayout::for_viewport(viewport);
        let brush = BrushSettings {
            stroke_method: method.to_u8(),
            ..crate::paint_brush::FALLBACK_BRUSH
        };
        {
            let mut ctx = PaintCtx {
                host: &mut host,
                layout: &layout,
                viewport,
                scene: &mut scene,
                text_system: &mut text,
            };
            paint_stroke_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush);
        }
        host.hit_index_mut()
            .iter_registrations()
            .map(|(id, _)| id)
            .collect()
    }

    /// Dots: per-event method — Spacing/Dash are no-ops, so they must be hidden;
    /// Jitter and Input Samples stay (Method is always shown).
    #[test]
    fn dots_hides_spacing_and_dash_keeps_jitter_and_samples() {
        let ids = painted_hit_ids(StrokeMethod::Dots);
        for hidden in [
            core_ids::PAINTER_BRUSH_SPACING,
            core_ids::PAINTER_BRUSH_SPACE_ATTEN,
            core_ids::PAINTER_BRUSH_DASH_RATIO,
            core_ids::PAINTER_BRUSH_DASH_LENGTH,
        ] {
            assert!(
                !ids.contains(&hidden),
                "Dots painted a hit rect for {hidden:?} — Spacing/Dash must be hidden \
                 (silent no-op). painted = {ids:?}"
            );
        }
        for shown in [
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            core_ids::PAINTER_BRUSH_JITTER,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
            core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        ] {
            assert!(
                ids.contains(&shown),
                "Dots dropped a row it should show ({shown:?}). painted = {ids:?}"
            );
        }
    }

    /// Space (the default): the only positive control — proves the gate is not
    /// hiding everything; every spacing-driven row is present.
    #[test]
    fn space_shows_spacing_dash_and_the_rest() {
        let ids = painted_hit_ids(StrokeMethod::Space);
        for shown in [
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            core_ids::PAINTER_BRUSH_SPACING,
            core_ids::PAINTER_BRUSH_SPACE_ATTEN,
            core_ids::PAINTER_BRUSH_JITTER,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
            core_ids::PAINTER_BRUSH_DASH_RATIO,
            core_ids::PAINTER_BRUSH_DASH_LENGTH,
            core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        ] {
            assert!(
                ids.contains(&shown),
                "Space dropped {shown:?} — the spacing-driven default must show every \
                 Stroke row. painted = {ids:?}"
            );
        }
    }

    /// Drag Dot: the most-restricted method (no jitter, no smooth, no spacing) —
    /// only Method + Input Samples survive. This is the UI-honesty half of the
    /// Enio-reported "Drag Dot wrong" gap.
    #[test]
    fn dragdot_shows_only_method_and_samples() {
        let ids = painted_hit_ids(StrokeMethod::DragDot);
        for hidden in [
            core_ids::PAINTER_BRUSH_SPACING,
            core_ids::PAINTER_BRUSH_SPACE_ATTEN,
            core_ids::PAINTER_BRUSH_DASH_RATIO,
            core_ids::PAINTER_BRUSH_DASH_LENGTH,
            core_ids::PAINTER_BRUSH_JITTER,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
        ] {
            assert!(
                !ids.contains(&hidden),
                "Drag Dot painted a hit rect for {hidden:?} — it forces pressure 1, no \
                 jitter. painted = {ids:?}"
            );
        }
        for shown in [
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        ] {
            assert!(
                ids.contains(&shown),
                "Drag Dot dropped {shown:?} (always shown). painted = {ids:?}"
            );
        }
    }
}

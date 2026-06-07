//! Brush Studio `populate` — pre-registers every widget id in the `WidgetStore`
//! at host boot (once via `Panel::populate`).
//!
//! Sliders store the normalized `0..1` value (sliders always emit 0..1 on
//! drag); the editable chip shows the display unit via an affine link
//! (`display = value*scale + offset`). Most params are 0..1 → shown as percent
//! (scale 100). `shape_count` → 1..16 integer (scale 15, offset 1); `grain_scale`
//! → 0.1..4.0 (scale 3.9, offset 0.1). Seeds come from
//! [`BrushStudioSnapshot::default`] (Round Hard); the host overwrites every
//! frame from the live snapshot. Pattern mirrors the sidebar / BgRemoval.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, TextInputState,
};
use ph2d_tool_painter::BrushStudioSnapshot;

/// Inverse map: `shape_count` (1..16) → normalized slider `0..1`.
pub(crate) fn count_to_slider01(count: u32) -> f32 {
    ((count.clamp(1, 16) as f32 - 1.0) / 15.0).clamp(0.0, 1.0)
}

/// Inverse map: `grain_scale` (0.1..4.0) → normalized slider `0..1`.
pub(crate) fn grain_scale_to_slider01(scale: f32) -> f32 {
    ((scale.clamp(0.1, 4.0) - 0.1) / 3.9).clamp(0.0, 1.0)
}

pub fn populate(store: &mut WidgetStore) {
    // Close (X) button.
    store.register(
        ids::CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );

    // Section headers — collapsible (UI canon: a left-click on a header id
    // marked here flips `store.is_collapsed` via the generic dispatch, same as
    // the Inspector sections). The panel paint reads `is_collapsed` to fold the
    // body. Mirror of `pre_populate::mark_collapsible_section`.
    store.mark_collapsible_section(ids::SEC_STROKE);
    store.mark_collapsible_section(ids::SEC_SHAPE);
    store.mark_collapsible_section(ids::SEC_RENDERING);
    store.mark_collapsible_section(ids::SEC_COLOR);
    store.mark_collapsible_section(ids::SEC_DYNAMICS);

    let s = BrushStudioSnapshot::default();

    // ── Stroke Path — percent sliders (0..1 → 0..100%) ──────────────────────
    pct(store, ids::SPACING_SLIDER, ids::SPACING_CHIP, s.spacing);
    pct(
        store,
        ids::SPACING_JITTER_SLIDER,
        ids::SPACING_JITTER_CHIP,
        s.spacing_jitter,
    );
    pct(
        store,
        ids::JITTER_LATERAL_SLIDER,
        ids::JITTER_LATERAL_CHIP,
        s.jitter_lateral,
    );
    pct(store, ids::FALLOFF_SLIDER, ids::FALLOFF_CHIP, s.falloff);
    pct(store, ids::TAPER_SLIDER, ids::TAPER_CHIP, s.taper_length);
    pct(
        store,
        ids::STREAMLINE_SLIDER,
        ids::STREAMLINE_CHIP,
        s.streamline_amount,
    );
    pct(
        store,
        ids::STABILIZATION_SLIDER,
        ids::STABILIZATION_CHIP,
        s.stabilization,
    );
    // One-Euro motion filtering (ADR-0077 D10) — unipolar 0..1.
    pct(
        store,
        ids::MOTION_FILTER_SLIDER,
        ids::MOTION_FILTER_CHIP,
        s.motion_filtering_amount,
    );
    pct(
        store,
        ids::MOTION_EXPR_SLIDER,
        ids::MOTION_EXPR_CHIP,
        s.motion_filtering_expression,
    );

    // ── Shape — percent sliders + count + checkboxes ────────────────────────
    pct(
        store,
        ids::SHAPE_SCATTER_SLIDER,
        ids::SHAPE_SCATTER_CHIP,
        s.shape_scatter,
    );
    // shape_count: integer 1..16.
    slider_chip(
        store,
        ids::SHAPE_COUNT_SLIDER,
        ids::SHAPE_COUNT_CHIP,
        count_to_slider01(s.shape_count),
        15.0,
        1.0,
        true,
    );
    pct(
        store,
        ids::SHAPE_COUNT_JITTER_SLIDER,
        ids::SHAPE_COUNT_JITTER_CHIP,
        s.shape_count_jitter,
    );
    checkbox(store, ids::SHAPE_ROTATION_FOLLOW, s.shape_rotation_follow);
    checkbox(store, ids::SHAPE_RANDOMIZED, s.shape_randomized);
    checkbox(store, ids::SHAPE_FLIP_X, s.shape_flip_x);
    checkbox(store, ids::SHAPE_FLIP_Y, s.shape_flip_y);

    // ── Rendering — sliders + checkboxes + cyclers ──────────────────────────
    pct(store, ids::FLOW_SLIDER, ids::FLOW_CHIP, s.flow);
    // grain_scale: 0.1..4.0.
    slider_chip(
        store,
        ids::GRAIN_SCALE_SLIDER,
        ids::GRAIN_SCALE_CHIP,
        grain_scale_to_slider01(s.grain_scale),
        3.9,
        0.1,
        false,
    );
    pct(
        store,
        ids::GRAIN_DEPTH_SLIDER,
        ids::GRAIN_DEPTH_CHIP,
        s.grain_depth,
    );
    checkbox(store, ids::PIGMENT, s.pigment_enabled);
    checkbox(store, ids::ACCUMULATE, s.accumulate_enabled);
    checkbox(store, ids::WET_EDGES, s.wet_edges);
    checkbox(store, ids::BURNT_EDGES, s.burnt_edges);
    checkbox(store, ids::FLUID, s.fluid_enabled);
    // Register the interactive slider state UNCONDITIONALLY (same pattern as the
    // grain sliders) — `paint` shows it only when an edge mode is on, but the
    // `SliderState` must exist in the store regardless so a drag is accepted the
    // moment it appears (a conditional register here leaves it un-draggable).
    pct(
        store,
        ids::EDGE_INTENSITY_SLIDER,
        ids::EDGE_INTENSITY_CHIP,
        s.edge_intensity,
    );
    // Paper tooth — always visible, so register straight (no conditional).
    pct(store, ids::PAPER_SLIDER, ids::PAPER_CHIP, s.paper_grain);
    button(store, ids::GRAIN_TYPE);
    button(store, ids::RENDERING_MODE);

    // ── Color Dynamics — per-stamp OKLab jitter (engine-wired) ──────────────
    pct(
        store,
        ids::HUE_JITTER_SLIDER,
        ids::HUE_JITTER_CHIP,
        s.stamp_hue_jitter,
    );
    pct(
        store,
        ids::SAT_JITTER_SLIDER,
        ids::SAT_JITTER_CHIP,
        s.stamp_saturation_jitter,
    );
    pct(
        store,
        ids::LIGHT_JITTER_SLIDER,
        ids::LIGHT_JITTER_CHIP,
        s.stamp_lightness_jitter,
    );
    pct(
        store,
        ids::DARK_JITTER_SLIDER,
        ids::DARK_JITTER_CHIP,
        s.stamp_darkness_jitter,
    );

    // ── Dynamics — per-stamp size/opacity jitter (engine-wired T1.7) ────────
    pct(
        store,
        ids::SIZE_JITTER_SLIDER,
        ids::SIZE_JITTER_CHIP,
        s.jitter_size,
    );
    pct(
        store,
        ids::OPACITY_JITTER_SLIDER,
        ids::OPACITY_JITTER_CHIP,
        s.jitter_opacity,
    );
    // Velocity dynamics (ADR-0077 D10) — bipolar −1..1 (centre = off).
    bipolar(
        store,
        ids::SPEED_SIZE_SLIDER,
        ids::SPEED_SIZE_CHIP,
        s.speed_size,
    );
    bipolar(
        store,
        ids::SPEED_OPACITY_SLIDER,
        ids::SPEED_OPACITY_CHIP,
        s.speed_opacity,
    );
    bipolar(
        store,
        ids::SPEED_SPACING_SLIDER,
        ids::SPEED_SPACING_CHIP,
        s.speed_spacing,
    );
}

/// Register a percent slider+chip pair (0..1 → 0..100%, integer display).
fn pct(store: &mut WidgetStore, slider_id: NodeId, chip_id: NodeId, value01: f32) {
    slider_chip(store, slider_id, chip_id, value01, 100.0, 0.0, true);
}

/// Register a bipolar slider+chip pair (−1..1 → −100..+100%, integer display).
/// The slider stores `(value + 1)/2 ∈ [0,1]`; the chip shows the signed percent.
fn bipolar(store: &mut WidgetStore, slider_id: NodeId, chip_id: NodeId, value: f32) {
    slider_chip(
        store,
        slider_id,
        chip_id,
        (value + 1.0) * 0.5,
        200.0,
        -100.0,
        true,
    );
}

/// Register a slider + editable chip with an affine display map
/// (`display = value*scale + offset`) and link them.
fn slider_chip(
    store: &mut WidgetStore,
    slider_id: NodeId,
    chip_id: NodeId,
    value01: f32,
    scale: f32,
    offset: f32,
    integer: bool,
) {
    let display = (value01 * scale + offset) as f64;
    store.register(
        slider_id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: value01,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip_id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: display,
            buffer: format!("{display:.3}"),
            caret: 0,
            last_committed: display,
            selection_anchor: None,
        },
    );
    if integer {
        store.link_slider_number_mapped_integer(slider_id, chip_id, scale, offset);
    } else {
        store.link_slider_number_mapped(slider_id, chip_id, scale, offset);
    }
}

fn checkbox(store: &mut WidgetStore, id: NodeId, checked: bool) {
    store.register(
        id,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: if checked {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            },
        },
    );
}

fn button(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_tool_painter::{BrushParam, PainterTool, PainterUiEdit};

    #[test]
    fn inverse_maps_endpoints_and_clamp() {
        assert_eq!(count_to_slider01(1), 0.0);
        assert_eq!(count_to_slider01(16), 1.0);
        assert_eq!(count_to_slider01(0), 0.0); // clamps up to 1
        assert_eq!(count_to_slider01(99), 1.0); // clamps down to 16
        assert!((grain_scale_to_slider01(0.1) - 0.0).abs() < 1e-6);
        assert!((grain_scale_to_slider01(4.0) - 1.0).abs() < 1e-6);
        assert!((grain_scale_to_slider01(-5.0) - 0.0).abs() < 1e-6);
        assert!((grain_scale_to_slider01(99.0) - 1.0).abs() < 1e-6);
    }

    /// End-to-end: the panel's inverse map → the tool's `SetBrushParam` forward
    /// map → `brush_studio_snapshot` must recover the value the panel displays.
    /// This is the real wiring the slider drives, not an isolated unit.
    #[test]
    fn shape_count_round_trips_through_tool() {
        for count in [1u32, 3, 8, 12, 16] {
            let mut tool = PainterTool::default();
            tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
                BrushParam::ShapeCount,
                count_to_slider01(count),
            ));
            assert_eq!(
                tool.brush_studio_snapshot().shape_count,
                count,
                "shape_count {count} must round-trip panel↔tool"
            );
        }
    }

    #[test]
    fn grain_scale_round_trips_through_tool() {
        for scale in [0.1f32, 1.0, 2.0, 3.5, 4.0] {
            let mut tool = PainterTool::default();
            tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
                BrushParam::GrainScale,
                grain_scale_to_slider01(scale),
            ));
            let got = tool.brush_studio_snapshot().grain_scale;
            assert!(
                (got - scale).abs() < 1e-3,
                "grain_scale {scale} must round-trip panel↔tool (got {got})"
            );
        }
    }

    #[test]
    fn smoothing_params_round_trip_through_tool() {
        // Streamline / Stabilize are percent sliders (value01 == param), so the
        // panel→tool→snapshot loop must echo the value back verbatim. The
        // scheduler then honors it (proven in ph2d-painter-brush tests).
        let mut tool = PainterTool::default();
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::StreamlineAmount,
            0.5,
        ));
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::Stabilization,
            0.75,
        ));
        let s = tool.brush_studio_snapshot();
        assert!((s.streamline_amount - 0.5).abs() < 1e-6);
        assert!((s.stabilization - 0.75).abs() < 1e-6);
    }

    #[test]
    fn color_dynamics_jitter_round_trips_through_tool() {
        // Color Dynamics is engine-wired (the scheduler applies per-stamp OKLab
        // jitter); the panel→tool→snapshot loop must echo each value back.
        let mut tool = PainterTool::default();
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(BrushParam::HueJitter, 0.4));
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::SaturationJitter,
            0.6,
        ));
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::LightnessJitter,
            0.3,
        ));
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::DarknessJitter,
            0.8,
        ));
        let s = tool.brush_studio_snapshot();
        assert!((s.stamp_hue_jitter - 0.4).abs() < 1e-6);
        assert!((s.stamp_saturation_jitter - 0.6).abs() < 1e-6);
        assert!((s.stamp_lightness_jitter - 0.3).abs() < 1e-6);
        assert!((s.stamp_darkness_jitter - 0.8).abs() < 1e-6);
    }

    #[test]
    fn dynamics_jitter_round_trips_through_tool() {
        // Dynamics size/opacity jitter is engine-wired (scheduler det_random
        // 0xD1/0xD2); the panel→tool→snapshot loop must echo the values.
        let mut tool = PainterTool::default();
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(BrushParam::SizeJitter, 0.45));
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(
            BrushParam::OpacityJitter,
            0.65,
        ));
        let s = tool.brush_studio_snapshot();
        assert!((s.jitter_size - 0.45).abs() < 1e-6);
        assert!((s.jitter_opacity - 0.65).abs() < 1e-6);
    }

    #[test]
    fn bool_and_open_close_wiring() {
        let mut tool = PainterTool::default();
        // shape_flip_x defaults false; set it true via the generic param (a
        // live, panel-exposed bool — the scheduler honors the flip flag).
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(BrushParam::ShapeFlipX, 1.0));
        assert!(tool.brush_studio_snapshot().shape_flip_x);
        tool.apply_ui_edit(PainterUiEdit::SetBrushParam(BrushParam::ShapeFlipX, 0.0));
        assert!(!tool.brush_studio_snapshot().shape_flip_x);
        // Open/close flag drives dock visibility (read by the bridge).
        assert!(!tool.show_brush_studio());
        tool.apply_ui_edit(PainterUiEdit::OpenBrushStudio);
        assert!(tool.show_brush_studio());
        tool.close_brush_studio();
        assert!(!tool.show_brush_studio());
    }
}

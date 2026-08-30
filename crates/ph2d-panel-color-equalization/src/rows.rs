//! Color Equalization panel — the **slider-row table**.
//!
//! Split out of `paint_sections.rs` (Wave 10 panel LOC cap: 600/file)
//! when the `show` gate landed and pushed that file past its frozen
//! 660-LOC allowance. It is the honest split, not a raised cap: this is
//! a TABLE, not paint code — thirteen rows of data, one predicate each,
//! and the painter that walks it stays next door.
//!
//! ⚠️ **Every `show` here is a call into
//! `ph2d_tool_color_equalization::params::stage`** — the same expression
//! the pipeline uses to decide whether to run the stage, never a
//! re-statement of it. A stage that stops running takes its knobs off
//! the panel in the same edit.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_tool_color_equalization::params::{
    ColorEqualizationUiSnapshot, brightness_to_slider, clip_limit_to_slider, contrast_to_slider,
    exposure_to_slider, lut_intensity_to_slider, lut_mix_to_slider, saturation_to_slider,
    sharpen_amount_to_slider, sharpen_radius_to_slider, temperature_to_slider, tile_grid_to_slider,
    tint_to_slider, vibrance_to_slider,
};

pub(crate) struct SliderRow {
    pub(crate) label: &'static str,
    pub(crate) slider_id: NodeId,
    pub(crate) chip_id: NodeId,
    pub(crate) snap_track: f32,
    pub(crate) snap_chip: f64,
    pub(crate) chip_display: String,
    /// Whether the stage that CONSUMES this row is running.
    ///
    /// ⚠️ Every predicate here is a call into
    /// `ph2d_tool_color_equalization::params::stage` — the same
    /// expression the pipeline uses to decide whether to run the stage,
    /// never a re-statement of it. `always` is spelled out for the rows
    /// whose stage has no off switch, so a new row cannot get a free
    /// pass by omitting the field — the compiler refuses an incomplete
    /// struct literal, which is the only enforcement that never
    /// bit-rots.
    pub(crate) show: fn(&ColorEqualizationUiSnapshot) -> bool,
}

/// The rows whose consuming stage is unconditional (Phase 1 tonal +
/// sharpen + LUT intensity all run whenever the bake runs).
pub(crate) fn always(_: &ColorEqualizationUiSnapshot) -> bool {
    true
}

pub(crate) fn build_slider_rows(snapshot: &ColorEqualizationUiSnapshot) -> [SliderRow; 13] {
    [
        SliderRow {
            label: "Clip",
            slider_id: ids::CEQ_CLIP_LIMIT,
            chip_id: ids::CEQ_CLIP_LIMIT_NUM,
            snap_track: clip_limit_to_slider(snapshot.clip_limit),
            snap_chip: snapshot.clip_limit as f64,
            chip_display: format!("{:.2}", snapshot.clip_limit),
            // Clip IS the CLAHE on-switch — hiding it would hide the
            // only control that can turn the stage on.
            show: always,
        },
        SliderRow {
            label: "Tile Grid",
            slider_id: ids::CEQ_TILE_GRID,
            chip_id: ids::CEQ_TILE_GRID_NUM,
            snap_track: tile_grid_to_slider(snapshot.tile_grid_size),
            snap_chip: snapshot.tile_grid_size as f64,
            chip_display: snapshot.tile_grid_size.to_string(),
            // ⭐ `tile_grid_size` is handed to `clahe(..)` and to
            // nothing else. At the default `clip_limit == CLIP_LIMIT_MIN`
            // the pipeline never calls it, so this row moved a number
            // that reached no consumer — in the state the panel is born
            // in.
            show: ColorEqualizationUiSnapshot::clahe_stage_runs,
        },
        SliderRow {
            label: "Exposure",
            slider_id: ids::CEQ_EXPOSURE,
            chip_id: ids::CEQ_EXPOSURE_NUM,
            snap_track: exposure_to_slider(snapshot.exposure),
            snap_chip: snapshot.exposure as f64,
            chip_display: format!("{:+.2} EV", snapshot.exposure),
            show: always,
        },
        SliderRow {
            label: "Temperature",
            slider_id: ids::CEQ_TEMPERATURE,
            chip_id: ids::CEQ_TEMPERATURE_NUM,
            snap_track: temperature_to_slider(snapshot.temperature),
            snap_chip: snapshot.temperature as f64,
            chip_display: format!("{:+.2}", snapshot.temperature),
            show: always,
        },
        SliderRow {
            label: "Tint",
            slider_id: ids::CEQ_TINT,
            chip_id: ids::CEQ_TINT_NUM,
            snap_track: tint_to_slider(snapshot.tint),
            snap_chip: snapshot.tint as f64,
            chip_display: format!("{:+.2}", snapshot.tint),
            show: always,
        },
        SliderRow {
            label: "Brightness",
            slider_id: ids::CEQ_BRIGHTNESS,
            chip_id: ids::CEQ_BRIGHTNESS_NUM,
            snap_track: brightness_to_slider(snapshot.brightness),
            snap_chip: snapshot.brightness as f64,
            chip_display: format!("{:+.2}", snapshot.brightness),
            show: always,
        },
        SliderRow {
            label: "Contrast",
            slider_id: ids::CEQ_CONTRAST,
            chip_id: ids::CEQ_CONTRAST_NUM,
            snap_track: contrast_to_slider(snapshot.contrast),
            snap_chip: snapshot.contrast as f64,
            chip_display: format!("{:.2}", snapshot.contrast),
            show: always,
        },
        SliderRow {
            label: "Vibrance",
            slider_id: ids::CEQ_VIBRANCE,
            chip_id: ids::CEQ_VIBRANCE_NUM,
            snap_track: vibrance_to_slider(snapshot.vibrance),
            snap_chip: snapshot.vibrance as f64,
            chip_display: format!("{:+.2}", snapshot.vibrance),
            show: always,
        },
        SliderRow {
            label: "Saturation",
            slider_id: ids::CEQ_SATURATION,
            chip_id: ids::CEQ_SATURATION_NUM,
            snap_track: saturation_to_slider(snapshot.saturation),
            snap_chip: snapshot.saturation as f64,
            chip_display: format!("{:+.2}", snapshot.saturation),
            show: always,
        },
        SliderRow {
            label: "Sharpen",
            slider_id: ids::CEQ_SHARPEN_AMOUNT,
            chip_id: ids::CEQ_SHARPEN_AMOUNT_NUM,
            snap_track: sharpen_amount_to_slider(snapshot.sharpen_amount),
            snap_chip: snapshot.sharpen_amount as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_amount),
            show: always,
        },
        SliderRow {
            label: "Radius",
            slider_id: ids::CEQ_SHARPEN_RADIUS,
            chip_id: ids::CEQ_SHARPEN_RADIUS_NUM,
            snap_track: sharpen_radius_to_slider(snapshot.sharpen_radius),
            snap_chip: snapshot.sharpen_radius as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_radius),
            show: always,
        },
        SliderRow {
            label: "LUT Intensity",
            slider_id: ids::CEQ_LUT_INTENSITY,
            chip_id: ids::CEQ_LUT_INTENSITY_NUM,
            snap_track: lut_intensity_to_slider(snapshot.lut_intensity),
            snap_chip: snapshot.lut_intensity as f64,
            chip_display: format!("{:.2}", snapshot.lut_intensity),
            // Intensity is the LUT stage's own on-switch (`0` bypasses
            // it), so it stays reachable at all times.
            show: always,
        },
        SliderRow {
            label: "LUT Mix",
            slider_id: ids::CEQ_LUT_MIX,
            chip_id: ids::CEQ_LUT_MIX_NUM,
            snap_track: lut_mix_to_slider(snapshot.lut_mix),
            snap_chip: snapshot.lut_mix as f64,
            chip_display: format!("{:.2}", snapshot.lut_mix),
            // ⭐ `lut_mix` reaches `blend_luts` on ONE arm of the LUT
            // match — the `(Some, Some)` one. Both preset slots default
            // to `None`, and even with one slot filled the pipeline
            // applies that cube directly and discards the mix. The wire
            // was complete and the consumer projected the value out.
            show: ColorEqualizationUiSnapshot::lut_blend_stage_runs,
        },
    ]
}

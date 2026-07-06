//! Brush-section dropdown **option-id decoders**: map a clicked popover option's [`NodeId`] back to
//! its wire `u8`. Split from `event.rs` to keep that file under the workspace LOC cap. Each iterates
//! its option factory over the full discriminant range — the range MUST include the last value
//! (`0..=N`), or the final option's click is silently dropped (the historic Ellipse = 7 bug). The
//! round-trip is locked by `event/tests.rs`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_tool_painter::{
    MAX_BRUSH_BLEND_MODES, MAX_FALLOFF, RampAlphaMode, RampColorMode, RampInterp, TextureKind,
    TextureMapping,
};

/// Decode a top-of-panel **Preset** popover option id → its preset idx (`0` = Digital, `1` = Watercolor).
pub(super) fn decode_brush_preset_option(id: NodeId) -> Option<u8> {
    (0..core_ids::PAINTER_BRUSH_PRESET_COUNT)
        .find(|&i| core_ids::painter_brush_preset_option_id(i) == id)
}

/// Decode a Watercolor **Paper** kind popover option id → its `TextureKind` wire u8.
pub(super) fn decode_paper_kind_option(id: NodeId) -> Option<u8> {
    (0..TextureKind::COUNT).find(|&k| core_ids::painter_paper_kind_option_id(k) == id)
}

/// Decode a Watercolor **Paper** mapping popover option id → its `TextureMapping` wire u8.
pub(super) fn decode_paper_mapping_option(id: NodeId) -> Option<u8> {
    (0..TextureMapping::COUNT).find(|&m| core_ids::painter_paper_mapping_option_id(m) == id)
}

/// Decode a brush blend-mode popover option id → its mode `u8` (fixed; iterate the 24 stable ids).
pub(super) fn decode_brush_blend_option(id: NodeId) -> Option<u8> {
    (0..MAX_BRUSH_BLEND_MODES).find(|&m| core_ids::painter_brush_blend_option_id(m) == id)
}

/// Decode a brush Falloff popover option id → its preset `u8` (fixed; iterate the stable preset ids).
pub(super) fn decode_brush_falloff_option(id: NodeId) -> Option<u8> {
    (0..MAX_FALLOFF).find(|&p| core_ids::painter_brush_falloff_option_id(p) == id)
}

/// Decode a Stroke-Method popover option id → its `StrokeMethod` wire `u8` (the 10 methods: the 7
/// Blender ones `0..=6` + the PH2D `Ellipse` (7) + `Polygon` (8) + `FreeHand` (9) extensions).
pub(super) fn decode_stroke_method_option(id: NodeId) -> Option<u8> {
    (0..10).find(|&m| core_ids::painter_brush_stroke_method_option_id(m) == id)
}

/// Decode a Jitter-Unit popover option id → its wire `u8` (`0` = Brush, `1` = View).
pub(super) fn decode_jitter_unit_option(id: NodeId) -> Option<u8> {
    (0..2).find(|&u| core_ids::painter_brush_jitter_unit_option_id(u) == id)
}

/// Decode a texture-Kind popover option id → its `TextureKind` wire `u8` (`0..=4`, includes None).
pub(super) fn decode_texture_kind_option(id: NodeId) -> Option<u8> {
    (0..TextureKind::COUNT).find(|&k| core_ids::painter_brush_texture_kind_option_id(k) == id)
}

/// Decode a Shape-source popover option id → its `TextureKind` wire `u8`. Only `None` (0) and `Image`
/// (5) are offered; iterate the full range so both stable ids are matched.
pub(super) fn decode_shape_kind_option(id: NodeId) -> Option<u8> {
    (0..TextureKind::COUNT).find(|&k| core_ids::painter_shape_kind_option_id(k) == id)
}

/// Decode a Shape-ramp Interpolation popover option id → its `RampInterp` wire `u8`.
pub(super) fn decode_shape_ramp_interp_option(id: NodeId) -> Option<u8> {
    (0..RampInterp::COUNT).find(|&i| core_ids::painter_shape_ramp_interp_option_id(i) == id)
}

/// Decode a Shape-ramp colour-Mode popover option id → its `RampColorMode` wire `u8`.
pub(super) fn decode_shape_ramp_mode_option(id: NodeId) -> Option<u8> {
    (0..RampColorMode::COUNT).find(|&m| core_ids::painter_shape_ramp_mode_option_id(m) == id)
}

/// Decode a Shape-ramp Alpha-action popover option id → its `RampAlphaMode` wire `u8`.
pub(super) fn decode_shape_ramp_alpha_option(id: NodeId) -> Option<u8> {
    (0..RampAlphaMode::COUNT).find(|&m| core_ids::painter_shape_ramp_alpha_option_id(m) == id)
}

/// Decode a texture-Mapping popover option id → its `TextureMapping` wire `u8` (`0..=2`).
pub(super) fn decode_texture_mapping_option(id: NodeId) -> Option<u8> {
    (0..TextureMapping::COUNT).find(|&m| core_ids::painter_brush_texture_mapping_option_id(m) == id)
}

/// Decode a Color Ramp Mode popover option id → its `RampColorMode` wire `u8` (`0..=2`).
pub(super) fn decode_texture_ramp_mode_option(id: NodeId) -> Option<u8> {
    (0..RampColorMode::COUNT)
        .find(|&m| core_ids::painter_brush_texture_ramp_mode_option_id(m) == id)
}

/// Decode a Color Ramp Interpolation popover option id → its `RampInterp` wire `u8` (`0..=4`).
pub(super) fn decode_texture_ramp_interp_option(id: NodeId) -> Option<u8> {
    (0..RampInterp::COUNT).find(|&i| core_ids::painter_brush_texture_ramp_interp_option_id(i) == id)
}

/// Decode a Color Ramp Alpha-action popover option id → its `RampAlphaMode` wire `u8` (`0..=2`).
pub(super) fn decode_texture_ramp_alpha_option(id: NodeId) -> Option<u8> {
    (0..RampAlphaMode::COUNT)
        .find(|&m| core_ids::painter_brush_texture_ramp_alpha_option_id(m) == id)
}

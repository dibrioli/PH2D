//! Regression tests for the Brush-section dropdown-option decoders. The Stroke Method decoder once
//! used `0..7`, which silently dropped the PH2D shape extensions (Ellipse = 7, then Polygon = 8) —
//! clicking them in the dropdown did nothing. These lock the full 9-method round-trip + distinctness.

use super::decode::{
    decode_stroke_method_option, decode_texture_kind_option, decode_texture_mapping_option,
    decode_texture_ramp_alpha_option,
};
use super::*;
use ph2d_editor_core::ids::{
    painter_brush_stroke_method_option_id, painter_brush_texture_kind_option_id,
    painter_brush_texture_mapping_option_id, painter_brush_texture_ramp_alpha_option_id,
};
use ph2d_tool_painter::{RampAlphaMode, TextureKind, TextureMapping};

#[test]
fn every_stroke_method_option_id_round_trips() {
    // All 9 methods (Dots..Curve = 0..=6, Ellipse = 7, Polygon = 8) must decode back to themselves.
    for m in 0u8..9 {
        let id = painter_brush_stroke_method_option_id(m);
        assert_eq!(
            decode_stroke_method_option(id),
            Some(m),
            "option id for method {m} did not decode back (Ellipse = 7 / Polygon = 8 regression)"
        );
    }
    // A foreign id decodes to None (not a false match).
    assert_eq!(
        decode_stroke_method_option(core_ids::PAINTER_BRUSH_STROKE_METHOD),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn stroke_method_option_ids_are_distinct() {
    let ids: Vec<_> = (0u8..9)
        .map(painter_brush_stroke_method_option_id)
        .collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "option ids {i} and {j} collide");
        }
    }
}

#[test]
fn every_texture_kind_option_id_round_trips() {
    // All kinds (`0..=COUNT-1`, includes None) must decode back — the range MUST include the last.
    for k in 0u8..TextureKind::COUNT {
        let id = painter_brush_texture_kind_option_id(k);
        assert_eq!(
            decode_texture_kind_option(id),
            Some(k),
            "texture kind {k} did not decode back (last-value-dropped regression)"
        );
    }
    assert_eq!(
        decode_texture_kind_option(core_ids::PAINTER_BRUSH_TEXTURE_KIND),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn every_texture_mapping_option_id_round_trips() {
    for m in 0u8..TextureMapping::COUNT {
        let id = painter_brush_texture_mapping_option_id(m);
        assert_eq!(
            decode_texture_mapping_option(id),
            Some(m),
            "texture mapping {m} did not decode back"
        );
    }
    assert_eq!(
        decode_texture_mapping_option(core_ids::PAINTER_BRUSH_TEXTURE_MAPPING),
        None,
        "the chip id is not an option id"
    );
}

#[test]
fn every_ramp_alpha_option_id_round_trips() {
    // All 3 alpha actions (Off / Strength / Sprite) must decode back — the range MUST include the last.
    for m in 0u8..RampAlphaMode::COUNT {
        let id = painter_brush_texture_ramp_alpha_option_id(m);
        assert_eq!(
            decode_texture_ramp_alpha_option(id),
            Some(m),
            "ramp alpha action {m} did not decode back (last-value-dropped regression)"
        );
    }
    assert_eq!(
        decode_texture_ramp_alpha_option(core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE),
        None,
        "the chip id is not an option id"
    );
}

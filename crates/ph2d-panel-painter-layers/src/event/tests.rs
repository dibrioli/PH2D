//! Regression tests for the Brush-section dropdown-option decoders. The Stroke Method decoder once
//! used `0..7`, which silently dropped the PH2D shape extensions (Circle = 7, then Polygon = 8) —
//! clicking them in the dropdown did nothing. These lock the full 9-method round-trip + distinctness.

use super::*;
use ph2d_editor_core::ids::painter_brush_stroke_method_option_id;

#[test]
fn every_stroke_method_option_id_round_trips() {
    // All 9 methods (Dots..Curve = 0..=6, Circle = 7, Polygon = 8) must decode back to themselves.
    for m in 0u8..9 {
        let id = painter_brush_stroke_method_option_id(m);
        assert_eq!(
            decode_stroke_method_option(id),
            Some(m),
            "option id for method {m} did not decode back (Circle = 7 / Polygon = 8 regression)"
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

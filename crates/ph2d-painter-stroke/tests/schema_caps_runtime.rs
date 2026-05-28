//! Integration test: cap em runtime que o gate textual de
//! `ph2d-painter-contracts` não cobre — `StrokeRecord.points.len() ≤ u16::MAX`
//! (ADR-0046 §2.2 audit B-3).
//!
//! O gate em `architecture_painter_contract_surface.rs` checa caps **textual**
//! (struct field count, enum variant count). Mas `points.len()` é runtime —
//! caller pode tentar empurrar > 65535 samples em um stroke. Esta sanidade
//! local valida o BOUNDARY exato.

use ph2d_color::OklchColor;
use ph2d_painter_brush::BrushHandle;
use ph2d_painter_stroke::{
    MAX_SAMPLES_PER_STROKE, RawPointerSample, StrokeRecord, device::LayerId,
};

#[test]
fn max_samples_per_stroke_is_u16_max() {
    assert_eq!(MAX_SAMPLES_PER_STROKE, u16::MAX as usize);
    assert_eq!(MAX_SAMPLES_PER_STROKE, 65_535);
}

#[test]
fn stroke_record_detects_points_at_cap_boundary() {
    let mut rec = StrokeRecord::new(
        0,
        BrushHandle::default(),
        [0u8; 32],
        LayerId::default(),
        OklchColor::default(),
    );
    assert!(!rec.points_at_cap());
    rec.points
        .resize_with(MAX_SAMPLES_PER_STROKE - 1, RawPointerSample::default);
    assert!(!rec.points_at_cap(), "one shy of cap is not at cap");
    rec.points.push(RawPointerSample::default());
    assert!(rec.points_at_cap(), "exactly at cap counts as at cap");
}

#[test]
fn stroke_record_postcard_roundtrip_at_cap() {
    let mut rec = StrokeRecord::new(
        42,
        BrushHandle::default(),
        [0u8; 32],
        LayerId(7),
        OklchColor::opaque(0.5, 0.2, 180.0),
    );
    // 1024 samples — ordem de magnitude real (estudo Procreate típico: 256-512).
    rec.points.resize_with(1024, RawPointerSample::default);
    let bytes = postcard::to_allocvec(&rec).expect("serialize stroke");
    let rec2: StrokeRecord = postcard::from_bytes(&bytes).expect("deserialize stroke");
    assert_eq!(rec, rec2);
    assert_eq!(rec2.points.len(), 1024);
}

//! Integration test: `.ph2d-painter` save → load roundtrip byte-identical.
//!
//! Behavior gate `painter_persistence_roundtrip_v1` (ADR-0046 §2.12) —
//! postcard determinístico DEVE produzir output byte-equal pra mesma input,
//! E `deserialize ∘ serialize` DEVE ser identidade. Sem isso, content-
//! addressing via blake3 quebra (mesmo canon → checksum diferente).

use ph2d_painter_brush::{Brush, BrushHandle};
use ph2d_painter_stroke::{
    CanvasInfo, ColorProfile, PAINT_PROJECT_MAGIC, PaintProject, SCHEMA_VERSION, StrokeHistory,
    StrokeRecord, ToolMode, device::PersistLayerId,
};

fn sample_project() -> PaintProject {
    let mut p = PaintProject::new(CanvasInfo {
        width: 2048,
        height: 2048,
        color_profile: ColorProfile::Srgb,
        ppm: 11.811,
    });
    let brush = Brush::default();
    let hash = brush.params_blake3();
    let handle = BrushHandle::default();
    // 3 strokes pra ter algum content significativo.
    for seq in 0..3 {
        let mut rec = StrokeRecord::new(
            seq,
            handle,
            hash,
            PersistLayerId(0),
            ph2d_color::OklchColor::opaque(0.7, 0.12, 30.0 * seq as f32),
        );
        rec.tool_mode = if seq % 2 == 0 {
            ToolMode::Paint
        } else {
            ToolMode::Smudge
        };
        // Golden-ratio LCG splat — wrapping_mul porque a constant × seq=2 já
        // estoura u64 em debug builds (constante ≈ 1.14e19; u64::MAX ≈ 1.84e19).
        rec.rng_seed = seq.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        p.history.push(rec);
    }
    p.brush_snapshots.push((hash, brush));
    p.created_at = 1_700_000_000_000;
    p.modified_at = 1_700_000_000_500;
    p.recompute_checksum();
    p
}

#[test]
fn paint_project_roundtrip_byte_identical() {
    let p = sample_project();
    let bytes_a = postcard::to_allocvec(&p).expect("serialize");
    let p2: PaintProject = postcard::from_bytes(&bytes_a).expect("deserialize");
    let bytes_b = postcard::to_allocvec(&p2).expect("re-serialize");
    assert_eq!(
        bytes_a, bytes_b,
        "postcard must be deterministic — bytes(serialize(deserialize(x))) == bytes(x)"
    );
    assert_eq!(p, p2, "deserialize ∘ serialize must be identity");
}

#[test]
fn paint_project_checksum_persists_across_roundtrip() {
    let p = sample_project();
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let p2: PaintProject = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(p.checksum, p2.checksum);
    assert!(p2.verify_checksum(), "loaded project checksum must verify");
}

#[test]
fn paint_project_magic_and_version_stable() {
    let p = sample_project();
    assert_eq!(p.magic, PAINT_PROJECT_MAGIC);
    assert_eq!(p.version, SCHEMA_VERSION);
    assert_eq!(
        SCHEMA_VERSION, 3,
        "v3 = aquarela/fluido removidos do RenderingParams (ADR-0096), quebra dura \
         de save (postcard posicional); pré-v3 não carrega. Bump exige migration \
         helper sequencial (ou quebra dura documentada) + atualizar os pins de versão"
    );
}

#[test]
fn stroke_history_full_roundtrip_with_records() {
    let p = sample_project();
    assert_eq!(p.history.len(), 3);
    let bytes = postcard::to_allocvec(&p.history).expect("serialize history");
    let h2: StrokeHistory = postcard::from_bytes(&bytes).expect("deserialize history");
    assert_eq!(h2.len(), 3);
    let seqs: Vec<_> = h2.iter().map(|r| r.seq).collect();
    assert_eq!(seqs, vec![0, 1, 2]);
}

//! Integration tests: checksum semântica + cross-instance determinism.
//!
//! Audit T1.8 L1-F9/L2-F5 — content-addressing via blake3 só serve se for
//! byte-deterministic. Estes tests valida que dois projects logicamente
//! idênticos produzem MESMO checksum, e que mutações detectam tamper.

use ph2d_color::OklchColor;
use ph2d_painter_brush::{Brush, BrushHandle};
use ph2d_painter_stroke::{
    CanvasInfo, LoadError, PaintProject, SCHEMA_VERSION, StrokeRecord, ToolMode, device::PersistLayerId,
    load,
};
use uuid::Uuid;

/// Constrói um project **totalmente deterministic** — UUIDs fixados via
/// `Uuid::from_u128(seed)` em vez de `Uuid::new_v4()` random. Sem isto,
/// `make_project(seed)` duas vezes seguidas produz checksums diferentes
/// (UUID v4 = random) e o test de cross-instance determinism falha sem
/// que haja bug real.
///
/// ADR-0046 §2.2: `StrokeRecord::uuid` é parte do schema — content-
/// addressing depende dele. Caller que QUER reproducibilidade (det-mode
/// replay, golden tests, MCP audit) DEVE usar `Uuid::from_u128` ou similar.
fn make_project(seed: u64) -> PaintProject {
    let mut p = PaintProject::new(CanvasInfo::default());
    let brush = Brush::default();
    let hash = brush.params_blake3();
    let handle = BrushHandle::default();
    for i in 0..3 {
        let mut rec = StrokeRecord::new(
            i,
            handle,
            hash,
            PersistLayerId(0),
            OklchColor::opaque(0.5, 0.1, (i as f32) * 30.0),
        );
        // PIN UUID determinístico — caso contrário Uuid::new_v4() randomiza
        // cada chamada e cross-instance equality NÃO se sustenta.
        rec.uuid = Uuid::from_u128((seed as u128) * 1000 + (i as u128));
        rec.rng_seed = seed.wrapping_add(i);
        rec.tool_mode = ToolMode::Paint;
        p.history.push(rec);
    }
    p.brush_snapshots.push((hash, brush));
    p.created_at = 1_700_000_000_000;
    p.modified_at = 1_700_000_000_500;
    p.recompute_checksum();
    p
}

#[test]
fn two_logically_identical_projects_have_same_checksum() {
    // Audit T1.8 L1-F9: content-addressing via blake3 EXIGE que dois
    // projects construídos identicamente produzam o mesmo checksum.
    // Sem este test, regressão em postcard determinismo passa silenciosa.
    let p1 = make_project(42);
    let p2 = make_project(42);
    assert_eq!(
        p1.checksum, p2.checksum,
        "Dois PaintProjects logicamente idênticos DEVEM ter checksum igual \
         (content-addressing depende disso)"
    );
}

#[test]
fn different_inputs_produce_different_checksums() {
    let p1 = make_project(1);
    let p2 = make_project(2);
    assert_ne!(p1.checksum, p2.checksum, "RNG seed diff → checksum diff");
}

#[test]
fn checksum_changes_under_mutation_until_recomputed() {
    let mut p = make_project(0);
    let original = p.checksum;
    p.modified_at = 9_999_999;
    // SEM recompute, verify falha (checksum stale).
    assert!(!p.verify_checksum(), "tamper detected");
    p.recompute_checksum();
    assert!(p.verify_checksum(), "após recompute, verifica");
    assert_ne!(original, p.checksum, "checksum mudou após mutação");
}

#[test]
fn load_roundtrip_via_postcard() {
    let p = make_project(7);
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let p2 = load(&bytes).expect("load");
    assert_eq!(p, p2);
}

#[test]
fn load_rejects_wrong_magic() {
    let mut p = make_project(0);
    p.magic = *b"NOT-A-CANVAS";
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let err = load(&bytes).expect_err("should fail");
    assert_eq!(err, LoadError::WrongMagic);
}

#[test]
fn load_rejects_future_version() {
    let mut p = make_project(0);
    p.version = SCHEMA_VERSION + 1;
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let err = load(&bytes).expect_err("should fail");
    assert!(matches!(err, LoadError::FutureVersion { .. }));
}

#[test]
fn load_rejects_corrupted_checksum() {
    let mut p = make_project(0);
    p.modified_at = 12345; // mutate WITHOUT recompute
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let err = load(&bytes).expect_err("should fail");
    assert_eq!(err, LoadError::IntegrityMismatch);
}

#[test]
fn load_rejects_garbage_bytes() {
    // Audit T1.8 L3-G2 + L4-H5: LoadError::Deserialization preserva
    // causa interna do postcard (era LoadError::Postcard antes).
    let err = load(&[0u8; 4]).expect_err("garbage bytes should fail");
    assert!(matches!(err, LoadError::Deserialization(_)));
}

#[test]
fn load_rejects_version_zero() {
    // Audit T1.8 L3-G2: LoadError::UnsupportedVersion(0) reachable mas sem test.
    let mut p = make_project(0);
    p.version = 0;
    p.recompute_checksum();
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    let err = load(&bytes).expect_err("version 0 should fail");
    assert_eq!(err, LoadError::UnsupportedVersion(0));
}

#[test]
fn redo_in_ring_can_evict_unrelated_stroke() {
    // Audit T1.8 L3-G5: documenta semântica anti-UX (undo→push→redo
    // pode perder stroke do meio). Sentinel test pra ratificar
    // comportamento atual; UI layer T1.9+ deve separar undo-stack.
    use ph2d_painter_stroke::StrokeHistory;
    let mut h = StrokeHistory::ring(3);
    // Estado inicial: 3 strokes [A, B, C]. `make_project`'s first stroke
    // is always seq=0, so assign MONOTONIC seqs for the gated `push`
    // calls (ADR-0046 §2.2, audit T1.9 S-10). UUIDs stay distinct, so the
    // eviction-identity assertions below are unaffected.
    let mut a = make_project(1).history.iter().next().unwrap().clone();
    let mut b = make_project(2).history.iter().next().unwrap().clone();
    let mut c = make_project(3).history.iter().next().unwrap().clone();
    a.seq = 0;
    b.seq = 1;
    c.seq = 2;
    h.push(a.clone());
    h.push(b.clone());
    h.push(c.clone());
    // undo C → [A, B]
    let popped = h.undo().unwrap();
    assert_eq!(popped.uuid, c.uuid);
    // Push D (caller faz algo entre undo e redo). seq=3 keeps push monotonic;
    // the subsequent redo(C) re-inserts seq=2 < 3 — legitimately
    // non-monotonic, which `redo` allows (it bypasses the push gate).
    let mut d = make_project(4).history.iter().next().unwrap().clone();
    d.seq = 3;
    let evicted_by_d = h.push(d.clone());
    assert!(evicted_by_d.is_none(), "ring tinha espaço — D só preenche");
    // Estado agora: [A, B, D]. redo(C) → evicta A.
    let evicted_by_redo = h.redo(popped);
    assert!(
        evicted_by_redo.is_some(),
        "redo em Ring cheio evicta stroke MAIS ANTIGO (A) — caller UX deveria logar warning"
    );
    assert_eq!(evicted_by_redo.unwrap().uuid, a.uuid);
}

#[test]
fn iter_ring_yields_chronological_order() {
    // Audit T1.8 L3-G6: cobre Ring iter (default test só cobre Full).
    use ph2d_painter_stroke::StrokeHistory;
    let mut h = StrokeHistory::ring(5);
    for i in 0..5u64 {
        let mut rec = StrokeRecord::new(
            i,
            ph2d_painter_brush::BrushHandle::default(),
            [0u8; 32],
            ph2d_painter_stroke::device::PersistLayerId::default(),
            OklchColor::default(),
        );
        rec.uuid = Uuid::from_u128(i as u128);
        h.push(rec);
    }
    let seqs: Vec<_> = h.iter().map(|r| r.seq).collect();
    assert_eq!(
        seqs,
        vec![0, 1, 2, 3, 4],
        "Ring iter deve front → back chronological"
    );
}

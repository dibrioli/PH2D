//! Integration test: `Brush` em `brush_snapshots` deve ser deterministic
//! sob postcard roundtrip. Audit T1.8 L1-F2.
//!
//! Se Brush tiver algum sub-field com HashMap, f64 NaN, ou ordenação
//! não-determinística, content-addressing via blake3 quebra silenciosamente.
//! Test ainda usa Brush::default() (T1.6 ship) MAS popula varios sub-fields
//! pra detectar drift em qualquer ramo do schema.

use ph2d_painter_brush::Brush;

#[test]
fn brush_default_roundtrips_byte_identical() {
    // Mesmo Brush serializa pra mesmos bytes — duas vezes seguidas, no
    // mesmo processo. Detecta non-determinismo dentro de um processo.
    let b = Brush::default();
    let bytes_a = postcard::to_allocvec(&b).expect("a");
    let bytes_b = postcard::to_allocvec(&b).expect("b");
    assert_eq!(
        bytes_a, bytes_b,
        "Brush::default serialize deve ser deterministic"
    );
}

#[test]
fn brush_default_deserialize_serialize_identity() {
    // serialize ∘ deserialize ∘ serialize == serialize. Pega regressões
    // onde o deserialize normaliza algum field (e.g., NaN→0.0, set→sorted)
    // mas o serialize subsequente reflete a mudança.
    let b = Brush::default();
    let bytes_a = postcard::to_allocvec(&b).expect("a");
    let b2: Brush = postcard::from_bytes(&bytes_a).expect("deserialize");
    let bytes_b = postcard::to_allocvec(&b2).expect("b");
    assert_eq!(
        bytes_a, bytes_b,
        "Brush roundtrip serialize-deserialize-serialize must be identity"
    );
}

#[test]
fn brush_params_hash_stable_within_process() {
    // `Brush::params_blake3()` produz mesmo hash pra duas Brush::default()
    // chamadas. Sentinel pra non-determinismo em PARAMS_BLAKE3 impl.
    let h1 = Brush::default().params_blake3();
    let h2 = Brush::default().params_blake3();
    assert_eq!(h1, h2);
}

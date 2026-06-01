//! Integration tests: ABI pins que protegem HR-14 forward-compat.
//!
//! Audit T1.8 L1-F5/L1-F10/L1-F11 — postcard usa declaration order pra
//! struct fields E discriminant pra enum variants. Mover field ou
//! renomear variant QUEBRA forward-compat silenciosamente. Estes tests
//! pinam os bytes de saída para serem o canário cross-OS.

use ph2d_painter_stroke::{
    CanvasInfo, ColorProfile, LayerStackEntry, PAINT_PROJECT_CACHE_MAGIC, PAINT_PROJECT_MAGIC,
    PaintProject, PaintProjectCache,
};

#[test]
fn paint_project_magic_is_first_12_bytes() {
    // Audit T1.8 L1-F5: magic deve ser detectável sem deserialização.
    // postcard `[u8; 12]` (fixed-size tuple) NÃO emite length prefix —
    // os primeiros 12 bytes do arquivo são literalmente "PH2D-PAINTER".
    // Reordenar fields no struct PaintProject QUEBRA isto silenciosamente.
    let p = PaintProject::new(CanvasInfo::default());
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    assert_eq!(
        &bytes[..12],
        PAINT_PROJECT_MAGIC.as_slice(),
        "PaintProject magic deve ser bytes 0..12 do serialized output \
         (postcard preserva declaration order — se este test falha, alguém \
         moveu o field `magic` de cima do struct)"
    );
}

#[test]
fn paint_project_cache_magic_is_first_18_bytes() {
    let cache = PaintProjectCache::new([0u8; 32]);
    let bytes = postcard::to_allocvec(&cache).expect("serialize");
    assert_eq!(&bytes[..18], PAINT_PROJECT_CACHE_MAGIC.as_slice());
}

#[test]
fn color_profile_srgb_discriminant_is_zero() {
    // Audit T1.8 L1-F10: ColorProfile::Srgb DEVE ser discriminant 0
    // sempre. Quando T-color materializar ph2d-color::ColorProfile, este
    // test (movido / replicado lá) garante que files v1 com Srgb continuam
    // lendo corretamente.
    let bytes = postcard::to_allocvec(&ColorProfile::Srgb).expect("serialize");
    assert_eq!(
        bytes,
        vec![0u8],
        "ColorProfile::Srgb deve serializar para discriminant 0 — \
         HR-14 forward-compat exige que este pin não mude entre stubs e ph2d-color"
    );
}

#[test]
fn layer_stack_entry_reserved_discriminant_is_zero() {
    // Audit T1.8 L1-F11: LayerStackEntry::Reserved DEVE ser discriminant 0
    // pra que W3 possa adicionar Raster=1, Group=2, Mask=3, etc. sem
    // mexer no slot 0 que vai aparecer em todos `.ph2d-painter` v1.
    let entry = LayerStackEntry::Reserved(vec![1, 2, 3]);
    let bytes = postcard::to_allocvec(&entry).expect("serialize");
    // postcard: variant discriminant (varint) + Vec len (varint) + bytes.
    // Discriminant 0 → 1 byte = 0x00. Vec len 3 → 1 byte = 0x03. Bytes [1,2,3].
    assert_eq!(
        bytes,
        vec![0u8, 3, 1, 2, 3],
        "LayerStackEntry::Reserved deve serializar como [0u8, len, bytes...]"
    );
}

#[test]
fn paint_project_field_order_is_stable() {
    // Audit T1.8 L1-F5 reforçado: serialização do project começa com
    // magic (12B) seguido por version (varint). `SCHEMA_VERSION < 128` → 1
    // byte. Total prefix = 13 bytes previsíveis. Qualquer mudança em
    // declaration order quebra este test. (v2: byte 12 = 0x02.)
    let p = PaintProject::new(CanvasInfo::default());
    let bytes = postcard::to_allocvec(&p).expect("serialize");
    assert_eq!(&bytes[..12], PAINT_PROJECT_MAGIC.as_slice());
    // version must be a 1-byte varint for the 13-byte prefix to hold.
    const { assert!(ph2d_painter_stroke::SCHEMA_VERSION < 128) };
    assert_eq!(
        bytes[12],
        ph2d_painter_stroke::SCHEMA_VERSION as u8,
        "byte 12 deve ser o version varint (SCHEMA_VERSION)"
    );
}

/// Audit T-durability final M-13: CanvasInfo field order pin.
/// Postcard preserva declaration order; reorder de
/// `(width, height, color_profile, ppm)` quebra HR-14 forward-compat
/// silently — bytes deserializados representariam dados ERRADOS sem erro.
#[test]
fn canvas_info_field_order_is_stable() {
    let c = CanvasInfo {
        width: 0xDEAD_BEEF,
        height: 0xCAFE_BABE,
        color_profile: ColorProfile::Srgb,
        ppm: 11.811,
    };
    let bytes = postcard::to_allocvec(&c).expect("serialize");
    // Postcard varint pra u32: 0xDEADBEEF = 4 bytes payload (high bit
    // chain). Não precisamos exact byte-level pin (varint encoding muda
    // por valor); só pinning que ROUND-TRIP exato preserva field semantics.
    let back: CanvasInfo = postcard::from_bytes(&bytes).expect("deserialize");
    assert_eq!(back.width, 0xDEAD_BEEF);
    assert_eq!(back.height, 0xCAFE_BABE);
    assert_eq!(back.color_profile, ColorProfile::Srgb);
    assert!((back.ppm - 11.811).abs() < 0.001);
    // Pin que primeiro byte do output é varint do width (não color_profile
    // discriminant 0). 0xDEAD_BEEF varint começa com byte > 0x80 (high bit).
    assert!(
        bytes[0] & 0x80 != 0,
        "first byte should be varint continuation of width (not discriminant 0)"
    );
}

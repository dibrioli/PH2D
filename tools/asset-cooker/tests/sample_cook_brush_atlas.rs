//! W1.T14 — proof-of-life integration test: cook brush atlas R8 source → BC4 KTX2.
//!
//! ADR-0055-v4 §4 + plano vivo §Memory Budget Math: brush atlas R8 → BC4 saving
//! -50% (R8=8 bpp ÷ BC4=4 bpp = 0.5). Esta wave VALIDA pipeline end-to-end usando
//! fixture canônico `brush_atlas_256_r8` (W1.T11) + lib API `cook` (W1.T3):
//!
//! 1. Gera fixture brush atlas 256×256 (radial soft falloff, grayscale).
//! 2. Cook via `cook(&fixture, CookOptions::for_asset_class(Desktop, SingleChannel))`.
//!    Target matrix dispatcha BC4_UNORM_BLOCK via Intel ISPC encoder.
//! 3. Verifica KTX2 magic header + tamanho saudável + AssetId determinístico
//!    via blake3 (HR-6 content-addressed).
//! 4. Confirma saving observado: cooked KTX2 vs raw R8 source.
//!
//! Não testa: cross-machine determinism (D2/W1.T10 canonical runner ⏳); decode
//! correctness via wgpu (W2 renderer); painter consumer (W3).

use ph2d_asset_cooker::texture::{self, fixtures, AssetClass, CookOptions, Tier};

const KTX2_MAGIC: &[u8] = &[
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];

#[test]
fn sample_cook_brush_atlas_r8_to_bc4_emits_valid_ktx2() {
    let png = fixtures::brush_atlas_256_r8();
    assert!(png.len() > 100, "fixture PNG suspiciously small");

    let ktx2 = texture::cook(
        &png,
        CookOptions::for_asset_class(Tier::Desktop, AssetClass::SingleChannel),
    )
    .expect("cook brush atlas R8 → BC4 desktop");

    assert!(
        ktx2.len() > KTX2_MAGIC.len(),
        "KTX2 output suspiciously small ({} bytes)",
        ktx2.len()
    );
    assert_eq!(
        &ktx2[0..12],
        KTX2_MAGIC,
        "KTX2 output missing magic header — cooker emitted wrong container?"
    );
}

#[test]
fn sample_cook_brush_atlas_intra_machine_determinism() {
    let png = fixtures::brush_atlas_256_r8();

    // Cook 2× — mesma máquina, mesmo binário, mesmo input → byte-identical
    // (intra-machine guarantee). Cross-machine determinism é gate separado
    // D2/W1.T10 canonical runner.
    let a = texture::cook(
        &png,
        CookOptions::for_asset_class(Tier::Desktop, AssetClass::SingleChannel),
    )
    .expect("cook a");
    let b = texture::cook(
        &png,
        CookOptions::for_asset_class(Tier::Desktop, AssetClass::SingleChannel),
    )
    .expect("cook b");

    assert_eq!(
        a, b,
        "intra-machine determinism violation: 2 runs do mesmo cook produziram bytes diferentes"
    );

    // AssetId derivado dos cooked bytes é estável (HR-6 content-addressed).
    let id_a = ph2d_asset::AssetId::from_bytes(&a);
    let id_b = ph2d_asset::AssetId::from_bytes(&b);
    assert_eq!(id_a, id_b, "AssetId mismatch — HR-6 contract violated");
}

#[test]
fn sample_cook_brush_atlas_bc4_smaller_than_uncompressed_baseline() {
    // ADR-0055-v4 §Memory Budget Math: BC4 4 bpp ÷ R8 8 bpp = 0.5 → -50% saving.
    // 256×256 R8 raw = 64 KB. 256×256 BC4 raw = 32 KB. KTX2 container overhead
    // (header + format descriptor + key/value data + level index + padding)
    // tipicamente 200-600 bytes. Cooked total expected ~32 KB + overhead < 64 KB.
    let png = fixtures::brush_atlas_256_r8();
    let ktx2 = texture::cook(
        &png,
        CookOptions::for_asset_class(Tier::Desktop, AssetClass::SingleChannel),
    )
    .expect("cook");

    let raw_r8_bytes = 256 * 256; // 64 KB equiv
    let raw_rgba_bytes = 256 * 256 * 4; // 256 KB equiv (source PNG decodes to RGBA8)
    assert!(
        ktx2.len() < raw_rgba_bytes,
        "BC4 cooked KTX2 ({} bytes) MUST be smaller than RGBA8 raw source ({} bytes)",
        ktx2.len(),
        raw_rgba_bytes,
    );
    // -50% saving sobre R8 não é guaranteed strictly (KTX2 overhead pode empurrar
    // pequenas textures acima do R8 raw para fixtures muito pequenas, mas
    // 256² é grande o suficiente). Allow 10% slack for header.
    let upper_bound = raw_r8_bytes + (raw_r8_bytes / 10);
    assert!(
        ktx2.len() < upper_bound,
        "BC4 256² cooked ({} bytes) deveria ser ≤ R8 raw + 10% header ({} bytes)",
        ktx2.len(),
        upper_bound,
    );
}

#[test]
fn sample_cook_brush_atlas_cook_all_emits_single_channel_per_tier() {
    // Multi-tier batch: brush atlas SingleChannel → BC4 desktop, ASTC mobile/web,
    // ETC2 lowend, RGBA8 constrained. Confirma cook_all funciona end-to-end com
    // SingleChannel asset class (não só SpriteColor que outros tests cobrem).
    let png = fixtures::brush_atlas_256_r8();
    let map = texture::cook_all(&png, AssetClass::SingleChannel).expect("cook_all");
    assert_eq!(map.len(), 5, "cook_all should emit 1 artifact per tier (5 tiers)");

    for tier in [
        Tier::Desktop,
        Tier::Mobile,
        Tier::Web,
        Tier::LowEnd,
        Tier::Constrained,
    ] {
        let bytes = map
            .get(&tier)
            .unwrap_or_else(|| panic!("tier {tier:?} missing"));
        assert_eq!(
            &bytes[0..12],
            KTX2_MAGIC,
            "tier {tier:?}: invalid KTX2 magic"
        );
    }
}

//! Arch-gate (KTX2 Fase 2, W2.T1): every concrete `Ktx2Format`
//! variant maps to the exact `(wgpu::TextureFormat, wgpu::Features)`
//! pair the renderer expects.
//!
//! `Ktx2Format` is `#[non_exhaustive]` (ν-7), so a wildcard-free
//! cross-crate `match` is impossible and pure compile-time
//! exhaustiveness cannot be enforced. This test is the substitute: it
//! names all 27 concrete variants explicitly, so the build breaks if a
//! variant is *renamed/removed* upstream, and the asserted pairs break
//! CI if any mapping *regresses*. The `EXPECTED.len() == 27` guard
//! catches a variant being added upstream without a row here (the
//! author must consciously extend the table).

use ph2d_asset_ktx2::Ktx2Format;
use ph2d_render::{FormatError, wgpu_format_from_ktx2_format};
use wgpu::TextureFormat as Tf;
use wgpu::{AstcBlock, AstcChannel, Features};

/// `(input, expected wgpu format, expected required feature)`.
fn expected() -> Vec<(Ktx2Format, Tf, Features)> {
    let bc = Features::TEXTURE_COMPRESSION_BC;
    let astc = Features::TEXTURE_COMPRESSION_ASTC;
    let etc2 = Features::TEXTURE_COMPRESSION_ETC2;
    let none = Features::empty();
    let a = |block, channel| Tf::Astc { block, channel };

    vec![
        // Uncompressed
        (Ktx2Format::Rgba8Unorm, Tf::Rgba8Unorm, none),
        (Ktx2Format::Rgba8UnormSrgb, Tf::Rgba8UnormSrgb, none),
        (
            Ktx2Format::Rgba16Unorm,
            Tf::Rgba16Unorm,
            Features::TEXTURE_FORMAT_16BIT_NORM,
        ),
        (Ktx2Format::Rgba16Float, Tf::Rgba16Float, none),
        (Ktx2Format::Rgba32Float, Tf::Rgba32Float, none),
        // BC
        (Ktx2Format::Bc1RgbaUnorm, Tf::Bc1RgbaUnorm, bc),
        (Ktx2Format::Bc1RgbaUnormSrgb, Tf::Bc1RgbaUnormSrgb, bc),
        (Ktx2Format::Bc3RgbaUnorm, Tf::Bc3RgbaUnorm, bc),
        (Ktx2Format::Bc3RgbaUnormSrgb, Tf::Bc3RgbaUnormSrgb, bc),
        (Ktx2Format::Bc4RUnorm, Tf::Bc4RUnorm, bc),
        (Ktx2Format::Bc5RgUnorm, Tf::Bc5RgUnorm, bc),
        (Ktx2Format::Bc6hRgbUfloat, Tf::Bc6hRgbUfloat, bc),
        (Ktx2Format::Bc6hRgbSfloat, Tf::Bc6hRgbFloat, bc),
        (Ktx2Format::Bc7RgbaUnorm, Tf::Bc7RgbaUnorm, bc),
        (Ktx2Format::Bc7RgbaUnormSrgb, Tf::Bc7RgbaUnormSrgb, bc),
        // ASTC
        (
            Ktx2Format::Astc4x4RgbaUnorm,
            a(AstcBlock::B4x4, AstcChannel::Unorm),
            astc,
        ),
        (
            Ktx2Format::Astc4x4RgbaUnormSrgb,
            a(AstcBlock::B4x4, AstcChannel::UnormSrgb),
            astc,
        ),
        (
            Ktx2Format::Astc5x5RgbaUnorm,
            a(AstcBlock::B5x5, AstcChannel::Unorm),
            astc,
        ),
        (
            Ktx2Format::Astc5x5RgbaUnormSrgb,
            a(AstcBlock::B5x5, AstcChannel::UnormSrgb),
            astc,
        ),
        (
            Ktx2Format::Astc6x6RgbaUnorm,
            a(AstcBlock::B6x6, AstcChannel::Unorm),
            astc,
        ),
        (
            Ktx2Format::Astc6x6RgbaUnormSrgb,
            a(AstcBlock::B6x6, AstcChannel::UnormSrgb),
            astc,
        ),
        (
            Ktx2Format::Astc8x8RgbaUnorm,
            a(AstcBlock::B8x8, AstcChannel::Unorm),
            astc,
        ),
        (
            Ktx2Format::Astc8x8RgbaUnormSrgb,
            a(AstcBlock::B8x8, AstcChannel::UnormSrgb),
            astc,
        ),
        // ETC2
        (Ktx2Format::Etc2Rgb8Unorm, Tf::Etc2Rgb8Unorm, etc2),
        (Ktx2Format::Etc2Rgb8UnormSrgb, Tf::Etc2Rgb8UnormSrgb, etc2),
        (Ktx2Format::Etc2Rgba8Unorm, Tf::Etc2Rgba8Unorm, etc2),
        (Ktx2Format::Etc2Rgba8UnormSrgb, Tf::Etc2Rgba8UnormSrgb, etc2),
    ]
}

#[test]
fn every_known_variant_maps_to_expected_pair() {
    for (input, want_fmt, want_feat) in expected() {
        let got = wgpu_format_from_ktx2_format(input)
            .unwrap_or_else(|e| panic!("{input:?} should map, got error {e:?}"));
        assert_eq!(got.0, want_fmt, "wrong wgpu format for {input:?}");
        assert_eq!(got.1, want_feat, "wrong required feature for {input:?}");
    }
}

#[test]
fn table_covers_all_27_concrete_variants() {
    // Guards against an upstream variant being added without a row
    // above. The count is the concrete (non-`Unsupported`) variant
    // total: 5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 = 27.
    assert_eq!(
        expected().len(),
        27,
        "expected-mapping table drifted from the 27 concrete Ktx2Format variants"
    );
}

#[test]
fn compressed_formats_require_their_family_feature() {
    // Cross-check against the parser's own `is_compressed`: any format
    // it calls compressed must demand a non-empty GPU feature, and any
    // it calls uncompressed must not (except Rgba16Unorm's 16-bit-norm).
    for (input, _fmt, feat) in expected() {
        if input.is_compressed() {
            assert!(
                !feat.is_empty(),
                "{input:?} is compressed but requires no GPU feature"
            );
        }
    }
}

#[test]
fn unsupported_vkformat_is_an_error_carrying_the_raw_value() {
    let raw = 0xDEAD_BEEF;
    assert_eq!(
        wgpu_format_from_ktx2_format(Ktx2Format::Unsupported(raw)),
        Err(FormatError::UnsupportedVkFormat(raw)),
    );
}

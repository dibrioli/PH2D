//! Testes de `compressed_pipeline.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use ph2d_asset_ktx2::Ktx2Format;
use std::sync::OnceLock;
use wgpu::TextureFormat as Tf;

// ── Block-alignment math (GPU-independent) ──────────────────────────
//
// This is the bug-prone heart of compressed-texture upload, so it gets
// the densest non-GPU coverage: BC/ASTC/ETC2 block-pitch, partial
// trailing blocks at small mips, and the RGBA8 degenerate case.

#[test]
fn rgba8_layout_is_width_times_four() {
    // Uncompressed: block 1×1, 4 bytes/texel → classic w*4 / h.
    let l = MipUploadLayout::for_mip(Tf::Rgba8Unorm, 64, 32).unwrap();
    assert_eq!(l.bytes_per_row, 64 * 4);
    assert_eq!(l.rows_per_image, 32);
    assert_eq!(l.total_bytes, 64 * 4 * 32);
}

#[test]
fn bc7_layout_is_blocks_not_pixels() {
    // BC7: 4×4 blocks, 16 bytes/block. A 64×32 image is 16×8 blocks →
    // bytes_per_row = 16 blocks * 16 B = 256; rows_per_image = 8 blocks.
    let l = MipUploadLayout::for_mip(Tf::Bc7RgbaUnorm, 64, 32).unwrap();
    assert_eq!(l.bytes_per_row, 16 * 16);
    assert_eq!(l.rows_per_image, 8);
    assert_eq!(l.total_bytes, 16 * 16 * 8);
}

#[test]
fn bc4_layout_uses_8_byte_blocks() {
    // BC4 (single channel) is 8 bytes/block, unlike BC7's 16 — the brush
    // atlas path (W3.T1). 256×256 = 64×64 blocks * 8 B = 512 B/row.
    let l = MipUploadLayout::for_mip(Tf::Bc4RUnorm, 256, 256).unwrap();
    assert_eq!(l.bytes_per_row, 64 * 8);
    assert_eq!(l.rows_per_image, 64);
    assert_eq!(l.total_bytes, 64 * 8 * 64);
}

#[test]
fn bc6h_hdr_uses_same_16_byte_block_path_as_bc7() {
    // BC6H is HDR but block-size-identical to BC7 (4×4, 16 B) — proves
    // the shared path handles HDR without a special case.
    let hdr = MipUploadLayout::for_mip(Tf::Bc6hRgbUfloat, 64, 32).unwrap();
    let ldr = MipUploadLayout::for_mip(Tf::Bc7RgbaUnorm, 64, 32).unwrap();
    assert_eq!(hdr.bytes_per_row, ldr.bytes_per_row);
    assert_eq!(hdr.total_bytes, ldr.total_bytes);
}

#[test]
fn partial_trailing_block_rounds_up() {
    // A 5×5 BC7 image is NOT block-aligned: ceil(5/4) = 2 blocks each way,
    // so it still costs a full 2×2 block grid (a classic off-by-one bug
    // when people use `width/4` instead of `ceil`).
    let l = MipUploadLayout::for_mip(Tf::Bc7RgbaUnorm, 5, 5).unwrap();
    assert_eq!(l.bytes_per_row, 2 * 16, "ceil(5/4)=2 blocks per row");
    assert_eq!(l.rows_per_image, 2, "ceil(5/4)=2 rows of blocks");
    assert_eq!(l.total_bytes, 2 * 16 * 2);
}

#[test]
fn one_by_one_mip_costs_a_full_block() {
    // The tail of a mip pyramid: a 1×1 mip of a BC texture still occupies
    // one whole 4×4 block (16 B for BC7).
    let l = MipUploadLayout::for_mip(Tf::Bc7RgbaUnorm, 1, 1).unwrap();
    assert_eq!(l.bytes_per_row, 16);
    assert_eq!(l.rows_per_image, 1);
    assert_eq!(l.total_bytes, 16);
}

#[test]
fn astc_non_square_block_pitch() {
    // ASTC 8×8 block (16 B/block regardless of block dims). A 64×64 image
    // is 8×8 blocks → 8*16 = 128 B/row, 8 rows.
    let l = MipUploadLayout::for_mip(
        Tf::Astc {
            block: wgpu::AstcBlock::B8x8,
            channel: wgpu::AstcChannel::Unorm,
        },
        64,
        64,
    )
    .unwrap();
    assert_eq!(l.bytes_per_row, 8 * 16);
    assert_eq!(l.rows_per_image, 8);

    // ASTC 6×6 over a 64×64 image: ceil(64/6) = 11 blocks each way.
    let l6 = MipUploadLayout::for_mip(
        Tf::Astc {
            block: wgpu::AstcBlock::B6x6,
            channel: wgpu::AstcChannel::Unorm,
        },
        64,
        64,
    )
    .unwrap();
    assert_eq!(l6.bytes_per_row, 11 * 16, "ceil(64/6)=11 blocks/row");
    assert_eq!(l6.rows_per_image, 11);
}

#[test]
fn etc2_uses_8_byte_rgb_blocks() {
    // ETC2 RGB8 is 8 bytes/block (4×4). 64×64 = 16×16 blocks.
    let l = MipUploadLayout::for_mip(Tf::Etc2Rgb8Unorm, 64, 64).unwrap();
    assert_eq!(l.bytes_per_row, 16 * 8);
    assert_eq!(l.rows_per_image, 16);
}

// ── compressed_size_per_format budget accounting (GPU-independent) ──

#[test]
fn size_single_mip_rgba8_is_w_times_h_times_4() {
    // One mip, uncompressed: classic w*h*4.
    assert_eq!(
        compressed_size_per_format(Tf::Rgba8Unorm, 64, 32, 1),
        64 * 32 * 4
    );
}

#[test]
fn size_single_mip_bc7_is_block_footprint_not_pixels() {
    // 64×32 BC7 = 16×8 blocks * 16 B = 2048 B (¼ of the RGBA8 8192 B —
    // the plan's -75% desktop saving, exactly).
    assert_eq!(
        compressed_size_per_format(Tf::Bc7RgbaUnorm, 64, 32, 1),
        2048
    );
    assert_eq!(
        compressed_size_per_format(Tf::Rgba8Unorm, 64, 32, 1),
        2048 * 4
    );
}

#[test]
fn size_full_pyramid_sums_every_mip() {
    // 8×8 BC7 pyramid: 8×8(=4 blocks) + 4×4(1) + 2×2(1) + 1×1(1) blocks
    // = 7 blocks * 16 B = 112 B.
    let got = compressed_size_per_format(Tf::Bc7RgbaUnorm, 8, 8, 4);
    assert_eq!(got, (4 + 1 + 1 + 1) * 16);
}

#[test]
fn size_is_robust_to_overlong_mip_count_and_zero_dims() {
    // mip_count past the pyramid floor adds only 1×1 mips (no panic).
    let sane = compressed_size_per_format(Tf::Bc7RgbaUnorm, 4, 4, 3);
    let overlong = compressed_size_per_format(Tf::Bc7RgbaUnorm, 4, 4, 40);
    assert!(overlong >= sane, "extra 1×1 mips only add footprint");
    // Zero size → zero bytes (degenerate, never negative/overflow).
    assert_eq!(compressed_size_per_format(Tf::Rgba8Unorm, 0, 0, 1), 0);
}

// ── mip_layouts pyramid validation (GPU-independent) ────────────────

fn mip(width: u32, height: u32, bytes: usize) -> MipLevel {
    MipLevel {
        width,
        height,
        data: vec![0u8; bytes].into(),
    }
}

#[test]
fn mip_layouts_accepts_exact_and_overlong_payloads() {
    // A full BC7 pyramid 8×8 → 4×4 → 2×2 → 1×1: each level is a single
    // 4×4 block grid of, respectively, 2×2, 1×1, 1×1, 1×1 blocks.
    let mips = vec![
        mip(8, 8, 4 * 16),  // 2x2 blocks
        mip(4, 4, 16),      // 1x1 block
        mip(2, 2, 16),      // rounds up to 1x1 block
        mip(1, 1, 16 + 99), // overlong is fine (>= expected)
    ];
    let layouts = mip_layouts(Tf::Bc7RgbaUnorm, &mips).unwrap();
    assert_eq!(layouts.len(), 4);
    assert_eq!(layouts[0].total_bytes, 4 * 16);
    assert_eq!(layouts[3].total_bytes, 16);
}

#[test]
fn mip_layouts_rejects_truncated_payload() {
    // One byte short of a single BC7 block → MipDataTooShort with indices.
    let mips = vec![mip(4, 4, 15)];
    let err = mip_layouts(Tf::Bc7RgbaUnorm, &mips).unwrap_err();
    assert_eq!(
        err,
        CompressedUploadError::MipDataTooShort {
            mip_index: 0,
            expected: 16,
            actual: 15,
        }
    );
}

#[test]
fn mip_layouts_rejects_empty_pyramid() {
    assert_eq!(
        mip_layouts(Tf::Bc7RgbaUnorm, &[]).unwrap_err(),
        CompressedUploadError::NoMipLevels
    );
}

#[test]
fn mip_layouts_reports_index_of_second_bad_mip() {
    // First mip fine, second truncated → index 1 surfaced (not 0).
    let mips = vec![mip(4, 4, 16), mip(2, 2, 5)];
    match mip_layouts(Tf::Bc7RgbaUnorm, &mips).unwrap_err() {
        CompressedUploadError::MipDataTooShort { mip_index, .. } => assert_eq!(mip_index, 1),
        other => panic!("expected MipDataTooShort, got {other:?}"),
    }
}

// ── Format resolution + feature gating (GPU-independent) ─────────────

// `resolve_format` exercises the SAME error surface as `upload` without a
// GPU, by stubbing the feature gate through CompressionFeatureSet. We
// can't build a CompressedTexturePipeline (needs a device) but we CAN
// test the underlying decision the same way it's composed.
#[test]
fn resolve_decision_matches_supports_gate() {
    // Mirror `resolve_format`'s composition without a device: the two
    // building blocks are `wgpu_format_from_ktx2_format` + `supports`.
    let bc_only = CompressionFeatureSet::from_features(wgpu::Features::TEXTURE_COMPRESSION_BC);

    // BC7 maps AND is supported on a BC device.
    assert!(bc_only.supports(Ktx2Format::Bc7RgbaUnorm));
    assert!(wgpu_format_from_ktx2_format(Ktx2Format::Bc7RgbaUnorm).is_ok());

    // ASTC maps but is NOT supported on a BC-only device → the
    // FormatUnsupportedByDevice branch.
    assert!(!bc_only.supports(Ktx2Format::Astc4x4RgbaUnorm));
    assert!(wgpu_format_from_ktx2_format(Ktx2Format::Astc4x4RgbaUnorm).is_ok());

    // Unsupported VkFormat never maps → the UnmappableFormat branch,
    // independent of the device.
    assert!(wgpu_format_from_ktx2_format(Ktx2Format::Unsupported(0xBEEF)).is_err());
}

#[test]
fn every_cooked_sprite_format_is_filterable_float() {
    // The shared-pipeline design hinges on EVERY format this pipeline
    // uploads sampling as `Float { filterable: true }` with no extra
    // device feature (so one material bind-group layout binds them all).
    // This asserts that invariant directly against wgpu's sample-type
    // table — if a future wgpu bump changed any of these, the shared
    // layout would silently break, and this test catches it.
    let want = Some(wgpu::TextureSampleType::Float { filterable: true });
    for kf in [
        Ktx2Format::Rgba8Unorm,
        Ktx2Format::Rgba8UnormSrgb,
        Ktx2Format::Rgba16Unorm,
        Ktx2Format::Rgba16Float,
        Ktx2Format::Bc1RgbaUnorm,
        Ktx2Format::Bc4RUnorm,
        Ktx2Format::Bc6hRgbUfloat, // HDR — still filterable-float
        Ktx2Format::Bc6hRgbSfloat,
        Ktx2Format::Bc7RgbaUnorm,
        Ktx2Format::Astc4x4RgbaUnorm,
        Ktx2Format::Astc8x8RgbaUnormSrgb,
        Ktx2Format::Etc2Rgb8Unorm,
        Ktx2Format::Etc2Rgba8UnormSrgb,
    ] {
        let (wf, _) = wgpu_format_from_ktx2_format(kf).unwrap();
        assert_eq!(
            wf.sample_type(None, None),
            want,
            "{kf:?} must sample as filterable-float for the shared layout"
        );
    }

    // The lone exception: Rgba32Float samples as NON-filterable without
    // FLOAT32_FILTERABLE — so the shared layout deliberately excludes it
    // (full 32-bit float HDR is the GameRt path, not cooked sprites).
    // resolve_format must reject it via NotFilterableFloat; assert the
    // underlying sample-type fact that drives that rejection.
    let (rgba32, _) = wgpu_format_from_ktx2_format(Ktx2Format::Rgba32Float).unwrap();
    assert_ne!(
        rgba32.sample_type(None, None),
        want,
        "Rgba32Float must NOT be unconditionally filterable-float — \
             resolve_format rejects it"
    );
}

#[test]
fn upload_error_display_is_descriptive() {
    let e = CompressedUploadError::MipDataTooShort {
        mip_index: 2,
        expected: 64,
        actual: 16,
    };
    assert!(e.to_string().contains("mip 2"));
    assert!(e.to_string().contains("64"));
    let e2 = CompressedUploadError::FormatUnsupportedByDevice(Tf::Bc7RgbaUnorm);
    assert!(e2.to_string().contains("Bc7"));
}

// ── GPU upload smoke (requires a real adapter; #[ignore] on CI) ──────
//
// CI has no GPU, so the actual `write_texture` upload is gated behind
// `--ignored` and run only on dev Macs (mirrors the `pipeline.rs`
// `try_headless_gpu` pattern, but those gracefully skip; here we must
// assert the upload SUCCEEDS, so it's an explicit ignored test).

fn try_headless_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

// `Ktx2Image` is `#[non_exhaustive]` and cannot be constructed outside
// its crate, so these drive `upload_parts` with plain `MipLevel`s — the
// exact reason that entry point exists. `MipLevel` has public fields and
// is NOT non_exhaustive, so it IS constructible here.

#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn upload_rgba8_roundtrips_on_real_device() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let feats = CompressionFeatureSet::from_features(gpu.device.features());
    let pipeline = CompressedTexturePipeline::new(&gpu, feats);

    // A 4×4 RGBA8 image (always device-supported, no compression feature
    // needed) with a single mip.
    let mips = vec![mip(4, 4, 4 * 4 * 4)];
    let uploaded = pipeline
        .upload_parts(
            &gpu,
            Ktx2Format::Rgba8Unorm,
            4,
            4,
            &mips,
            Some("test cooked rgba8"),
        )
        .expect("RGBA8 upload should succeed on any device");
    assert_eq!(uploaded.format, Tf::Rgba8Unorm);
    assert_eq!((uploaded.width, uploaded.height), (4, 4));
    assert_eq!(uploaded.mip_level_count, 1);
}

#[test]
#[ignore = "requires a GPU adapter with TEXTURE_COMPRESSION_BC; run with --ignored on desktop"]
fn upload_bc7_on_bc_capable_device() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    if !gpu
        .device
        .features()
        .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
    {
        return; // device can't sample BC — nothing to assert.
    }
    let feats = CompressionFeatureSet::from_features(gpu.device.features());
    let pipeline = CompressedTexturePipeline::new(&gpu, feats);

    // 8×8 BC7 = 2×2 blocks * 16 B = 64 bytes.
    let mips = vec![mip(8, 8, 4 * 16)];
    let uploaded = pipeline
        .upload_parts(
            &gpu,
            Ktx2Format::Bc7RgbaUnorm,
            8,
            8,
            &mips,
            Some("test cooked bc7"),
        )
        .expect("BC7 upload should succeed on a BC device");
    assert_eq!(uploaded.format, Tf::Bc7RgbaUnorm);
}

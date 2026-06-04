use super::*;
use crate::decode::validate_2d_only;
use crate::patch::{parse_level_index, serialize_kvd};
use std::collections::BTreeMap;
use std::sync::Arc;

// ── uncompressed RGBA8 encoder round-trip (W2.T4) ──────────────

#[test]
fn encode_uncompressed_rgba8_round_trips_through_decode() {
    // A 2×2 RGBA8 image: distinct corners so a transposed / mis-strided
    // decode would be caught.
    let (w, h) = (2u32, 2u32);
    let rgba: Vec<u8> = vec![
        255, 0, 0, 255, // TL red
        0, 255, 0, 255, // TR green
        0, 0, 255, 255, // BL blue
        255, 255, 0, 128, // BR semi-transparent yellow
    ];
    for srgb in [true, false] {
        let bytes = encode_uncompressed_rgba8(w, h, srgb, &rgba).expect("encode");
        let img = decode_ktx2_bytes(&bytes).expect("decode round-trips");
        assert_eq!((img.width, img.height), (w, h));
        assert_eq!(img.mip_levels.len(), 1, "single mip");
        assert_eq!(&img.base_level().data[..], &rgba[..], "payload preserved");
        let want = if srgb {
            Ktx2Format::Rgba8UnormSrgb
        } else {
            Ktx2Format::Rgba8Unorm
        };
        assert_eq!(
            img.format, want,
            "transfer function preserved (srgb={srgb})"
        );
    }
}

#[test]
fn encode_uncompressed_rgba8_rejects_bad_inputs() {
    // Wrong buffer length → InvalidContainer (not a panic / silent crop).
    assert!(matches!(
        encode_uncompressed_rgba8(2, 2, true, &[0u8; 3]),
        Err(Ktx2Error::InvalidContainer(_))
    ));
    // Zero dimension → ZeroDimension.
    assert!(matches!(
        encode_uncompressed_rgba8(0, 4, true, &[]),
        Err(Ktx2Error::ZeroDimension)
    ));
}

// ── garbage-input rejection (no fixture needed) ────────────────

/// Empty buffer must error, not panic.
#[test]
fn decode_empty_bytes_errors() {
    let result = decode_ktx2_bytes(&[]);
    assert!(matches!(result, Err(Ktx2Error::InvalidContainer(_))));
}

/// Garbage bytes must error, not panic.
#[test]
fn decode_random_bytes_errors() {
    // Definitely not the KTX2 magic (`«KTX 20»\r\n\x1a\n`, 12 bytes).
    let bogus: [u8; 32] = [0xAB; 32];
    let result = decode_ktx2_bytes(&bogus);
    assert!(matches!(result, Err(Ktx2Error::InvalidContainer(_))));
}

/// Truncated header (only the magic) must error cleanly.
#[test]
fn decode_only_magic_errors() {
    const MAGIC: [u8; 12] = [
        0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ];
    let result = decode_ktx2_bytes(&MAGIC);
    assert!(matches!(result, Err(Ktx2Error::InvalidContainer(_))));
}

// ── VkFormat mapping coverage ─────────────────────────────────

/// The full canonical mapping. The (raw, expected) pairs are
/// built from `ktx2::Format::*` constants — if any of the Vulkan
/// registry IDs we use is wrong, this stops compiling (typed
/// reference) rather than passing with a typo (raw integer
/// literal).
fn canonical_format_table() -> Vec<(u32, Ktx2Format)> {
    use ktx2::Format as F;
    vec![
        (F::R8G8B8A8_UNORM.value(), Ktx2Format::Rgba8Unorm),
        (F::R8G8B8A8_SRGB.value(), Ktx2Format::Rgba8UnormSrgb),
        (F::R16G16B16A16_UNORM.value(), Ktx2Format::Rgba16Unorm),
        (F::R16G16B16A16_SFLOAT.value(), Ktx2Format::Rgba16Float),
        (F::R32G32B32A32_SFLOAT.value(), Ktx2Format::Rgba32Float),
        (F::BC1_RGBA_UNORM_BLOCK.value(), Ktx2Format::Bc1RgbaUnorm),
        (F::BC1_RGBA_SRGB_BLOCK.value(), Ktx2Format::Bc1RgbaUnormSrgb),
        (F::BC3_UNORM_BLOCK.value(), Ktx2Format::Bc3RgbaUnorm),
        (F::BC3_SRGB_BLOCK.value(), Ktx2Format::Bc3RgbaUnormSrgb),
        (F::BC4_UNORM_BLOCK.value(), Ktx2Format::Bc4RUnorm),
        (F::BC5_UNORM_BLOCK.value(), Ktx2Format::Bc5RgUnorm),
        (F::BC6H_UFLOAT_BLOCK.value(), Ktx2Format::Bc6hRgbUfloat),
        (F::BC6H_SFLOAT_BLOCK.value(), Ktx2Format::Bc6hRgbSfloat),
        (F::BC7_UNORM_BLOCK.value(), Ktx2Format::Bc7RgbaUnorm),
        (F::BC7_SRGB_BLOCK.value(), Ktx2Format::Bc7RgbaUnormSrgb),
        (
            F::ETC2_R8G8B8_UNORM_BLOCK.value(),
            Ktx2Format::Etc2Rgb8Unorm,
        ),
        (
            F::ETC2_R8G8B8_SRGB_BLOCK.value(),
            Ktx2Format::Etc2Rgb8UnormSrgb,
        ),
        (
            F::ETC2_R8G8B8A8_UNORM_BLOCK.value(),
            Ktx2Format::Etc2Rgba8Unorm,
        ),
        (
            F::ETC2_R8G8B8A8_SRGB_BLOCK.value(),
            Ktx2Format::Etc2Rgba8UnormSrgb,
        ),
        (
            F::ASTC_4x4_UNORM_BLOCK.value(),
            Ktx2Format::Astc4x4RgbaUnorm,
        ),
        (
            F::ASTC_4x4_SRGB_BLOCK.value(),
            Ktx2Format::Astc4x4RgbaUnormSrgb,
        ),
        (
            F::ASTC_5x5_UNORM_BLOCK.value(),
            Ktx2Format::Astc5x5RgbaUnorm,
        ),
        (
            F::ASTC_5x5_SRGB_BLOCK.value(),
            Ktx2Format::Astc5x5RgbaUnormSrgb,
        ),
        (
            F::ASTC_6x6_UNORM_BLOCK.value(),
            Ktx2Format::Astc6x6RgbaUnorm,
        ),
        (
            F::ASTC_6x6_SRGB_BLOCK.value(),
            Ktx2Format::Astc6x6RgbaUnormSrgb,
        ),
        (
            F::ASTC_8x8_UNORM_BLOCK.value(),
            Ktx2Format::Astc8x8RgbaUnorm,
        ),
        (
            F::ASTC_8x8_SRGB_BLOCK.value(),
            Ktx2Format::Astc8x8RgbaUnormSrgb,
        ),
    ]
}

/// Every entry in the canonical table must round-trip through
/// `from_vk_format`. Exhaustive — not a representative sample.
#[test]
fn vk_format_mapping_via_ktx2_constants() {
    for (raw, expected) in canonical_format_table() {
        assert_eq!(
            Ktx2Format::from_vk_format(raw),
            expected,
            "VkFormat {raw} (= {expected:?}) misroutes",
        );
    }
}

/// Unknown VkFormat must surface as `Unsupported(raw)` rather
/// than silently mapping to something.
#[test]
fn vk_format_unknown_is_unsupported() {
    use ktx2::Format as F;

    // 9999 isn't a real VkFormat — pick anything outside our subset.
    assert_eq!(
        Ktx2Format::from_vk_format(9999),
        Ktx2Format::Unsupported(9999)
    );
    // VK_FORMAT_R4G4_UNORM_PACK8 (1) is real but we don't use it.
    assert_eq!(Ktx2Format::from_vk_format(1), Ktx2Format::Unsupported(1));
    // BC2 (135 / 136) is a real VkFormat we deliberately omitted
    // because no PH2D pipeline targets it (BC3 supersedes for
    // alpha-having sprites). Document the omission via test.
    assert_eq!(
        Ktx2Format::from_vk_format(F::BC2_UNORM_BLOCK.value()),
        Ktx2Format::Unsupported(F::BC2_UNORM_BLOCK.value()),
    );
}

// ── classifier exhaustiveness ─────────────────────────────────

/// Every canonical (= non-`Unsupported`) variant. Kept in lockstep
/// with [`Ktx2Format`] — adding a variant without updating this
/// table fails the exhaustiveness tests below.
fn all_canonical_formats() -> Vec<Ktx2Format> {
    canonical_format_table()
        .into_iter()
        .map(|(_, f)| f)
        .collect()
}

/// `is_compressed` must return `true` for every BC*/ASTC*/ETC2*
/// variant and `false` for every uncompressed RGBA variant.
#[test]
fn is_compressed_exhaustive() {
    for f in all_canonical_formats() {
        let expected = matches!(
            f,
            Ktx2Format::Rgba8Unorm
                | Ktx2Format::Rgba8UnormSrgb
                | Ktx2Format::Rgba16Unorm
                | Ktx2Format::Rgba16Float
                | Ktx2Format::Rgba32Float
        );
        assert_eq!(f.is_compressed(), !expected, "is_compressed for {f:?}");
    }
    // `Unsupported` is opaque — caller cannot reason about it.
    assert!(!Ktx2Format::Unsupported(9999).is_compressed());
}

/// `is_hdr` must return `true` ONLY for float-storage variants.
/// Rgba16Unorm specifically must be FALSE — it has 16 bits of
/// precision but the storage range is `[0, 1]`, identical to
/// RGBA8 from a dynamic-range standpoint.
#[test]
fn is_hdr_exhaustive() {
    for f in all_canonical_formats() {
        let expected = matches!(
            f,
            Ktx2Format::Rgba16Float
                | Ktx2Format::Rgba32Float
                | Ktx2Format::Bc6hRgbUfloat
                | Ktx2Format::Bc6hRgbSfloat
        );
        assert_eq!(f.is_hdr(), expected, "is_hdr for {f:?}");
    }
    // Explicit anti-regression — Rgba16Unorm has precision, not range.
    assert!(!Ktx2Format::Rgba16Unorm.is_hdr());
    assert!(!Ktx2Format::Unsupported(9999).is_hdr());
}

// ── dimensionality reject tests (header-only fixture) ──────────
//
// `validate_2d_only` operates on a parsed `ktx2::Header`, which
// `Header::from_bytes` builds from exactly 80 bytes — cheaper than
// a full KTX2 file for the layout-shape tests below.

/// Build a valid 80-byte KTX2 header. Defaults are a plain 2D
/// texture (8×8, no depth, no array, single face, RGBA8_SRGB).
fn build_header_bytes(pixel_depth: u32, layer_count: u32, face_count: u32) -> [u8; 80] {
    let mut buf = [0u8; 80];
    buf[0..12].copy_from_slice(&[
        0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]);
    buf[12..16].copy_from_slice(&43u32.to_le_bytes()); // VK_FORMAT_R8G8B8A8_SRGB
    buf[16..20].copy_from_slice(&1u32.to_le_bytes()); // type_size
    buf[20..24].copy_from_slice(&8u32.to_le_bytes()); // pixel_width
    buf[24..28].copy_from_slice(&8u32.to_le_bytes()); // pixel_height
    buf[28..32].copy_from_slice(&pixel_depth.to_le_bytes());
    buf[32..36].copy_from_slice(&layer_count.to_le_bytes());
    buf[36..40].copy_from_slice(&face_count.to_le_bytes());
    buf[40..44].copy_from_slice(&1u32.to_le_bytes()); // level_count
    buf[44..48].copy_from_slice(&0u32.to_le_bytes()); // supercompression = none
    buf
}

#[test]
fn validate_2d_only_accepts_plain_2d() {
    let bytes = build_header_bytes(0, 0, 1);
    let header = ktx2::Header::from_bytes(&bytes).expect("synthetic header parses");
    assert!(validate_2d_only(&header).is_ok());
}

#[test]
fn validate_2d_only_accepts_single_layer_array() {
    // layer_count = 1 — degenerate array, still 2D in practice.
    let bytes = build_header_bytes(0, 1, 1);
    let header = ktx2::Header::from_bytes(&bytes).expect("synthetic header parses");
    assert!(validate_2d_only(&header).is_ok());
}

#[test]
fn validate_2d_only_rejects_3d_texture() {
    let bytes = build_header_bytes(/* pixel_depth */ 4, 0, 1);
    let header = ktx2::Header::from_bytes(&bytes).expect("synthetic header parses");
    let err = validate_2d_only(&header).expect_err("3D must reject");
    assert!(matches!(
        err,
        Ktx2Error::UnsupportedDimensionality { reason } if reason.contains("3D")
    ));
}

#[test]
fn validate_2d_only_rejects_cubemap() {
    let bytes = build_header_bytes(0, 0, /* face_count */ 6);
    let header = ktx2::Header::from_bytes(&bytes).expect("synthetic header parses");
    let err = validate_2d_only(&header).expect_err("cubemap must reject");
    assert!(matches!(
        err,
        Ktx2Error::UnsupportedDimensionality { reason } if reason.contains("cubemap")
    ));
}

#[test]
fn validate_2d_only_rejects_texture_array() {
    let bytes = build_header_bytes(0, /* layer_count */ 8, 1);
    let header = ktx2::Header::from_bytes(&bytes).expect("synthetic header parses");
    let err = validate_2d_only(&header).expect_err("array must reject");
    assert!(matches!(
        err,
        Ktx2Error::UnsupportedDimensionality { reason } if reason.contains("array")
    ));
}

// ── full-file synthetic fixture builder ────────────────────────
//
// The upstream `ktx2` crate exposes enough public API to build a
// fully spec-compliant KTX2 in memory: `Basic::from_format` for
// the DFD, `Header::as_bytes` + `LevelIndex::as_bytes` for the
// layout, and `Block::to_vec` to serialise the DFD block. The
// `FixtureSpec` knobs below let each test pinpoint one error
// path or success scenario.

/// Knobs for [`build_fixture`]. Per-test sites flip only the
/// fields they care about; the rest stay at the
/// `valid_rgba8_srgb_1x1()` defaults.
struct FixtureSpec {
    width: u32,
    height: u32,
    pixel_depth: u32,
    layer_count: u32,
    face_count: u32,
    /// `(payload, declared_uncompressed_byte_length)` per level.
    /// `None` declared-length = use `payload.len()` (the well-
    /// formed case). Forcing a mismatch drives the
    /// `LevelDataMismatch` path.
    levels: Vec<(Vec<u8>, Option<u64>)>,
    /// Overwrite header bytes 12..16 (raw VkFormat). `None` keeps
    /// the typed `Format::R8G8B8A8_SRGB`. `Some(0)` triggers
    /// `MissingFormat`.
    raw_format_override: Option<u32>,
    /// Overwrite header bytes 44..48 (raw supercompression
    /// scheme). `None` keeps zero (uncompressed). `Some(2)`
    /// drives the `UnsupportedSupercompression` path (zstd).
    raw_supercompression_override: Option<u32>,
    /// W1.T9 audit Lente ξ-F1 — `keyValueData` entries to emit as a
    /// real KVD section between the DFD and the level data. Empty =
    /// `kvd_byte_length = 0` (the pre-audit behaviour). Drives the
    /// real parse path for `MAX_KVD_ENTRIES` / `MAX_KVD_VALUE_BYTES`
    /// rejection and `PH2D_PREMUL` round-trips.
    kvd_entries: Vec<(String, Vec<u8>)>,
}

impl FixtureSpec {
    /// Canonical valid file: 2D RGBA8_SRGB, 1×1, 1 mip, hot magenta.
    fn valid_rgba8_srgb_1x1() -> Self {
        Self {
            width: 1,
            height: 1,
            pixel_depth: 0,
            layer_count: 0,
            face_count: 1,
            levels: vec![(vec![0xFF, 0x00, 0x80, 0xFF], None)],
            raw_format_override: None,
            raw_supercompression_override: None,
            kvd_entries: Vec::new(),
        }
    }
}

/// W1.T9 audit Lente ξ-F1 — serialize KVD entries into the on-disk
/// KTX2 `keyValueData` layout the `ktx2` reader expects: per entry a
/// `u32` LE `keyAndValueByteLength` (= key + NUL + value), then the
/// NUL-terminated UTF-8 key, then the value bytes, then zero-padding
/// to the next 4-byte boundary. Mirrors `KeyValueDataIterator::next`.
fn build_kvd_section(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for (key, value) in entries {
        let key_bytes = key.as_bytes();
        let kv_len = key_bytes.len() + 1 + value.len();
        out.extend_from_slice(&(kv_len as u32).to_le_bytes());
        out.extend_from_slice(key_bytes);
        out.push(0u8);
        out.extend_from_slice(value);
        while !out.len().is_multiple_of(4) {
            out.push(0u8);
        }
    }
    out
}

/// Construct a KTX2 byte buffer following `spec`. Tests must not
/// pass malformed specs (e.g. `levels.is_empty()`) — only
/// well-known shapes that exercise the decoder's branches.
fn build_fixture(spec: &FixtureSpec) -> Vec<u8> {
    use ktx2::dfd::{Basic, Block};
    use ktx2::{Format, Header, Index, LevelIndex};

    let (basic, type_size) =
        Basic::from_format(Format::R8G8B8A8_SRGB).expect("R8G8B8A8_SRGB is a known format");
    let block_bytes = Block::Basic(basic).to_vec();
    let dfd_total_size: u32 = u32::try_from(4 + block_bytes.len()).expect("DFD fits in u32");
    let mut dfd_section = Vec::with_capacity(dfd_total_size as usize);
    dfd_section.extend_from_slice(&dfd_total_size.to_le_bytes());
    dfd_section.extend_from_slice(&block_bytes);

    let level_count = spec.levels.len() as u32;
    let level_index_offset: u32 = 80;
    let level_index_len: u32 = level_count * 24;
    let dfd_byte_offset: u32 = level_index_offset + level_index_len;
    let dfd_byte_length: u32 = u32::try_from(dfd_section.len()).expect("DFD len fits in u32");
    // W1.T9 audit Lente ξ-F1 — optional KVD section sits between the
    // DFD and the level data (KTX2 layout order). 4-byte aligned
    // start; `kvd_byte_offset = kvd_byte_length = 0` when there are
    // no entries (pre-audit behaviour for every existing fixture).
    let kvd_section = build_kvd_section(&spec.kvd_entries);
    let (kvd_byte_offset, kvd_byte_length): (u32, u32) = if kvd_section.is_empty() {
        (0, 0)
    } else {
        let off = (dfd_byte_offset + dfd_byte_length + 3) & !3;
        (
            off,
            u32::try_from(kvd_section.len()).expect("KVD len fits in u32"),
        )
    };

    // Level data is aligned to lcm(4, texel_block_size); for the
    // RGBA8 / BC* / ASTC sizes we test, 4 is always a safe LCM
    // multiple. `Reader::new` also enforces `kvd_end < input.len()`
    // (and `dfd_end < input.len()` when there is no KVD) strictly —
    // so we always leave at least 1 byte of slack between the last
    // metadata section and the first level (rounding `+ 4` down to
    // the next multiple of 4 satisfies both invariants even when
    // every payload is empty).
    let metadata_end = if kvd_section.is_empty() {
        dfd_byte_offset + dfd_byte_length
    } else {
        kvd_byte_offset + kvd_byte_length
    };
    let mut level_data_offset = (metadata_end + 4) & !3;

    // Pre-compute each level's stored byte offset + payload size,
    // tracking running offset for the level index.
    let mut level_offsets: Vec<(u64, u64, u64)> = Vec::with_capacity(spec.levels.len());
    for (payload, declared_override) in &spec.levels {
        let byte_length = payload.len() as u64;
        let declared = declared_override.unwrap_or(byte_length);
        level_offsets.push((u64::from(level_data_offset), byte_length, declared));
        // Each level aligned to 4 (KTX2 mip-padding).
        let next = (level_data_offset + payload.len() as u32 + 3) & !3;
        level_data_offset = next;
    }
    let total_len = level_data_offset as usize;

    let header = Header {
        format: Some(Format::R8G8B8A8_SRGB),
        type_size,
        pixel_width: spec.width,
        pixel_height: spec.height,
        pixel_depth: spec.pixel_depth,
        layer_count: spec.layer_count,
        face_count: spec.face_count,
        level_count,
        supercompression_scheme: None,
        index: Index {
            dfd_byte_offset,
            dfd_byte_length,
            kvd_byte_offset,
            kvd_byte_length,
            sgd_byte_offset: 0,
            sgd_byte_length: 0,
        },
    };

    let mut buf = vec![0u8; total_len];
    buf[0..80].copy_from_slice(&header.as_bytes());

    // Level index — one 24-byte entry per level.
    for (i, &(byte_offset, byte_length, declared)) in level_offsets.iter().enumerate() {
        let entry = LevelIndex {
            byte_offset,
            byte_length,
            uncompressed_byte_length: declared,
        };
        let start = 80 + i * 24;
        buf[start..start + 24].copy_from_slice(&entry.as_bytes());
    }

    // DFD section.
    buf[dfd_byte_offset as usize..(dfd_byte_offset + dfd_byte_length) as usize]
        .copy_from_slice(&dfd_section);

    // KVD section (W1.T9 audit Lente ξ-F1), when present.
    if !kvd_section.is_empty() {
        let start = kvd_byte_offset as usize;
        buf[start..start + kvd_section.len()].copy_from_slice(&kvd_section);
    }

    // Each level's payload at its declared offset.
    for ((payload, _), &(byte_offset, _, _)) in spec.levels.iter().zip(&level_offsets) {
        let start = byte_offset as usize;
        buf[start..start + payload.len()].copy_from_slice(payload);
    }

    // Apply byte-level overrides AFTER `Header::as_bytes` wrote
    // the typed fields — the only way to forge values the typed
    // `Header` constructor refuses to represent.
    if let Some(raw) = spec.raw_format_override {
        buf[12..16].copy_from_slice(&raw.to_le_bytes());
    }
    if let Some(raw) = spec.raw_supercompression_override {
        buf[44..48].copy_from_slice(&raw.to_le_bytes());
    }

    buf
}

// ── positive round-trips ──────────────────────────────────────

#[test]
fn decode_synthetic_rgba8_1x1_round_trips() {
    let bytes = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let image = decode_ktx2_bytes(&bytes).expect("valid synthetic file decodes");

    assert_eq!(image.format, Ktx2Format::Rgba8UnormSrgb);
    assert_eq!(image.width, 1);
    assert_eq!(image.height, 1);
    assert_eq!(image.mip_levels.len(), 1);
    assert_eq!(image.base_level().data.as_ref(), &[0xFF, 0x00, 0x80, 0xFF]);
}

#[test]
fn decode_synthetic_format_classifies_correctly() {
    let bytes = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let image = decode_ktx2_bytes(&bytes).expect("decodes");
    assert!(!image.format.is_compressed());
    assert!(!image.format.is_hdr());
}

#[test]
fn decode_synthetic_4x4_with_three_mips_round_trips() {
    // 4×4 base, 2×2 mip1, 1×1 mip2. Each level filled with a
    // distinct byte pattern so a mip-order bug shows up in the
    // assertions.
    let mip0 = vec![0xAA; 4 * 4 * 4]; // 4×4 × RGBA8 = 64 B
    let mip1 = vec![0xBB; 2 * 2 * 4]; // 16 B
    let mip2 = vec![0xCC; 4]; // 1×1 × RGBA8 = 4 B
    let spec = FixtureSpec {
        width: 4,
        height: 4,
        levels: vec![
            (mip0.clone(), None),
            (mip1.clone(), None),
            (mip2.clone(), None),
        ],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let image = decode_ktx2_bytes(&bytes).expect("3-mip RGBA8 decodes");

    assert_eq!(image.width, 4);
    assert_eq!(image.height, 4);
    assert_eq!(image.mip_levels.len(), 3);
    assert_eq!(
        (image.mip_levels[0].width, image.mip_levels[0].height),
        (4, 4)
    );
    assert_eq!(
        (image.mip_levels[1].width, image.mip_levels[1].height),
        (2, 2)
    );
    assert_eq!(
        (image.mip_levels[2].width, image.mip_levels[2].height),
        (1, 1)
    );
    assert_eq!(image.mip_levels[0].data.as_ref(), mip0.as_slice());
    assert_eq!(image.mip_levels[1].data.as_ref(), mip1.as_slice());
    assert_eq!(image.mip_levels[2].data.as_ref(), mip2.as_slice());
}

#[test]
fn decode_synthetic_npot_5x3_mip_rounding() {
    // 5×3 has the awkward halving: mip1 = max(1, 5>>1) × max(1, 3>>1)
    // = 2×1, mip2 = max(1, 5>>2) × max(1, 3>>2) = 1×1.
    let mip0 = vec![0x11; 5 * 3 * 4]; // 60 B
    let mip1 = vec![0x22; 2 * 4]; // 2×1 × RGBA8 = 8 B
    let mip2 = vec![0x33; 4]; // 1×1 × RGBA8 = 4 B
    let spec = FixtureSpec {
        width: 5,
        height: 3,
        levels: vec![(mip0, None), (mip1, None), (mip2, None)],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let image = decode_ktx2_bytes(&bytes).expect("NPOT 5×3 decodes");

    assert_eq!(image.mip_levels.len(), 3);
    assert_eq!(
        (image.mip_levels[0].width, image.mip_levels[0].height),
        (5, 3)
    );
    assert_eq!(
        (image.mip_levels[1].width, image.mip_levels[1].height),
        (2, 1)
    );
    assert_eq!(
        (image.mip_levels[2].width, image.mip_levels[2].height),
        (1, 1)
    );
}

#[test]
fn base_level_returns_mip_zero() {
    // Confirms the ergonomic accessor matches `mip_levels[0]`.
    let bytes = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let image = decode_ktx2_bytes(&bytes).unwrap();
    let by_accessor = image.base_level();
    let by_index = &image.mip_levels[0];
    assert_eq!(by_accessor.width, by_index.width);
    assert_eq!(by_accessor.height, by_index.height);
    assert_eq!(by_accessor.data.as_ref(), by_index.data.as_ref());
}

// ── decoder rejection paths via the fixture builder ────────────

#[test]
fn decode_rejects_zero_dimension() {
    // `Header::from_bytes` refuses pixel_width == 0, but pixel_height
    // can be zero (the spec uses it for 1D textures). Our decoder
    // catches it before any other check.
    let spec = FixtureSpec {
        height: 0,
        levels: vec![(Vec::new(), None)],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("zero height must reject");
    assert!(matches!(err, Ktx2Error::ZeroDimension));
}

#[test]
fn decode_rejects_bounds_exceeded_width() {
    let spec = FixtureSpec {
        width: MAX_DIMENSION + 1,
        levels: vec![(vec![0u8; 4], None)],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("oversize width must reject");
    assert!(
        matches!(err, Ktx2Error::BoundsExceeded { dim, max }
            if dim == MAX_DIMENSION + 1 && max == MAX_DIMENSION),
        "got {err:?}",
    );
}

#[test]
fn decode_rejects_bounds_exceeded_height() {
    let spec = FixtureSpec {
        height: MAX_DIMENSION + 1,
        levels: vec![(vec![0u8; 4], None)],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("oversize height must reject");
    assert!(matches!(err, Ktx2Error::BoundsExceeded { .. }));
}

#[test]
fn decode_rejects_too_many_levels() {
    // 17 levels (MAX_LEVELS = 16). Each payload is empty so the
    // file stays tiny; the cap fires before any payload is read.
    let levels = (0..(MAX_LEVELS + 1))
        .map(|_| (Vec::new(), None))
        .collect::<Vec<_>>();
    let spec = FixtureSpec {
        levels,
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("too many levels must reject");
    assert!(
        matches!(err, Ktx2Error::TooManyLevels { count, max }
            if count == MAX_LEVELS + 1 && max == MAX_LEVELS),
        "got {err:?}",
    );
}

#[test]
fn decode_rejects_missing_format() {
    // Override the raw VkFormat field to 0 (VK_FORMAT_UNDEFINED).
    // The container is otherwise valid (DFD is still RGBA8_SRGB
    // shaped), but our decoder demands an explicit format.
    let spec = FixtureSpec {
        raw_format_override: Some(0),
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("VK_FORMAT_UNDEFINED must reject");
    assert!(matches!(err, Ktx2Error::MissingFormat), "got {err:?}");
}

#[test]
fn decode_rejects_supercompression_zstd() {
    // Scheme 2 = zstd per the KTX2 spec.
    let spec = FixtureSpec {
        raw_supercompression_override: Some(2),
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("zstd SS must reject");
    assert!(
        matches!(err, Ktx2Error::UnsupportedSupercompression { raw: 2 }),
        "got {err:?}",
    );
}

#[test]
fn decode_rejects_level_data_mismatch() {
    // Payload is 4 bytes but the level index claims 5.
    let spec = FixtureSpec {
        levels: vec![(vec![0xFF, 0x00, 0x80, 0xFF], Some(5))],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("declared/actual mismatch must reject");
    assert!(
        matches!(err, Ktx2Error::LevelDataMismatch { level: 0 }),
        "got {err:?}",
    );
}

// ── defensive guards NOT covered by unit tests ────────────────
//
// `Ktx2Error::TotalBytesExceeded` triggers only if cumulative
// mip payloads exceed 512 MiB — exercising it would require a
// ~512 MiB allocation in test, which violates the slow-test
// policy. The guard is one `saturating_add` plus a compare; it
// is verified by inspection.

// ── W1.T9 audit Lente ξ-F1 — kvd parse-path coverage ──────────
//
// These exercise the REAL decode path (synthetic KTX2 bytes with a
// populated KVD section), not struct literals. The whole bounds
// verdict rests on the count→size→alloc ordering in the parse loop;
// a future refactor that inverts it is now caught by CI.

#[test]
fn decode_fixture_with_kvd_round_trips() {
    let spec = FixtureSpec {
        kvd_entries: vec![
            ("KTXorientation".to_string(), b"rd".to_vec()),
            ("KTXswizzle".to_string(), b"rgba".to_vec()),
        ],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let image = decode_ktx2_bytes(&bytes).expect("valid kvd fixture decodes");
    assert_eq!(image.kvd.len(), 2);
    assert_eq!(
        image.kvd.get("KTXorientation").map(Vec::as_slice),
        Some(&b"rd"[..])
    );
    assert_eq!(
        image.kvd.get("KTXswizzle").map(Vec::as_slice),
        Some(&b"rgba"[..])
    );
}

#[test]
fn decode_kvd_premul_round_trips_end_to_end() {
    // PH2D_PREMUL=1 through the real parser must surface as
    // Premultiplied (not just the struct-literal helper test).
    let spec = FixtureSpec {
        kvd_entries: vec![(PH2D_PREMUL_KEY.to_string(), vec![1u8])],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let image = decode_ktx2_bytes(&bytes).expect("premul kvd fixture decodes");
    assert_eq!(image.premul_intent(), PremulIntent::Premultiplied);
}

#[test]
fn decode_rejects_too_many_kvd_entries() {
    // MAX_KVD_ENTRIES = 64; the 65th entry must be rejected BEFORE
    // it is inserted (count check precedes the map insert).
    let entries: Vec<(String, Vec<u8>)> = (0..(MAX_KVD_ENTRIES + 1))
        .map(|i| (format!("PH2D_K{i:03}"), vec![0u8]))
        .collect();
    let spec = FixtureSpec {
        kvd_entries: entries,
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("over-count kvd must reject");
    assert!(
        matches!(
            err,
            Ktx2Error::TooManyKvdEntries { count, max }
                if count == MAX_KVD_ENTRIES + 1 && max == MAX_KVD_ENTRIES
        ),
        "got {err:?}",
    );
}

#[test]
fn decode_rejects_oversized_kvd_value() {
    // A value 1 byte over MAX_KVD_VALUE_BYTES is rejected BEFORE the
    // `value.to_vec()` allocation (size check precedes the copy).
    let big = vec![0xABu8; MAX_KVD_VALUE_BYTES + 1];
    let spec = FixtureSpec {
        kvd_entries: vec![("PH2D_BLOB".to_string(), big)],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("oversized kvd value must reject");
    assert!(
        matches!(
            err,
            Ktx2Error::KvdValueTooLarge { ref key, size, max }
                if key == "PH2D_BLOB" && size == MAX_KVD_VALUE_BYTES + 1 && max == MAX_KVD_VALUE_BYTES
        ),
        "got {err:?}",
    );
}

#[test]
fn decode_accepts_kvd_value_at_exact_cap() {
    // Boundary: a value of exactly MAX_KVD_VALUE_BYTES is allowed
    // (the check is `>`, not `>=`).
    let at_cap = vec![0x5Au8; MAX_KVD_VALUE_BYTES];
    let spec = FixtureSpec {
        kvd_entries: vec![("PH2D_EDGE".to_string(), at_cap.clone())],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let image = decode_ktx2_bytes(&bytes).expect("at-cap kvd value decodes");
    assert_eq!(
        image.kvd.get("PH2D_EDGE").map(Vec::len),
        Some(MAX_KVD_VALUE_BYTES)
    );
}

#[test]
fn decode_rejects_too_many_duplicate_kvd_keys() {
    // ξ-F2 regression: MAX_KVD_ENTRIES+1 entries that all share ONE
    // key. The dedup'd `kvd.len()` would stay at 1 forever — only the
    // iteration counter catches the metadata-bloat / churn attack.
    let entries: Vec<(String, Vec<u8>)> = (0..(MAX_KVD_ENTRIES + 1))
        .map(|_| ("PH2D_DUP".to_string(), vec![0u8]))
        .collect();
    let spec = FixtureSpec {
        kvd_entries: entries,
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("duplicate-key flood must reject");
    assert!(
        matches!(
            err,
            Ktx2Error::TooManyKvdEntries { count, max }
                if count == MAX_KVD_ENTRIES + 1 && max == MAX_KVD_ENTRIES
        ),
        "got {err:?}",
    );
}

#[test]
fn decode_rejects_oversized_kvd_key() {
    // ξ-F3: a key 1 byte over MAX_KVD_KEY_BYTES is rejected before the
    // `key.to_string()` allocation; the error carries size, not key.
    let long_key = "K".repeat(MAX_KVD_KEY_BYTES + 1);
    let spec = FixtureSpec {
        kvd_entries: vec![(long_key, b"v".to_vec())],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let bytes = build_fixture(&spec);
    let err = decode_ktx2_bytes(&bytes).expect_err("oversized kvd key must reject");
    assert!(
        matches!(
            err,
            Ktx2Error::KvdKeyTooLong { size, max }
                if size == MAX_KVD_KEY_BYTES + 1 && max == MAX_KVD_KEY_BYTES
        ),
        "got {err:?}",
    );
}

// ── W1.T9 — kvd preservation + helper API (struct-literal tests) ──

fn ktx_image_with_kvd(kvd: BTreeMap<String, Vec<u8>>) -> Ktx2Image {
    Ktx2Image {
        format: Ktx2Format::Rgba8UnormSrgb,
        width: 1,
        height: 1,
        mip_levels: vec![MipLevel {
            width: 1,
            height: 1,
            data: Arc::<[u8]>::from(&[0u8; 4][..]),
        }],
        kvd,
    }
}

#[test]
fn premul_intent_unspecified_when_kvd_empty() {
    let img = ktx_image_with_kvd(BTreeMap::new());
    assert_eq!(img.premul_intent(), PremulIntent::Unspecified);
}

#[test]
fn premul_intent_straight_for_value_zero() {
    let mut kvd = BTreeMap::new();
    kvd.insert(PH2D_PREMUL_KEY.to_string(), vec![0u8]);
    let img = ktx_image_with_kvd(kvd);
    assert_eq!(img.premul_intent(), PremulIntent::Straight);
}

#[test]
fn premul_intent_premultiplied_for_value_one() {
    let mut kvd = BTreeMap::new();
    kvd.insert(PH2D_PREMUL_KEY.to_string(), vec![1u8]);
    let img = ktx_image_with_kvd(kvd);
    assert_eq!(img.premul_intent(), PremulIntent::Premultiplied);
}

#[test]
fn premul_intent_unspecified_for_invalid_value() {
    // Wildcard match arm: any value não-[0]/[1] (multi-byte, [2], [255])
    // degrade graciosamente para Unspecified (não panic, não erro).
    for value in [vec![2u8], vec![255u8], vec![0u8, 1u8], vec![]] {
        let mut kvd = BTreeMap::new();
        kvd.insert(PH2D_PREMUL_KEY.to_string(), value.clone());
        let img = ktx_image_with_kvd(kvd);
        assert_eq!(
            img.premul_intent(),
            PremulIntent::Unspecified,
            "value {value:?} should yield Unspecified"
        );
    }
}

#[test]
fn premul_intent_unspecified_when_other_keys_present() {
    // Other kvd keys não devem afetar premul_intent — só PH2D_PREMUL.
    let mut kvd = BTreeMap::new();
    kvd.insert("KTXswizzle".to_string(), b"rgba".to_vec());
    kvd.insert("KTXorientation".to_string(), b"rd".to_vec());
    let img = ktx_image_with_kvd(kvd);
    assert_eq!(img.premul_intent(), PremulIntent::Unspecified);
}

#[test]
fn byte_size_estimate_sums_mip_payloads() {
    let img = Ktx2Image {
        format: Ktx2Format::Rgba8UnormSrgb,
        width: 4,
        height: 4,
        mip_levels: vec![
            MipLevel {
                width: 4,
                height: 4,
                data: Arc::<[u8]>::from(&[0u8; 64][..]), // 4×4×4 = 64
            },
            MipLevel {
                width: 2,
                height: 2,
                data: Arc::<[u8]>::from(&[0u8; 16][..]), // 2×2×4 = 16
            },
            MipLevel {
                width: 1,
                height: 1,
                data: Arc::<[u8]>::from(&[0u8; 4][..]), // 1×1×4 = 4
            },
        ],
        kvd: BTreeMap::new(),
    };
    assert_eq!(img.byte_size_estimate(), 64 + 16 + 4);
}

#[test]
fn byte_size_estimate_single_level() {
    let img = ktx_image_with_kvd(BTreeMap::new());
    assert_eq!(img.byte_size_estimate(), 4); // 1×1×4 RGBA8
}

/// Parser smoke: the canonical fixture has no kvd entries →
/// `image.kvd` is empty (`kvd_byte_length = 0` path). Populated-kvd
/// parse coverage now lives in the `ξ-F1` tests above.
#[test]
fn decode_fixture_has_empty_kvd() {
    let bytes = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let image = decode_ktx2_bytes(&bytes).expect("valid file decodes");
    assert!(image.kvd.is_empty());
}

// ── W1.T8.1 post-hoc PH2D_PREMUL patcher ───────────────────────

/// `serialize_kvd` matches the test fixture's `build_kvd_section`
/// byte-for-byte: both implement the same on-disk KVD layout, so a
/// drift between them would silently corrupt one of the two paths.
#[test]
fn serialize_kvd_matches_fixture_builder() {
    let entries = vec![
        ("KTXorientation".to_string(), b"rd".to_vec()),
        (PH2D_PREMUL_KEY.to_string(), vec![1u8]),
    ];
    assert_eq!(serialize_kvd(&entries), build_kvd_section(&entries));
}

/// Patch into a file with NO existing KVD, then re-parse: the key is
/// present, `premul_intent()` reads back the patched value, and the
/// mip-0 payload is byte-identical to the original.
#[test]
fn patch_into_empty_kvd_round_trips_premultiplied() {
    let orig = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let orig_img = decode_ktx2_bytes(&orig).expect("orig decodes");
    assert_eq!(orig_img.premul_intent(), PremulIntent::Unspecified);

    let patched = patch_premul_intent(&orig, PremulIntent::Premultiplied).expect("patch succeeds");
    let img = decode_ktx2_bytes(&patched).expect("patched file still decodes");

    assert_eq!(img.premul_intent(), PremulIntent::Premultiplied);
    assert_eq!(
        img.kvd.get(PH2D_PREMUL_KEY).map(Vec::as_slice),
        Some(&[1u8][..])
    );
    // Mip data preserved byte-for-byte through the shift.
    assert_eq!(
        img.base_level().data.as_ref(),
        orig_img.base_level().data.as_ref(),
    );
    assert_eq!(img.width, orig_img.width);
    assert_eq!(img.height, orig_img.height);
    assert_eq!(img.format, orig_img.format);
}

#[test]
fn patch_straight_round_trips() {
    let orig = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let patched = patch_premul_intent(&orig, PremulIntent::Straight).expect("patch succeeds");
    let img = decode_ktx2_bytes(&patched).expect("decodes");
    assert_eq!(img.premul_intent(), PremulIntent::Straight);
    assert_eq!(
        img.kvd.get(PH2D_PREMUL_KEY).map(Vec::as_slice),
        Some(&[0u8][..])
    );
}

/// Patch into a file that ALREADY has unrelated KVD entries: the
/// existing entries survive and the merged set comes back sorted by
/// key (KTX2 codepoint-order requirement).
#[test]
fn patch_preserves_existing_kvd_and_sorts() {
    // "KTXorientation" < "KTXwriter" < "PH2D_PREMUL" lexicographically;
    // pass them out of order to prove the patcher re-sorts.
    let mut spec = FixtureSpec::valid_rgba8_srgb_1x1();
    spec.kvd_entries = vec![
        ("KTXwriter".to_string(), b"toktx v4".to_vec()),
        ("KTXorientation".to_string(), b"rd".to_vec()),
    ];
    let orig = build_fixture(&spec);

    let patched = patch_premul_intent(&orig, PremulIntent::Premultiplied).expect("patch succeeds");
    let img = decode_ktx2_bytes(&patched).expect("decodes");

    assert_eq!(img.premul_intent(), PremulIntent::Premultiplied);
    assert_eq!(
        img.kvd.get("KTXwriter").map(Vec::as_slice),
        Some(&b"toktx v4"[..])
    );
    assert_eq!(
        img.kvd.get("KTXorientation").map(Vec::as_slice),
        Some(&b"rd"[..])
    );
    assert_eq!(img.kvd.len(), 3);

    // Re-parse the RAW on-disk KVD order via the ktx2 reader: keys
    // must be in codepoint order (the patcher re-sorts before write).
    let reader = ktx2::Reader::new(&patched[..]).expect("reader parses patched bytes");
    let keys: Vec<String> = reader
        .key_value_data()
        .map(|(k, _)| k.to_string())
        .collect();
    assert_eq!(keys, vec!["KTXorientation", "KTXwriter", "PH2D_PREMUL"]);
}

/// Insert-only contract: a second patch errors with the existing
/// intent rather than silently overwriting.
#[test]
fn patch_is_insert_only_and_reports_existing() {
    let orig = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let once = patch_premul_intent(&orig, PremulIntent::Straight).expect("first patch ok");
    let err = patch_premul_intent(&once, PremulIntent::Premultiplied)
        .expect_err("second patch must error");
    match err {
        Ktx2PatchError::KeyAlreadyPresent { existing } => {
            assert_eq!(existing, PremulIntent::Straight);
        }
        other => panic!("expected KeyAlreadyPresent, got {other:?}"),
    }
}

#[test]
fn patch_rejects_unspecified() {
    let orig = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let err = patch_premul_intent(&orig, PremulIntent::Unspecified)
        .expect_err("Unspecified has no encoding");
    assert!(matches!(err, Ktx2PatchError::UnspecifiedIntent));
}

#[test]
fn patch_rejects_garbage_input() {
    let err = patch_premul_intent(&[0u8; 32], PremulIntent::Straight)
        .expect_err("non-KTX2 input rejected");
    assert!(matches!(err, Ktx2PatchError::InvalidContainer(_)));
}

/// Multi-mip file: every level's payload survives the offset shift
/// byte-for-byte, the level offsets stay 4-byte aligned, and the
/// patched container re-parses cleanly through the ktx2 reader.
#[test]
fn patch_preserves_all_mip_data_and_alignment() {
    let mip0 = vec![0xAAu8; 4 * 4 * 4]; // 64 B
    let mip1 = vec![0xBBu8; 2 * 2 * 4]; // 16 B
    let mip2 = vec![0xCCu8; 4]; // 4 B
    let spec = FixtureSpec {
        width: 4,
        height: 4,
        levels: vec![
            (mip0.clone(), None),
            (mip1.clone(), None),
            (mip2.clone(), None),
        ],
        ..FixtureSpec::valid_rgba8_srgb_1x1()
    };
    let orig = build_fixture(&spec);

    let patched = patch_premul_intent(&orig, PremulIntent::Premultiplied).expect("patch succeeds");

    // ktx2 reader accepts the patched file and reports valid bounds.
    let reader = ktx2::Reader::new(&patched[..]).expect("patched parses via ktx2");
    let levels: Vec<&[u8]> = reader.levels().map(|l| l.data).collect();
    assert_eq!(levels.len(), 3);
    assert_eq!(levels[0], &mip0[..]);
    assert_eq!(levels[1], &mip1[..]);
    assert_eq!(levels[2], &mip2[..]);

    // Every level offset is 4-byte aligned (KTX2 mip-padding for RGBA8).
    let raw = parse_level_index(&patched, 3).expect("level index parses");
    for (off, _) in raw {
        assert_eq!(off % 4, 0, "level offset {off} not 4-aligned");
    }

    // Full decode path agrees on the data + tag.
    let img = decode_ktx2_bytes(&patched).expect("decode ok");
    assert_eq!(img.premul_intent(), PremulIntent::Premultiplied);
    assert_eq!(img.mip_levels.len(), 3);
    assert_eq!(img.mip_levels[0].data.as_ref(), &mip0[..]);
    assert_eq!(img.mip_levels[2].data.as_ref(), &mip2[..]);
}

/// The patched KVD section length is always a multiple of 4 (every
/// entry is value-padded), and the header's recorded kvdByteLength
/// matches the section we wrote.
#[test]
fn patched_kvd_length_is_4_aligned_and_recorded() {
    let orig = build_fixture(&FixtureSpec::valid_rgba8_srgb_1x1());
    let patched = patch_premul_intent(&orig, PremulIntent::Straight).expect("patch ok");
    let reader = ktx2::Reader::new(&patched[..]).expect("parses");
    let idx = reader.header().index;
    assert_eq!(
        idx.kvd_byte_length % 4,
        0,
        "kvdByteLength must be 4-aligned"
    );
    assert_ne!(idx.kvd_byte_length, 0, "kvd now non-empty");
    assert_eq!(
        idx.kvd_byte_offset % 4,
        0,
        "kvdByteOffset must be 4-aligned"
    );
}

#[test]
fn encode_premul_value_mapping() {
    assert_eq!(encode_premul_value(PremulIntent::Straight), Some(0));
    assert_eq!(encode_premul_value(PremulIntent::Premultiplied), Some(1));
    assert_eq!(encode_premul_value(PremulIntent::Unspecified), None);
}

/// Build a minimal valid 1×1 RGBA8 KTX2 carrying a non-empty SGD
/// section (verbatim bytes) so the patcher's SGD-shift + align(8) path
/// is exercised. The standard `build_fixture` always emits
/// `sgd_byte_length = 0`; cooked PH2D textures never carry SGD (no
/// supercompression), so this lives only in the patcher tests.
fn build_fixture_with_sgd(sgd: &[u8]) -> Vec<u8> {
    use ktx2::dfd::{Basic, Block};
    use ktx2::{Format, Header, Index, LevelIndex};

    let (basic, type_size) = Basic::from_format(Format::R8G8B8A8_SRGB).expect("known format");
    let block_bytes = Block::Basic(basic).to_vec();
    let dfd_total: u32 = u32::try_from(4 + block_bytes.len()).unwrap();
    let mut dfd = Vec::new();
    dfd.extend_from_slice(&dfd_total.to_le_bytes());
    dfd.extend_from_slice(&block_bytes);

    let payload = [0xDEu8, 0xAD, 0xBE, 0xEF]; // 1×1 RGBA8

    let level_index_off: u32 = 80;
    let level_index_len: u32 = 24; // one level
    let dfd_off = level_index_off + level_index_len;
    let dfd_len = u32::try_from(dfd.len()).unwrap();
    // SGD on the next align(8) boundary after the DFD (no KVD here).
    let sgd_off = (dfd_off + dfd_len + 7) & !7;
    let sgd_len = u32::try_from(sgd.len()).unwrap();
    // Level data on the next align(4) past SGD, +4 slack so the
    // reader's strict `sgd_end < len` invariant holds.
    let level_off = (sgd_off + sgd_len + 4) & !3;
    let total = (level_off + payload.len() as u32) as usize;

    let header = Header {
        format: Some(Format::R8G8B8A8_SRGB),
        type_size,
        pixel_width: 1,
        pixel_height: 1,
        pixel_depth: 0,
        layer_count: 0,
        face_count: 1,
        level_count: 1,
        supercompression_scheme: None,
        index: Index {
            dfd_byte_offset: dfd_off,
            dfd_byte_length: dfd_len,
            kvd_byte_offset: 0,
            kvd_byte_length: 0,
            sgd_byte_offset: u64::from(sgd_off),
            sgd_byte_length: u64::from(sgd_len),
        },
    };

    let mut buf = vec![0u8; total];
    buf[0..80].copy_from_slice(&header.as_bytes());
    let li = LevelIndex {
        byte_offset: u64::from(level_off),
        byte_length: payload.len() as u64,
        uncompressed_byte_length: payload.len() as u64,
    };
    buf[80..104].copy_from_slice(&li.as_bytes());
    buf[dfd_off as usize..(dfd_off + dfd_len) as usize].copy_from_slice(&dfd);
    buf[sgd_off as usize..(sgd_off + sgd_len) as usize].copy_from_slice(sgd);
    buf[level_off as usize..level_off as usize + payload.len()].copy_from_slice(&payload);
    buf
}

/// Patching a file WITH an SGD section: the SGD bytes survive verbatim,
/// the new sgdByteOffset is 8-aligned, the level data is intact, and
/// the premul tag round-trips.
#[test]
fn patch_preserves_sgd_section_and_aligns_it() {
    let sgd = [0x11u8, 0x22, 0x33, 0x44, 0x55]; // 5 bytes (non-8-multiple)
    let orig = build_fixture_with_sgd(&sgd);
    // Sanity: original parses and carries our SGD.
    let r0 = ktx2::Reader::new(&orig[..]).expect("orig parses");
    assert_eq!(r0.supercompression_global_data(), &sgd[..]);

    let patched =
        patch_premul_intent(&orig, PremulIntent::Premultiplied).expect("patch with SGD succeeds");
    let r1 = ktx2::Reader::new(&patched[..]).expect("patched parses");

    // SGD preserved byte-for-byte.
    assert_eq!(r1.supercompression_global_data(), &sgd[..]);
    // sgdByteOffset 8-aligned (spec sgdPadding), length unchanged.
    let idx = r1.header().index;
    assert_eq!(
        idx.sgd_byte_offset % 8,
        0,
        "sgdByteOffset must be 8-aligned"
    );
    assert_eq!(idx.sgd_byte_length, sgd.len() as u64);
    // Level + tag round-trip.
    let img = decode_ktx2_bytes(&patched).expect("decode ok");
    assert_eq!(img.premul_intent(), PremulIntent::Premultiplied);
    assert_eq!(img.base_level().data.as_ref(), &[0xDE, 0xAD, 0xBE, 0xEF]);
}

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    Ktx2Error, Ktx2Format, Ktx2Image, MAX_DIMENSION, MAX_KVD_ENTRIES, MAX_KVD_KEY_BYTES,
    MAX_KVD_VALUE_BYTES, MAX_LEVELS, MAX_TOTAL_BYTES, MipLevel,
};

// ── decode entry point ──────────────────────────────────────────────

/// Reject KTX2 layouts that the Fase 1 renderer cannot consume:
/// 3D textures, cubemaps, and array textures. The KTX2 header encodes
/// these via three independent fields:
///
/// - `pixel_depth > 0` → 3D texture (`pixel_depth == 0` means 2D
///   per spec).
/// - `face_count == 6` → cubemap (spec allows only `1` or `6`).
/// - `layer_count > 1` → texture array (`0` means "not an array",
///   `1` is a degenerate single-layer array we still accept).
///
/// Split out from [`decode_ktx2_bytes`] so unit tests can drive it
/// from a synthetic `ktx2::Header` (via `Header::from_bytes`) without
/// having to fabricate a full KTX2 file (DFD + level index + payload).
pub(crate) fn validate_2d_only(header: &ktx2::Header) -> Result<(), Ktx2Error> {
    if header.pixel_depth > 0 {
        return Err(Ktx2Error::UnsupportedDimensionality {
            reason: "3D texture (pixel_depth > 0)",
        });
    }
    if header.face_count > 1 {
        return Err(Ktx2Error::UnsupportedDimensionality {
            reason: "cubemap (face_count > 1)",
        });
    }
    if header.layer_count > 1 {
        return Err(Ktx2Error::UnsupportedDimensionality {
            reason: "texture array (layer_count > 1)",
        });
    }
    Ok(())
}

/// Parse a `.ktx2` byte buffer into a typed [`Ktx2Image`].
///
/// The buffer is the entire file as read from disk (or any other
/// source — KTX2 is self-contained, no sidecar). On error the buffer
/// is untouched and no partial state escapes.
///
/// # Errors
///
/// See [`Ktx2Error`]. The common causes are: malformed bytes
/// (`InvalidContainer`), oversized header claims (`BoundsExceeded` /
/// `TotalBytesExceeded`), and `UnsupportedSupercompression` for files
/// that artists ship through a chain that compressed them with zstd
/// / BasisLZ.
///
/// # Examples
///
/// Garbage input returns a structured error — no panic:
///
/// ```
/// use ph2d_asset_ktx2::{decode_ktx2_bytes, Ktx2Error};
///
/// let result = decode_ktx2_bytes(&[0u8; 32]);
/// assert!(matches!(result, Err(Ktx2Error::InvalidContainer(_))));
/// ```
///
/// On success, `Ktx2Image::base_level` is the easiest way to reach
/// the full-resolution bytes:
///
/// ```no_run
/// # use ph2d_asset_ktx2::{decode_ktx2_bytes, Ktx2Format};
/// # fn read_file() -> Vec<u8> { unimplemented!() }
/// let bytes = read_file();
/// let image = decode_ktx2_bytes(&bytes).expect("file is a valid KTX2");
/// let base = image.base_level();
/// assert_eq!((base.width, base.height), (image.width, image.height));
/// match image.format {
///     Ktx2Format::Bc7RgbaUnormSrgb => { /* desktop-compressed path */ }
///     Ktx2Format::Rgba8UnormSrgb => { /* uncompressed sRGB path */ }
///     other => panic!("unexpected format {other:?}"),
/// }
/// ```
pub fn decode_ktx2_bytes(bytes: &[u8]) -> Result<Ktx2Image, Ktx2Error> {
    let reader =
        ktx2::Reader::new(bytes).map_err(|e| Ktx2Error::InvalidContainer(format!("{e:?}")))?;

    let header = reader.header();

    if header.pixel_width == 0 || header.pixel_height == 0 {
        return Err(Ktx2Error::ZeroDimension);
    }
    if header.pixel_width > MAX_DIMENSION {
        return Err(Ktx2Error::BoundsExceeded {
            dim: header.pixel_width,
            max: MAX_DIMENSION,
        });
    }
    if header.pixel_height > MAX_DIMENSION {
        return Err(Ktx2Error::BoundsExceeded {
            dim: header.pixel_height,
            max: MAX_DIMENSION,
        });
    }

    // 2D-only enforcement — Fase 1 has no path for 3D / cubemap /
    // array textures in the renderer. Without this guard each of
    // those layouts would silently decode the first plane / face /
    // layer and discard the rest.
    validate_2d_only(&header)?;

    // KTX2 spec: level_count == 0 means "engine generates the mip
    // pyramid" — we treat that as a single level (the base) and let
    // the renderer choose to generate on upload. Otherwise enforce
    // the cap.
    if header.level_count > MAX_LEVELS {
        return Err(Ktx2Error::TooManyLevels {
            count: header.level_count,
            max: MAX_LEVELS,
        });
    }

    // Supercompression: only `None` (raw bytes) is wired in Fase 1.
    // `SupercompressionScheme` is a pseudo-enum (`NonZeroU32` wrapper);
    // `None` ≡ scheme 0 ≡ uncompressed, anything else needs an ADR.
    if let Some(scheme) = header.supercompression_scheme {
        return Err(Ktx2Error::UnsupportedSupercompression {
            raw: scheme.value(),
        });
    }

    // VkFormat == 0 (VK_FORMAT_UNDEFINED) — represented as
    // `header.format == None` — means the file describes pixel layout
    // purely via the Data Format Descriptor. We do not parse the DFD
    // in Fase 1 — reject explicitly so callers know.
    let format_raw = header.format.ok_or(Ktx2Error::MissingFormat)?;
    let format = Ktx2Format::from_vk_format(format_raw.value());

    // Walk the mip index, harvest each level's payload, enforce the
    // total-bytes cap as we go (defensive: a single huge level can
    // still exhaust memory even if `level_count` is small).
    let mut mip_levels: Vec<MipLevel> = Vec::new();
    let mut total_bytes: u64 = 0;

    for (i, level) in reader.levels().enumerate() {
        let level_idx = i as u32;
        let payload: &[u8] = level.data;
        total_bytes = total_bytes.saturating_add(payload.len() as u64);
        if total_bytes > MAX_TOTAL_BYTES {
            return Err(Ktx2Error::TotalBytesExceeded {
                total: total_bytes,
                max: MAX_TOTAL_BYTES,
            });
        }

        // Mip dimensions follow the standard `max(1, base >> i)`
        // halving; KTX2 does not store per-level dimensions because
        // they are derivable.
        //
        // Shift safety (W1.T15 audit Lente ο-O2): `i` is bounded by the
        // `level_count > MAX_LEVELS` reject above (line ~602) and upstream
        // `levels()` yields exactly `level_count.max(1)` items — so `i <= 15`.
        // The compile-time `const _: assert!(MAX_LEVELS < 32)` (top of file)
        // guarantees `i < 32`, so `>> i` can never hit the shift-overflow
        // panic even if MAX_LEVELS is later raised.
        let mip_w = (header.pixel_width >> i).max(1);
        let mip_h = (header.pixel_height >> i).max(1);

        // KTX2 stores `uncompressed_byte_length` (for non-super-
        // compressed files, equal to payload.len()). Cross-check so a
        // malformed index can't silently mis-size a level.
        let declared = level.uncompressed_byte_length;
        if declared != payload.len() as u64 {
            return Err(Ktx2Error::LevelDataMismatch { level: level_idx });
        }

        mip_levels.push(MipLevel {
            width: mip_w,
            height: mip_h,
            data: Arc::<[u8]>::from(payload),
        });
    }

    if mip_levels.is_empty() {
        return Err(Ktx2Error::InvalidContainer(
            "KTX2 file has zero mip levels".to_string(),
        ));
    }

    // W1.T9 — preserve KTX2 keyValueData per spec §3.10.8 with bounded
    // collection (DOS defence). Anterior Fase 1 parser silently descartava
    // kvd; W1.T9 audit Lente A HIGH#3 identificou. PH2D usa kvd para tag
    // alpha intent (PH2D_PREMUL key, W1.T8 cooker emit deferred).
    let mut kvd: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    // W1.T9 audit Lente ξ-F2 — count ITERATIONS, not `kvd.len()`. A
    // hostile file can repeat one key thousands of times; that keeps
    // `kvd.len()` at 1 while forcing a `to_vec()` per entry. Bounding the
    // raw iteration count caps total work regardless of duplicate keys.
    // Order is COUNT → KEY-SIZE → VALUE-SIZE → ALLOC: every bound gates
    // its allocation, none allocate-before-check.
    let mut seen: usize = 0;
    for (key, value) in reader.key_value_data() {
        seen += 1;
        if seen > MAX_KVD_ENTRIES {
            return Err(Ktx2Error::TooManyKvdEntries {
                count: seen,
                max: MAX_KVD_ENTRIES,
            });
        }
        if key.len() > MAX_KVD_KEY_BYTES {
            return Err(Ktx2Error::KvdKeyTooLong {
                size: key.len(),
                max: MAX_KVD_KEY_BYTES,
            });
        }
        if value.len() > MAX_KVD_VALUE_BYTES {
            return Err(Ktx2Error::KvdValueTooLarge {
                key: key.to_string(),
                size: value.len(),
                max: MAX_KVD_VALUE_BYTES,
            });
        }
        kvd.insert(key.to_string(), value.to_vec());
    }

    Ok(Ktx2Image {
        format,
        width: header.pixel_width,
        height: header.pixel_height,
        mip_levels,
        kvd,
    })
}

/// Encode a single-mip, **uncompressed RGBA8** KTX2 container from raw
/// straight-alpha pixels.
///
/// This is the `Constrained`-tier cooked artifact — the universal RGBA8
/// floor the W2.T4 loader's fallback ladder lands on when no compressed tier
/// fits the device (or none was cooked). The compressed BC/ASTC/ETC2 tiers
/// come from the offline `cooker` (`tools/asset-cooker`) + an ISPC encoder,
/// out of this pure read-side codec's scope; the uncompressed case needs no
/// encoder library, so it lives here and keeps the cooked-texture loader
/// path (and its GPU smoke) self-contained — no ISPC, no committed binary
/// fixture.
///
/// `rgba` is tight-packed `width * height * 4` bytes. `srgb` picks the
/// transfer: `true` → `VK_FORMAT_R8G8B8A8_SRGB` (the canonical sprite
/// encoding, decoded by the GPU sampler), `false` → `..._UNORM`. The output
/// round-trips through [`decode_ktx2_bytes`] to a single-level
/// [`Ktx2Image`] (pinned by a unit test). No KVD / SGD / supercompression.
///
/// # Errors
/// [`Ktx2Error::ZeroDimension`] for a 0-sized image, or
/// [`Ktx2Error::InvalidContainer`] when `rgba.len() != width * height * 4`.
pub fn encode_uncompressed_rgba8(
    width: u32,
    height: u32,
    srgb: bool,
    rgba: &[u8],
) -> Result<Vec<u8>, Ktx2Error> {
    use ktx2::dfd::{Basic, Block};
    use ktx2::{Format, Header, Index, LevelIndex};

    if width == 0 || height == 0 {
        return Err(Ktx2Error::ZeroDimension);
    }
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() != expected {
        return Err(Ktx2Error::InvalidContainer(format!(
            "rgba len {} != width*height*4 = {expected}",
            rgba.len()
        )));
    }

    let format = if srgb {
        Format::R8G8B8A8_SRGB
    } else {
        Format::R8G8B8A8_UNORM
    };
    // Data Format Descriptor: a single Basic block (4-byte total-size prefix
    // + the block), mirroring `build_fixture` in the decode tests.
    let (basic, type_size) = Basic::from_format(format).expect("RGBA8 is a known KTX2 format");
    let block_bytes = Block::Basic(basic).to_vec();
    let dfd_total_size = 4 + block_bytes.len() as u32;
    let mut dfd_section = Vec::with_capacity(dfd_total_size as usize);
    dfd_section.extend_from_slice(&dfd_total_size.to_le_bytes());
    dfd_section.extend_from_slice(&block_bytes);

    // Fixed KTX2 layout: 80-byte header, one 24-byte level-index entry, the
    // DFD, then the level payload. No KVD / SGD.
    let dfd_byte_offset: u32 = 80 + 24;
    let dfd_byte_length = dfd_section.len() as u32;
    // Level data 4-aligned with ≥1 slack byte after the DFD — `Reader::new`
    // enforces `dfd_end < input.len()` (the `+ 4 & !3` keeps it strictly
    // above the DFD and 4-aligned, matching the KTX2 mip-padding rule).
    let level_data_offset = ((dfd_byte_offset + dfd_byte_length) + 4) & !3;
    let total_len = level_data_offset as usize + rgba.len();

    let header = Header {
        format: Some(format),
        type_size,
        pixel_width: width,
        pixel_height: height,
        pixel_depth: 0,
        layer_count: 0,
        face_count: 1,
        level_count: 1,
        supercompression_scheme: None,
        index: Index {
            dfd_byte_offset,
            dfd_byte_length,
            kvd_byte_offset: 0,
            kvd_byte_length: 0,
            sgd_byte_offset: 0,
            sgd_byte_length: 0,
        },
    };

    let mut buf = vec![0u8; total_len];
    buf[0..80].copy_from_slice(&header.as_bytes());
    let entry = LevelIndex {
        byte_offset: u64::from(level_data_offset),
        byte_length: rgba.len() as u64,
        uncompressed_byte_length: rgba.len() as u64,
    };
    buf[80..104].copy_from_slice(&entry.as_bytes());
    buf[dfd_byte_offset as usize..(dfd_byte_offset + dfd_byte_length) as usize]
        .copy_from_slice(&dfd_section);
    let start = level_data_offset as usize;
    buf[start..start + rgba.len()].copy_from_slice(rgba);
    Ok(buf)
}

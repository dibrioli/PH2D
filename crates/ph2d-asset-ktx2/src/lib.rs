#![forbid(unsafe_code)]
//! `ph2d-asset-ktx2` — Khronos KTX2 texture container decoder (Fase 1,
//! codec-only).
//!
//! The KTX2 container splits a single byte stream into header + Data
//! Format Descriptor + key/value metadata + mip pyramid, and is the
//! Khronos-blessed delivery format for GPU-compressed textures
//! (BC7/BC6H desktop, ASTC mobile + Apple Silicon, ETC2 Android
//! fallback) plus uncompressed RGBA8 / RGBA16F. PH2D's SKILL §11.10
//! lists those formats as the v1 cooked-texture target.
//!
//! ## Scope of THIS crate (Fase 1)
//!
//! - **Parse a `.ktx2` byte buffer** into a typed [`Ktx2Image`] (format,
//!   dimensions, per-mip bytes).
//! - **Map VkFormat → [`Ktx2Format`]** for the subset PH2D will use,
//!   surfacing the rest as [`Ktx2Format::Unsupported`] so callers can
//!   make an explicit decision rather than silently mis-decoding.
//! - **Reject unsafe inputs** (oversize dimensions, oversize total
//!   payload, supercompression that we have not wired) — mirrors the
//!   defensive limits already in [`ph2d_asset`'s PNG/JPEG path].
//!
//! Out of scope for Fase 1 — these need foundational edits in
//! `ph2d-asset` / `ph2d-render` and will land behind an ADR:
//!
//! - No conversion to `Asset::*` variants — caller owns that mapping.
//! - No `wgpu::TextureFormat` mapping — `ph2d-render` decides per
//!   pipeline (atlas vs individual, sRGB vs linear).
//! - No supercompression decode (zstd / BasisLZ supercompression
//!   schemes are rejected with an explicit error).
//! - No transcoding (BC7 stays BC7; no runtime decode-to-RGBA8).
//! - No writer / cooker — read path only.
//!
//! ## Two-world placement (ADR-0021)
//!
//! This crate runs at **load time**, never in the render / physics /
//! audio hot path (HR-3). One owned heap allocation per mip level is
//! acceptable here; the engine's hot-path code consumes the decoded
//! mip slices read-only.

use std::sync::Arc;

use thiserror::Error;

// ── public limits ───────────────────────────────────────────────────

/// Largest texture edge accepted, in pixels. Matches `ph2d-render`'s
/// atlas cap ([`ATLAS_DEFAULT_SIZE_PX`] = 8192) and the PNG / JPEG
/// limits in `ph2d-asset::loader`. Anything larger is almost certainly
/// a malformed or hostile file.
///
/// [`ATLAS_DEFAULT_SIZE_PX`]: ../ph2d_render/atlas/constant.ATLAS_DEFAULT_SIZE_PX.html
pub const MAX_DIMENSION: u32 = 8192;

/// Cap on the sum of every mip level's bytes after parsing. KTX2 is a
/// container — a tiny header can claim gigabytes of mip data. 512 MiB
/// matches the `MAX_ALLOC_BYTES` guard in `ph2d-asset::loader`.
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;

/// Cap on declared mip count. `log2(8192) + 1 = 14`; 16 leaves headroom
/// for the (rare) NPOT case while still rejecting absurd headers.
pub const MAX_LEVELS: u32 = 16;

// Compile-time sanity: catches anyone zeroing a limit by accident
// (clippy bans these as runtime `assert!`-on-constants — const-context
// asserts are the canonical form).
const _: () = assert!(MAX_DIMENSION > 0 && MAX_DIMENSION <= 16384);
const _: () = assert!(MAX_TOTAL_BYTES > 0);
const _: () = assert!(MAX_LEVELS > 0 && MAX_LEVELS < 32);

// ── error type ──────────────────────────────────────────────────────

/// Failures during KTX2 decode. Each variant carries enough context
/// for the caller to either surface a precise error toast or decide on
/// a fallback path.
#[derive(Debug, Error)]
pub enum Ktx2Error {
    /// The bytes are not a valid KTX2 container (bad magic, truncated
    /// header, malformed level index, …). The wrapped string is the
    /// upstream `ktx2` crate's diagnostic; treat as opaque.
    #[error("invalid KTX2 container: {0}")]
    InvalidContainer(String),

    /// The container declares dimensions beyond [`MAX_DIMENSION`].
    /// Could be a real high-res asset (raise the cap deliberately) or
    /// a header attack (file says 2³¹ but allocates nothing).
    #[error("dimension {dim} exceeds max {max}")]
    BoundsExceeded { dim: u32, max: u32 },

    /// Sum of mip-level payload bytes exceeds [`MAX_TOTAL_BYTES`].
    #[error("total mip bytes {total} exceeds max {max}")]
    TotalBytesExceeded { total: u64, max: u64 },

    /// One of width / height / layer / face was zero.
    #[error("zero dimension is not allowed")]
    ZeroDimension,

    /// Container declares more mip levels than [`MAX_LEVELS`].
    #[error("level count {count} exceeds max {max}")]
    TooManyLevels { count: u32, max: u32 },

    /// Header `format = VK_FORMAT_UNDEFINED` (0). The KTX2 spec allows
    /// this for formats described purely by the Data Format Descriptor
    /// (e.g. some Basis layouts), but we do not interpret the DFD in
    /// Fase 1 — callers must supply a file with an explicit VkFormat.
    #[error("KTX2 file has no declared VkFormat (DFD-only formats not supported in Fase 1)")]
    MissingFormat,

    /// Container declares a non-`None` supercompression scheme (zstd,
    /// BasisLZ, ZLIB, …). Decoding those is foundational work
    /// gated by an ADR; for now we reject explicitly.
    #[error("supercompression scheme {raw} is not supported in Fase 1")]
    UnsupportedSupercompression { raw: u32 },

    /// Declared mip-level data slice did not match the index.
    #[error("level {level}: data length mismatch")]
    LevelDataMismatch { level: u32 },

    /// Container declares a non-2D layout: 3D texture
    /// (`pixel_depth > 0`), cubemap (`face_count == 6`), or array
    /// texture (`layer_count > 1`). Fase 1 only wires 2D sprites
    /// — the renderer has no path for the other layouts yet.
    /// Lifting the restriction is a foundational change (Fase 2):
    /// `Ktx2Image` would need to carry layer/face arrays.
    #[error("non-2D KTX2 layout not supported in Fase 1: {reason}")]
    UnsupportedDimensionality { reason: &'static str },
}

// ── format enum ─────────────────────────────────────────────────────

/// Subset of VkFormat values that PH2D's renderer either supports
/// natively (RGBA8/16) or plans to support once
/// `wgpu::TextureFormat` paths land (BC7, BC6H, BC4, BC5, ASTC,
/// ETC2). Unknown or out-of-subset VkFormat values are surfaced as
/// [`Self::Unsupported`] with the raw u32 preserved, so the caller can
/// log it or decide whether to extend this enum.
///
/// Naming mirrors `wgpu::TextureFormat` (`Rgba8UnormSrgb`,
/// `Bc7RgbaUnormSrgb`, …) on purpose — Fase 2's wire-up just needs a
/// straight `match`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ktx2Format {
    // Uncompressed
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Unorm,
    Rgba16Float,
    Rgba32Float,

    // BC family — desktop (D3D12, Vulkan, Metal via MoltenVK)
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc5RgUnorm,
    Bc6hRgbUfloat,
    Bc6hRgbSfloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,

    // ASTC family — mobile, Apple Silicon native, Adreno / Mali
    Astc4x4RgbaUnorm,
    Astc4x4RgbaUnormSrgb,
    Astc5x5RgbaUnorm,
    Astc5x5RgbaUnormSrgb,
    Astc6x6RgbaUnorm,
    Astc6x6RgbaUnormSrgb,
    Astc8x8RgbaUnorm,
    Astc8x8RgbaUnormSrgb,

    // ETC family — Android fallback
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,

    /// VkFormat outside the supported subset above. Preserved as raw
    /// u32 so the caller can log it precisely (e.g. for telemetry on
    /// "what formats artists are actually shipping us").
    Unsupported(u32),
}

impl Ktx2Format {
    /// `true` if this variant is GPU-compressed (BC*, ASTC*, ETC2*).
    /// Useful for callers deciding whether to upload directly or
    /// transcode to RGBA8 first. [`Self::Unsupported`] returns
    /// `false` — caller cannot reason about it.
    #[must_use]
    pub fn is_compressed(self) -> bool {
        matches!(
            self,
            Self::Bc1RgbaUnorm
                | Self::Bc1RgbaUnormSrgb
                | Self::Bc3RgbaUnorm
                | Self::Bc3RgbaUnormSrgb
                | Self::Bc4RUnorm
                | Self::Bc5RgUnorm
                | Self::Bc6hRgbUfloat
                | Self::Bc6hRgbSfloat
                | Self::Bc7RgbaUnorm
                | Self::Bc7RgbaUnormSrgb
                | Self::Astc4x4RgbaUnorm
                | Self::Astc4x4RgbaUnormSrgb
                | Self::Astc5x5RgbaUnorm
                | Self::Astc5x5RgbaUnormSrgb
                | Self::Astc6x6RgbaUnorm
                | Self::Astc6x6RgbaUnormSrgb
                | Self::Astc8x8RgbaUnorm
                | Self::Astc8x8RgbaUnormSrgb
                | Self::Etc2Rgb8Unorm
                | Self::Etc2Rgb8UnormSrgb
                | Self::Etc2Rgba8Unorm
                | Self::Etc2Rgba8UnormSrgb
        )
    }

    /// `true` if this format carries HDR (>8 bpc / float) data.
    /// Drives the Sprite source decision in `ph2d-render` between the
    /// SDR atlas path and the HDR `GameRt` path.
    #[must_use]
    pub fn is_hdr(self) -> bool {
        matches!(
            self,
            Self::Rgba16Unorm
                | Self::Rgba16Float
                | Self::Rgba32Float
                | Self::Bc6hRgbUfloat
                | Self::Bc6hRgbSfloat
        )
    }

    /// Map a raw VkFormat enum value (as serialised in the KTX2
    /// header) to our typed enum. Unknown values fall back to
    /// [`Self::Unsupported`].
    ///
    /// VkFormat numeric values are from the Vulkan registry; the ones
    /// listed here are stable across Vulkan releases.
    #[must_use]
    pub fn from_vk_format(raw: u32) -> Self {
        // Vulkan VkFormat numeric values — Khronos registry.
        match raw {
            37 => Self::Rgba8Unorm,     // VK_FORMAT_R8G8B8A8_UNORM
            43 => Self::Rgba8UnormSrgb, // VK_FORMAT_R8G8B8A8_SRGB
            91 => Self::Rgba16Unorm,    // VK_FORMAT_R16G16B16A16_UNORM
            97 => Self::Rgba16Float,    // VK_FORMAT_R16G16B16A16_SFLOAT
            109 => Self::Rgba32Float,   // VK_FORMAT_R32G32B32A32_SFLOAT

            133 => Self::Bc1RgbaUnorm, // VK_FORMAT_BC1_RGBA_UNORM_BLOCK
            134 => Self::Bc1RgbaUnormSrgb, // VK_FORMAT_BC1_RGBA_SRGB_BLOCK
            137 => Self::Bc3RgbaUnorm, // VK_FORMAT_BC3_UNORM_BLOCK
            138 => Self::Bc3RgbaUnormSrgb, // VK_FORMAT_BC3_SRGB_BLOCK
            139 => Self::Bc4RUnorm,    // VK_FORMAT_BC4_UNORM_BLOCK
            141 => Self::Bc5RgUnorm,   // VK_FORMAT_BC5_UNORM_BLOCK
            143 => Self::Bc6hRgbUfloat, // VK_FORMAT_BC6H_UFLOAT_BLOCK
            144 => Self::Bc6hRgbSfloat, // VK_FORMAT_BC6H_SFLOAT_BLOCK
            145 => Self::Bc7RgbaUnorm, // VK_FORMAT_BC7_UNORM_BLOCK
            146 => Self::Bc7RgbaUnormSrgb, // VK_FORMAT_BC7_SRGB_BLOCK

            147 => Self::Etc2Rgb8Unorm, // VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK
            148 => Self::Etc2Rgb8UnormSrgb, // VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK
            151 => Self::Etc2Rgba8Unorm, // VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK
            152 => Self::Etc2Rgba8UnormSrgb, // VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK

            157 => Self::Astc4x4RgbaUnorm, // VK_FORMAT_ASTC_4x4_UNORM_BLOCK
            158 => Self::Astc4x4RgbaUnormSrgb, // VK_FORMAT_ASTC_4x4_SRGB_BLOCK
            161 => Self::Astc5x5RgbaUnorm, // VK_FORMAT_ASTC_5x5_UNORM_BLOCK
            162 => Self::Astc5x5RgbaUnormSrgb, // VK_FORMAT_ASTC_5x5_SRGB_BLOCK
            165 => Self::Astc6x6RgbaUnorm, // VK_FORMAT_ASTC_6x6_UNORM_BLOCK
            166 => Self::Astc6x6RgbaUnormSrgb, // VK_FORMAT_ASTC_6x6_SRGB_BLOCK
            171 => Self::Astc8x8RgbaUnorm, // VK_FORMAT_ASTC_8x8_UNORM_BLOCK
            172 => Self::Astc8x8RgbaUnormSrgb, // VK_FORMAT_ASTC_8x8_SRGB_BLOCK

            other => Self::Unsupported(other),
        }
    }
}

// ── image + mip types ───────────────────────────────────────────────

/// One mip level of the decoded pyramid. `data` is the raw bytes in
/// the declared [`Ktx2Format`] — uncompressed for RGBA*, compressed
/// blocks for BC / ASTC / ETC2. The decoder makes one heap allocation
/// per mip; the `Arc<[u8]>` lets the caller share the bytes between
/// the asset DB and the renderer without re-copying.
#[derive(Debug, Clone)]
pub struct MipLevel {
    /// Width of THIS mip in pixels (mip 0 == header width, mip N is
    /// `max(1, width >> N)`).
    pub width: u32,
    /// Height of THIS mip in pixels.
    pub height: u32,
    /// Raw payload — interpretation depends on [`Ktx2Image::format`].
    pub data: Arc<[u8]>,
}

/// A fully decoded KTX2 file. Header dimensions are mip 0 (the
/// largest level). Cubemap faces and array layers are NOT yet
/// flattened — Fase 1 rejects multi-layer / multi-face inputs to keep
/// the surface tight; the limits are deliberately conservative and
/// will be relaxed in Fase 2 if the asset pipeline needs them.
#[derive(Debug, Clone)]
pub struct Ktx2Image {
    pub format: Ktx2Format,
    pub width: u32,
    pub height: u32,
    /// Mip pyramid from level 0 (largest) to level N-1. Always at
    /// least one entry.
    pub mip_levels: Vec<MipLevel>,
}

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
fn validate_2d_only(header: &ktx2::Header) -> Result<(), Ktx2Error> {
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

    Ok(Ktx2Image {
        format,
        width: header.pixel_width,
        height: header.pixel_height,
        mip_levels,
    })
}

// ── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    /// VkFormat values come from the Vulkan registry — exercise the
    /// canonical entries we listed to catch typos in the mapping.
    #[test]
    fn vk_format_mapping_canonical_values() {
        assert_eq!(Ktx2Format::from_vk_format(37), Ktx2Format::Rgba8Unorm);
        assert_eq!(Ktx2Format::from_vk_format(43), Ktx2Format::Rgba8UnormSrgb);
        assert_eq!(Ktx2Format::from_vk_format(97), Ktx2Format::Rgba16Float);
        assert_eq!(Ktx2Format::from_vk_format(145), Ktx2Format::Bc7RgbaUnorm);
        assert_eq!(
            Ktx2Format::from_vk_format(146),
            Ktx2Format::Bc7RgbaUnormSrgb
        );
        assert_eq!(
            Ktx2Format::from_vk_format(166),
            Ktx2Format::Astc6x6RgbaUnormSrgb
        );
        assert_eq!(
            Ktx2Format::from_vk_format(152),
            Ktx2Format::Etc2Rgba8UnormSrgb
        );
    }

    /// Unknown VkFormat must surface as `Unsupported(raw)` rather
    /// than silently mapping to something.
    #[test]
    fn vk_format_unknown_is_unsupported() {
        // 9999 isn't a real VkFormat — pick anything outside our
        // mapped subset.
        assert_eq!(
            Ktx2Format::from_vk_format(9999),
            Ktx2Format::Unsupported(9999)
        );
        // VK_FORMAT_R4G4_UNORM_PACK8 (1) is real but we don't use it.
        assert_eq!(Ktx2Format::from_vk_format(1), Ktx2Format::Unsupported(1));
    }

    /// Compressed-vs-uncompressed classification is consumed by
    /// downstream code that picks the upload path.
    #[test]
    fn is_compressed_matches_family() {
        assert!(!Ktx2Format::Rgba8UnormSrgb.is_compressed());
        assert!(!Ktx2Format::Rgba16Float.is_compressed());
        assert!(Ktx2Format::Bc7RgbaUnormSrgb.is_compressed());
        assert!(Ktx2Format::Bc6hRgbSfloat.is_compressed());
        assert!(Ktx2Format::Astc6x6RgbaUnormSrgb.is_compressed());
        assert!(Ktx2Format::Etc2Rgba8UnormSrgb.is_compressed());
        // Unsupported can't be reasoned about.
        assert!(!Ktx2Format::Unsupported(9999).is_compressed());
    }

    /// HDR classification gates the SDR atlas vs HDR `GameRt` path
    /// in the future renderer wire-up.
    #[test]
    fn is_hdr_matches_family() {
        assert!(!Ktx2Format::Rgba8UnormSrgb.is_hdr());
        assert!(!Ktx2Format::Bc7RgbaUnormSrgb.is_hdr());
        assert!(Ktx2Format::Rgba16Float.is_hdr());
        assert!(Ktx2Format::Rgba32Float.is_hdr());
        assert!(Ktx2Format::Bc6hRgbUfloat.is_hdr());
        assert!(Ktx2Format::Bc6hRgbSfloat.is_hdr());
        assert!(!Ktx2Format::Unsupported(9999).is_hdr());
    }

    // ── dimensionality reject tests ─────────────────────────────────
    //
    // The `ktx2` crate's `Reader::new` validates the whole file (DFD +
    // level index + payload bounds), so we can't drive it from a
    // synthetic 80-byte header alone. But `validate_2d_only` operates
    // on a parsed `ktx2::Header`, which `Header::from_bytes` builds
    // from exactly 80 bytes — perfect for unit tests.

    /// Build a valid 80-byte KTX2 header (magic + fields) with the
    /// dimension knobs we want to flip. Defaults are a plain 2D
    /// texture (8×8, no depth, no array, single face, RGBA8_SRGB).
    fn build_header_bytes(pixel_depth: u32, layer_count: u32, face_count: u32) -> [u8; 80] {
        let mut buf = [0u8; 80];
        // Magic «KTX 20»\r\n\x1a\n (12 bytes).
        buf[0..12].copy_from_slice(&[
            0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
        ]);
        // VkFormat = 43 (R8G8B8A8_SRGB). Non-zero so Header parses.
        buf[12..16].copy_from_slice(&43u32.to_le_bytes());
        // typeSize = 1 (block-compressed/sRGB convention).
        buf[16..20].copy_from_slice(&1u32.to_le_bytes());
        // pixelWidth = 8 (non-zero — Header rejects zero width).
        buf[20..24].copy_from_slice(&8u32.to_le_bytes());
        // pixelHeight = 8.
        buf[24..28].copy_from_slice(&8u32.to_le_bytes());
        // pixelDepth — the knob.
        buf[28..32].copy_from_slice(&pixel_depth.to_le_bytes());
        // layerCount — the knob.
        buf[32..36].copy_from_slice(&layer_count.to_le_bytes());
        // faceCount — the knob (1 or 6 per spec; Header rejects 0).
        buf[36..40].copy_from_slice(&face_count.to_le_bytes());
        // levelCount = 1.
        buf[40..44].copy_from_slice(&1u32.to_le_bytes());
        // supercompressionScheme = 0 (none).
        buf[44..48].copy_from_slice(&0u32.to_le_bytes());
        // Index fields (DFD/KVD/SGD offsets+lengths) — left zero;
        // `Header::from_bytes` doesn't dereference them.
        buf
    }

    #[test]
    fn validate_2d_only_accepts_plain_2d() {
        // pixel_depth = 0, layer_count = 0 (non-array), face_count = 1.
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

    // ── positive round-trip via synthetic fixture ──────────────────
    //
    // The dev box has no `.ktx2` files lying around and we don't want
    // to commit a binary fixture before there's an asset pipeline
    // contract for it. The upstream `ktx2` crate exposes enough public
    // API (`Basic::from_format`, `Block::to_vec`, `Header::as_bytes`,
    // `LevelIndex::as_bytes`) to build a fully valid KTX2 file in
    // memory — that proves our decoder accepts spec-compliant input,
    // not just rejects garbage.

    /// Build a minimal valid KTX2 buffer: 2D RGBA8_SRGB, 1×1 px, 1 mip,
    /// no supercompression, no KVD, no SGD. Pixel = `[0xFF, 0x00,
    /// 0x80, 0xFF]` (hot magenta with full alpha) so the round-trip
    /// is observable.
    fn build_synthetic_rgba8_srgb_1x1() -> Vec<u8> {
        use ktx2::dfd::{Basic, Block};
        use ktx2::{Format, Header, Index, LevelIndex};

        // DFD section: 4-byte total-size prefix + serialized Basic block.
        let (basic, type_size) =
            Basic::from_format(Format::R8G8B8A8_SRGB).expect("R8G8B8A8_SRGB is a known format");
        let block_bytes = Block::Basic(basic).to_vec();
        let dfd_total_size: u32 = u32::try_from(4 + block_bytes.len()).expect("DFD fits in u32");
        let mut dfd_section = Vec::with_capacity(dfd_total_size as usize);
        dfd_section.extend_from_slice(&dfd_total_size.to_le_bytes());
        dfd_section.extend_from_slice(&block_bytes);

        // Layout: header (80) + level index (24 × 1) + DFD + level data.
        // Level data is aligned to lcm(4, texel_block_size) per spec; for
        // uncompressed RGBA8 that's 4.
        let level_index_offset: u32 = 80;
        let dfd_byte_offset: u32 = level_index_offset + 24;
        let dfd_byte_length: u32 = u32::try_from(dfd_section.len()).expect("DFD len fits in u32");
        let level_data_offset_raw = dfd_byte_offset + dfd_byte_length;
        let level_data_offset = (level_data_offset_raw + 3) & !3;
        let level_data_length: u64 = 4; // 1 px × 4 bytes RGBA8

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
                dfd_byte_offset,
                dfd_byte_length,
                kvd_byte_offset: 0,
                kvd_byte_length: 0,
                sgd_byte_offset: 0,
                sgd_byte_length: 0,
            },
        };

        let level_index = LevelIndex {
            byte_offset: u64::from(level_data_offset),
            byte_length: level_data_length,
            uncompressed_byte_length: level_data_length,
        };

        let total_len = level_data_offset as usize + level_data_length as usize;
        let mut buf = vec![0u8; total_len];
        buf[0..80].copy_from_slice(&header.as_bytes());
        buf[80..104].copy_from_slice(&level_index.as_bytes());
        buf[dfd_byte_offset as usize..(dfd_byte_offset + dfd_byte_length) as usize]
            .copy_from_slice(&dfd_section);
        // Hot magenta — distinguishable from zeroed padding.
        buf[level_data_offset as usize..level_data_offset as usize + 4]
            .copy_from_slice(&[0xFF, 0x00, 0x80, 0xFF]);

        buf
    }

    #[test]
    fn decode_synthetic_rgba8_1x1_round_trips() {
        let bytes = build_synthetic_rgba8_srgb_1x1();
        let image = decode_ktx2_bytes(&bytes).expect("valid synthetic file decodes");

        assert_eq!(image.format, Ktx2Format::Rgba8UnormSrgb);
        assert_eq!(image.width, 1);
        assert_eq!(image.height, 1);
        assert_eq!(image.mip_levels.len(), 1);

        let mip = &image.mip_levels[0];
        assert_eq!(mip.width, 1);
        assert_eq!(mip.height, 1);
        assert_eq!(mip.data.as_ref(), &[0xFF, 0x00, 0x80, 0xFF]);
    }

    #[test]
    fn decode_synthetic_format_classifies_correctly() {
        // Same buffer drives the classifier helpers — sanity-check that
        // an end-to-end decoded RGBA8_SRGB is reported as uncompressed
        // SDR (the SDR atlas path in the future renderer).
        let bytes = build_synthetic_rgba8_srgb_1x1();
        let image = decode_ktx2_bytes(&bytes).expect("decodes");
        assert!(!image.format.is_compressed());
        assert!(!image.format.is_hdr());
    }
}

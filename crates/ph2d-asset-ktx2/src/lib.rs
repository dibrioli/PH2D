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

use std::collections::BTreeMap;
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

/// W1.T9 — cap on KTX2 keyValueData entries preserved by the parser. KTX2
/// containers MAY ship arbitrary metadata (timestamps, swizzle, custom
/// keys); a malformed/hostile file could declare thousands. 64 covers all
/// realistic use cases (KTX-Software conventions, PH2D's own `PH2D_PREMUL`
/// W1.T8, glTF KHR_*, swizzle keys) with plenty of headroom.
pub const MAX_KVD_ENTRIES: usize = 64;

/// W1.T9 — cap on individual kvd value size in bytes. Most KTX2 metadata
/// values are < 64 B (strings, single bytes); 4 KiB allows complex JSON-like
/// values used by some tooling while bounding memory in pathological files.
pub const MAX_KVD_VALUE_BYTES: usize = 4 * 1024;

/// W1.T9 audit Lente ξ-F3 — cap on individual kvd *key* length in bytes.
/// Symmetric DOS defence to [`MAX_KVD_VALUE_BYTES`]: without it a hostile
/// file could ship a multi-MiB key (the value cap alone leaves the key
/// unbounded) and force a large `key.to_string()` allocation. Real KTX2
/// keys are short identifiers ("KTXorientation", "PH2D_PREMUL"); 256 B is
/// generous headroom. Aggregate worst case is therefore bounded:
/// `MAX_KVD_ENTRIES × (MAX_KVD_KEY_BYTES + MAX_KVD_VALUE_BYTES)`
/// ≈ 64 × (256 B + 4 KiB) ≈ 272 KiB.
pub const MAX_KVD_KEY_BYTES: usize = 256;

/// W1.T9 — canonical KTX2 keyValueData key used by PH2D to tag the alpha
/// intent of a cooked texture (`PremulIntent`). 1-byte value:
/// `[0] = Straight`, `[1] = Premultiplied`. Key ausente = `Unspecified`.
/// Emit deferred a W1.T8 (ctt 0.4.0 não suporta kvd write; precisa de
/// patcher post-hoc OR upstream PR).
pub const PH2D_PREMUL_KEY: &str = "PH2D_PREMUL";

// Compile-time sanity: catches anyone zeroing a limit by accident
// (clippy bans these as runtime `assert!`-on-constants — const-context
// asserts are the canonical form).
const _: () = assert!(MAX_DIMENSION > 0 && MAX_DIMENSION <= 16384);
const _: () = assert!(MAX_TOTAL_BYTES > 0);
const _: () = assert!(MAX_LEVELS > 0 && MAX_LEVELS < 32);
const _: () = assert!(MAX_KVD_ENTRIES > 0 && MAX_KVD_ENTRIES <= 256);
const _: () = assert!(MAX_KVD_VALUE_BYTES > 0 && MAX_KVD_VALUE_BYTES <= 64 * 1024);
const _: () = assert!(MAX_KVD_KEY_BYTES > 0 && MAX_KVD_KEY_BYTES <= 4 * 1024);

// ── error type ──────────────────────────────────────────────────────

/// Failures during KTX2 decode. Each variant carries enough context
/// for the caller to either surface a precise error toast or decide on
/// a fallback path.
///
/// `#[non_exhaustive]` (W1.T9 audit Lente ν-7): future variants must
/// not be a breaking change for downstream consumers. External callers
/// match with a wildcard arm; within this crate the attribute is inert.
#[derive(Debug, Error)]
#[non_exhaustive]
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

    /// W1.T9 — kvd section declares mais entries que [`MAX_KVD_ENTRIES`].
    /// Hostile file ou tooling explorou kvd para metadata bloat.
    #[error("kvd has {count} entries, exceeds max {max}")]
    TooManyKvdEntries { count: usize, max: usize },

    /// W1.T9 — kvd entry value excede [`MAX_KVD_VALUE_BYTES`]. Hostile file
    /// pode embedar arbitrary blobs em metadata.
    #[error("kvd entry '{key}' has {size} bytes, exceeds max {max}")]
    KvdValueTooLarge {
        key: String,
        size: usize,
        max: usize,
    },

    /// W1.T9 audit Lente ξ-F3 — kvd entry key excede [`MAX_KVD_KEY_BYTES`].
    /// The offending key is intentionally NOT carried in the error — doing
    /// so would perform the very multi-MiB allocation this bound prevents.
    #[error("kvd entry key has {size} bytes, exceeds max {max}")]
    KvdKeyTooLong { size: usize, max: usize },
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
// W1.T15 audit Lente π-1: o módulo doc + ADR-0055 prometem mais VkFormats em
// Fase 2 e o design espera que downstreams façam `match` neste enum (vide
// doctest + renderer W2). Adicionar variante seria breaking-change sem isto.
// `Unsupported(u32)` NÃO basta — `match Foo | Unsupported(_)` ainda quebra ao
// surgir `Bar`. Simétrico ao `#[non_exhaustive]` que ν-7 pôs em Ktx2Error/Image.
#[non_exhaustive]
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

    /// `true` if this format can carry values **outside** the `[0, 1]`
    /// sRGB-equivalent range — i.e. floating-point storage. Drives the
    /// Sprite source decision in `ph2d-render` between the SDR atlas
    /// path and the HDR `GameRt` path.
    ///
    /// Note: `Rgba16Unorm` is **not** HDR by this definition — it has
    /// 16 bits of *precision* per channel but the storage still
    /// represents `[0, 1]` like RGBA8. Use it for things like high-
    /// precision masks or normals, not for high-luminance content.
    /// HDR requires float storage (`Rgba16Float`, `Rgba32Float`,
    /// `Bc6hRgb{Ufloat,Sfloat}`).
    #[must_use]
    pub fn is_hdr(self) -> bool {
        matches!(
            self,
            Self::Rgba16Float | Self::Rgba32Float | Self::Bc6hRgbUfloat | Self::Bc6hRgbSfloat
        )
    }

    /// Map a raw VkFormat enum value (as serialised in the KTX2
    /// header) to our typed enum. Unknown values fall back to
    /// [`Self::Unsupported`].
    ///
    /// The numeric values are matched against the constants the
    /// upstream `ktx2` crate exposes (`ktx2::Format::*`), which
    /// mirror the Khronos Vulkan registry. Using the crate's
    /// constants instead of hard-coded integers means a typo in any
    /// VkFormat ID is caught at compile time.
    #[must_use]
    pub fn from_vk_format(raw: u32) -> Self {
        use ktx2::Format as F;

        // Common reference shorthand for each variant the renderer
        // is expected to consume in Fase 2.
        if raw == F::R8G8B8A8_UNORM.value() {
            return Self::Rgba8Unorm;
        }
        if raw == F::R8G8B8A8_SRGB.value() {
            return Self::Rgba8UnormSrgb;
        }
        if raw == F::R16G16B16A16_UNORM.value() {
            return Self::Rgba16Unorm;
        }
        if raw == F::R16G16B16A16_SFLOAT.value() {
            return Self::Rgba16Float;
        }
        if raw == F::R32G32B32A32_SFLOAT.value() {
            return Self::Rgba32Float;
        }

        if raw == F::BC1_RGBA_UNORM_BLOCK.value() {
            return Self::Bc1RgbaUnorm;
        }
        if raw == F::BC1_RGBA_SRGB_BLOCK.value() {
            return Self::Bc1RgbaUnormSrgb;
        }
        if raw == F::BC3_UNORM_BLOCK.value() {
            return Self::Bc3RgbaUnorm;
        }
        if raw == F::BC3_SRGB_BLOCK.value() {
            return Self::Bc3RgbaUnormSrgb;
        }
        if raw == F::BC4_UNORM_BLOCK.value() {
            return Self::Bc4RUnorm;
        }
        if raw == F::BC5_UNORM_BLOCK.value() {
            return Self::Bc5RgUnorm;
        }
        if raw == F::BC6H_UFLOAT_BLOCK.value() {
            return Self::Bc6hRgbUfloat;
        }
        if raw == F::BC6H_SFLOAT_BLOCK.value() {
            return Self::Bc6hRgbSfloat;
        }
        if raw == F::BC7_UNORM_BLOCK.value() {
            return Self::Bc7RgbaUnorm;
        }
        if raw == F::BC7_SRGB_BLOCK.value() {
            return Self::Bc7RgbaUnormSrgb;
        }

        if raw == F::ETC2_R8G8B8_UNORM_BLOCK.value() {
            return Self::Etc2Rgb8Unorm;
        }
        if raw == F::ETC2_R8G8B8_SRGB_BLOCK.value() {
            return Self::Etc2Rgb8UnormSrgb;
        }
        if raw == F::ETC2_R8G8B8A8_UNORM_BLOCK.value() {
            return Self::Etc2Rgba8Unorm;
        }
        if raw == F::ETC2_R8G8B8A8_SRGB_BLOCK.value() {
            return Self::Etc2Rgba8UnormSrgb;
        }

        if raw == F::ASTC_4x4_UNORM_BLOCK.value() {
            return Self::Astc4x4RgbaUnorm;
        }
        if raw == F::ASTC_4x4_SRGB_BLOCK.value() {
            return Self::Astc4x4RgbaUnormSrgb;
        }
        if raw == F::ASTC_5x5_UNORM_BLOCK.value() {
            return Self::Astc5x5RgbaUnorm;
        }
        if raw == F::ASTC_5x5_SRGB_BLOCK.value() {
            return Self::Astc5x5RgbaUnormSrgb;
        }
        if raw == F::ASTC_6x6_UNORM_BLOCK.value() {
            return Self::Astc6x6RgbaUnorm;
        }
        if raw == F::ASTC_6x6_SRGB_BLOCK.value() {
            return Self::Astc6x6RgbaUnormSrgb;
        }
        if raw == F::ASTC_8x8_UNORM_BLOCK.value() {
            return Self::Astc8x8RgbaUnorm;
        }
        if raw == F::ASTC_8x8_SRGB_BLOCK.value() {
            return Self::Astc8x8RgbaUnormSrgb;
        }

        Self::Unsupported(raw)
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
/// `#[non_exhaustive]` (W1.T9 audit Lente ν-7): adding a field in Fase 2
/// must stay additive for any future external consumer (today there are
/// none — decode goes through [`decode_ktx2_bytes`], not struct literals
/// outside this crate). Within this crate, struct-literal construction
/// remains allowed.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Ktx2Image {
    pub format: Ktx2Format,
    pub width: u32,
    pub height: u32,
    /// Mip pyramid from level 0 (largest) to level N-1. Always at
    /// least one entry.
    pub mip_levels: Vec<MipLevel>,
    /// W1.T9 — KTX2 `keyValueData` preserved per spec §3.10.8. Empty se
    /// container não tem kvd OR caller construiu via struct literal sem
    /// passar kvd. Bounded por [`MAX_KVD_ENTRIES`] + [`MAX_KVD_VALUE_BYTES`]
    /// no parser. BTreeMap (não HashMap) garante iteration ordering
    /// determinístico (HR-6).
    pub kvd: BTreeMap<String, Vec<u8>>,
}

impl Ktx2Image {
    /// Shorthand for `&self.mip_levels[0]` — the largest, full-
    /// resolution level. Always present: the decoder rejects files
    /// with zero mip levels as `InvalidContainer`, so this never
    /// panics for an `Ktx2Image` produced by [`decode_ktx2_bytes`].
    #[must_use]
    pub fn base_level(&self) -> &MipLevel {
        &self.mip_levels[0]
    }

    /// W1.T9 — sum of mip level payload bytes. HR-13 budget accounting
    /// helper: used by `ph2d-asset::Asset::TextureKtx2.byte_size()` so the
    /// memory budget aggregator can size cooked textures sem extra parse.
    /// Não conta kvd ou Arc/Vec overhead — pure payload.
    #[must_use]
    pub fn byte_size_estimate(&self) -> usize {
        self.mip_levels.iter().map(|m| m.data.len()).sum()
    }

    /// W1.T9 — read [`PremulIntent`] from `kvd[PH2D_PREMUL_KEY]`. Tri-state:
    /// `[0] = Straight`, `[1] = Premultiplied`, key ausente OR malformed →
    /// `Unspecified`. Renderer pode usar `Unspecified` para defer decision
    /// pra source asset metadata OR conservative default.
    ///
    /// NB W1.T8 deferral: ctt 0.4.0 cooker NÃO emite kvd. Cooked KTX2 hoje
    /// always retorna `Unspecified` aqui. API serve future cooker integration
    /// (W1.T8.1 OR upstream ctt PR).
    #[must_use]
    pub fn premul_intent(&self) -> PremulIntent {
        match self.kvd.get(PH2D_PREMUL_KEY).map(|v| v.as_slice()) {
            Some([0]) => PremulIntent::Straight,
            Some([1]) => PremulIntent::Premultiplied,
            _ => PremulIntent::Unspecified,
        }
    }
}

/// W1.T9 — tri-state alpha intent flag carried via KTX2 `PH2D_PREMUL` kvd key.
///
/// - `Straight` — RGB components encode non-premultiplied color. Renderer
///   deve premultiplicar antes de compositing.
/// - `Premultiplied` — RGB já contém color * alpha. Renderer composita direto.
/// - `Unspecified` — key ausente; caller decide default (conservative:
///   tratar como Straight; aggressive: assume Premultiplied per ctt convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// W1.T15 audit Lente π-2: enum de metadata forward-compat (alpha-intent tagging);
// uma intent futura (ex. AssociatedAlpha/Coverage) é plausível. Fence agora —
// zero consumidor externo, custo ergonômico zero.
#[non_exhaustive]
pub enum PremulIntent {
    Straight,
    Premultiplied,
    Unspecified,
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

// ── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
            while out.len() % 4 != 0 {
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
}

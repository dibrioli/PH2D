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

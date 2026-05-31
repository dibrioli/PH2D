//! KTX2 → wgpu texture-format mapping (KTX2 Fase 2, W2.T1).
//!
//! The cooker ([`tools/asset-cooker`]) emits compressed `.ktx2`
//! artifacts; the parser ([`ph2d_asset_ktx2`]) decodes their header
//! into a typed [`Ktx2Format`]. Before the renderer can upload those
//! bytes it must translate that typed format into the concrete
//! [`wgpu::TextureFormat`] the GPU pipeline binds, **and** the
//! [`wgpu::Features`] flag the device must have enabled for the upload
//! to be legal (BC/ASTC/ETC2 are all opt-in adapter features).
//!
//! ## Exhaustiveness vs `#[non_exhaustive]`
//!
//! W2.T1 of the texture-compression plan asked for a wildcard-free
//! `match` so a newly-added [`Ktx2Format`] variant would break the
//! build until mapped. That is **not possible**: [`Ktx2Format`] was
//! frozen `#[non_exhaustive]` (ν-7), which forces every downstream
//! `match` — in any other crate — to carry a `_ =>` arm. So pure
//! compile-time exhaustiveness across the crate boundary cannot exist.
//!
//! We preserve the *intent* two ways instead:
//!   1. Every one of the 27 concrete variants has an explicit arm
//!      (the `_ =>` arm is reachable only by a *future* variant added
//!      upstream), mapping a forgotten variant to
//!      [`FormatError::UnmappedKtx2Format`] — a loud programming-error
//!      sentinel, distinct from a file declaring an out-of-subset
//!      VkFormat ([`FormatError::UnsupportedVkFormat`]).
//!   2. The `ktx2_format_exhaustive_mapping` arch-gate test constructs
//!      every known variant and asserts its exact `(format, feature)`
//!      pair, so a regression in any mapping fails CI.

use ph2d_asset_ktx2::Ktx2Format;

/// Why a [`Ktx2Format`] could not be turned into a renderable
/// [`wgpu::TextureFormat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormatError {
    /// The KTX2 file declared a VkFormat outside the subset the
    /// renderer supports (i.e. [`Ktx2Format::Unsupported`]). The raw
    /// VkFormat value is carried for precise telemetry/logging.
    UnsupportedVkFormat(u32),
    /// A [`Ktx2Format`] variant exists that this mapper has no arm
    /// for — reachable only if a variant is added upstream (the enum
    /// is `#[non_exhaustive]`) without a matching arm here. This is a
    /// renderer bug, not a property of the asset.
    UnmappedKtx2Format,
}

impl core::fmt::Display for FormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedVkFormat(raw) => write!(
                f,
                "KTX2 declares VkFormat {raw} which is outside the renderer's supported subset"
            ),
            Self::UnmappedKtx2Format => {
                f.write_str("Ktx2Format variant has no wgpu mapping (renderer bug — add an arm)")
            }
        }
    }
}

impl core::error::Error for FormatError {}

/// Translate a typed [`Ktx2Format`] into the `(wgpu::TextureFormat,
/// wgpu::Features)` pair the renderer needs to upload and bind it.
///
/// The returned [`wgpu::Features`] is the **single** adapter feature
/// the device must have enabled for this format to be a legal texture
/// (`empty()` for the core uncompressed formats). Callers gate the
/// upload on `device.features().contains(returned_features)` and fall
/// back to an uncompressed tier when it does not — see W2.T1.5
/// (`detect_supported_compressions`).
///
/// # Errors
/// Returns [`FormatError::UnsupportedVkFormat`] when the source file
/// declared a VkFormat outside our subset, or
/// [`FormatError::UnmappedKtx2Format`] for a future unmapped variant.
pub fn wgpu_format_from_ktx2_format(
    fmt: Ktx2Format,
) -> Result<(wgpu::TextureFormat, wgpu::Features), FormatError> {
    use wgpu::TextureFormat as Tf;
    use wgpu::{AstcBlock, AstcChannel, Features};

    let bc = Features::TEXTURE_COMPRESSION_BC;
    let astc = Features::TEXTURE_COMPRESSION_ASTC;
    let etc2 = Features::TEXTURE_COMPRESSION_ETC2;

    let astc = |block: AstcBlock, channel: AstcChannel| (Tf::Astc { block, channel }, astc);

    let mapped = match fmt {
        // ── Uncompressed (core; Rgba16Unorm needs the 16-bit-norm feature) ──
        Ktx2Format::Rgba8Unorm => (Tf::Rgba8Unorm, Features::empty()),
        Ktx2Format::Rgba8UnormSrgb => (Tf::Rgba8UnormSrgb, Features::empty()),
        Ktx2Format::Rgba16Unorm => (Tf::Rgba16Unorm, Features::TEXTURE_FORMAT_16BIT_NORM),
        Ktx2Format::Rgba16Float => (Tf::Rgba16Float, Features::empty()),
        Ktx2Format::Rgba32Float => (Tf::Rgba32Float, Features::empty()),

        // ── BC family (desktop: D3D12 / Vulkan / Metal-via-MoltenVK) ──
        Ktx2Format::Bc1RgbaUnorm => (Tf::Bc1RgbaUnorm, bc),
        Ktx2Format::Bc1RgbaUnormSrgb => (Tf::Bc1RgbaUnormSrgb, bc),
        Ktx2Format::Bc3RgbaUnorm => (Tf::Bc3RgbaUnorm, bc),
        Ktx2Format::Bc3RgbaUnormSrgb => (Tf::Bc3RgbaUnormSrgb, bc),
        Ktx2Format::Bc4RUnorm => (Tf::Bc4RUnorm, bc),
        Ktx2Format::Bc5RgUnorm => (Tf::Bc5RgUnorm, bc),
        Ktx2Format::Bc6hRgbUfloat => (Tf::Bc6hRgbUfloat, bc),
        // PH2D "Sfloat" is wgpu's signed `Bc6hRgbFloat`.
        Ktx2Format::Bc6hRgbSfloat => (Tf::Bc6hRgbFloat, bc),
        Ktx2Format::Bc7RgbaUnorm => (Tf::Bc7RgbaUnorm, bc),
        Ktx2Format::Bc7RgbaUnormSrgb => (Tf::Bc7RgbaUnormSrgb, bc),

        // ── ASTC family (mobile, Apple Silicon native, Adreno / Mali) ──
        Ktx2Format::Astc4x4RgbaUnorm => astc(AstcBlock::B4x4, AstcChannel::Unorm),
        Ktx2Format::Astc4x4RgbaUnormSrgb => astc(AstcBlock::B4x4, AstcChannel::UnormSrgb),
        Ktx2Format::Astc5x5RgbaUnorm => astc(AstcBlock::B5x5, AstcChannel::Unorm),
        Ktx2Format::Astc5x5RgbaUnormSrgb => astc(AstcBlock::B5x5, AstcChannel::UnormSrgb),
        Ktx2Format::Astc6x6RgbaUnorm => astc(AstcBlock::B6x6, AstcChannel::Unorm),
        Ktx2Format::Astc6x6RgbaUnormSrgb => astc(AstcBlock::B6x6, AstcChannel::UnormSrgb),
        Ktx2Format::Astc8x8RgbaUnorm => astc(AstcBlock::B8x8, AstcChannel::Unorm),
        Ktx2Format::Astc8x8RgbaUnormSrgb => astc(AstcBlock::B8x8, AstcChannel::UnormSrgb),

        // ── ETC2 family (Android fallback) ──
        Ktx2Format::Etc2Rgb8Unorm => (Tf::Etc2Rgb8Unorm, etc2),
        Ktx2Format::Etc2Rgb8UnormSrgb => (Tf::Etc2Rgb8UnormSrgb, etc2),
        Ktx2Format::Etc2Rgba8Unorm => (Tf::Etc2Rgba8Unorm, etc2),
        Ktx2Format::Etc2Rgba8UnormSrgb => (Tf::Etc2Rgba8UnormSrgb, etc2),

        // ── Out-of-subset VkFormat preserved by the parser ──
        Ktx2Format::Unsupported(raw) => return Err(FormatError::UnsupportedVkFormat(raw)),

        // Reachable only by a future `#[non_exhaustive]` addition that
        // forgot an arm above — a renderer bug, surfaced loudly.
        _ => return Err(FormatError::UnmappedKtx2Format),
    };

    Ok(mapped)
}

/// The subset of texture-format adapter features the renderer cares
/// about, captured from a live device once at startup (W2.T1.5). The
/// loader (W2.T4) consults it to pick the richest tier the GPU can
/// actually sample — e.g. a desktop adapter reports BC and uploads the
/// `Desktop` BC7 artifact; a WebGPU adapter reports none and falls
/// back to the uncompressed `Constrained` tier.
///
/// Stores the masked [`wgpu::Features`] rather than loose booleans so
/// [`Self::supports`] can reuse [`wgpu_format_from_ktx2_format`] as the
/// single source of "which feature does this format need" — no second
/// format→feature table to keep in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionFeatureSet {
    features: wgpu::Features,
}

impl CompressionFeatureSet {
    /// Mask of the texture-format features that gate KTX2 uploads.
    fn relevant_mask() -> wgpu::Features {
        wgpu::Features::TEXTURE_COMPRESSION_BC
            | wgpu::Features::TEXTURE_COMPRESSION_ASTC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
    }

    /// Capture the relevant subset of a device's advertised features.
    #[must_use]
    pub fn from_features(device_features: wgpu::Features) -> Self {
        Self {
            features: device_features & Self::relevant_mask(),
        }
    }

    /// `true` if the device can sample BC-compressed textures (desktop).
    #[must_use]
    pub fn bc(self) -> bool {
        self.features
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
    }

    /// `true` if the device can sample ASTC LDR textures (mobile / Apple).
    #[must_use]
    pub fn astc(self) -> bool {
        self.features
            .contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC)
    }

    /// `true` if the device can sample ETC2 textures (Android fallback).
    #[must_use]
    pub fn etc2(self) -> bool {
        self.features
            .contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2)
    }

    /// `true` if this device can legally sample `fmt`. Uncompressed
    /// core formats need no feature (an empty requirement is always
    /// satisfied), so they always return `true`; compressed and
    /// 16-bit-norm formats require their family feature. An
    /// [`Ktx2Format::Unsupported`] is never sampleable → `false`.
    #[must_use]
    pub fn supports(self, fmt: Ktx2Format) -> bool {
        match wgpu_format_from_ktx2_format(fmt) {
            Ok((_, required)) => self.features.contains(required),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::Features;

    #[test]
    fn empty_device_supports_only_core_uncompressed() {
        let set = CompressionFeatureSet::from_features(Features::empty());
        assert!(!set.bc() && !set.astc() && !set.etc2());
        // Core uncompressed: always sampleable.
        assert!(set.supports(Ktx2Format::Rgba8Unorm));
        assert!(set.supports(Ktx2Format::Rgba8UnormSrgb));
        assert!(set.supports(Ktx2Format::Rgba16Float));
        assert!(set.supports(Ktx2Format::Rgba32Float));
        // 16-bit-norm + every compressed family: gated → unsupported.
        assert!(!set.supports(Ktx2Format::Rgba16Unorm));
        assert!(!set.supports(Ktx2Format::Bc7RgbaUnorm));
        assert!(!set.supports(Ktx2Format::Astc4x4RgbaUnorm));
        assert!(!set.supports(Ktx2Format::Etc2Rgba8Unorm));
        // Unsupported VkFormat is never sampleable.
        assert!(!set.supports(Ktx2Format::Unsupported(0xDEAD)));
    }

    #[test]
    fn bc_only_device_is_a_desktop_adapter() {
        let set = CompressionFeatureSet::from_features(Features::TEXTURE_COMPRESSION_BC);
        assert!(set.bc() && !set.astc() && !set.etc2());
        assert!(set.supports(Ktx2Format::Bc7RgbaUnorm));
        assert!(set.supports(Ktx2Format::Bc4RUnorm));
        assert!(set.supports(Ktx2Format::Rgba8Unorm)); // core still fine
        assert!(!set.supports(Ktx2Format::Astc6x6RgbaUnorm));
        assert!(!set.supports(Ktx2Format::Etc2Rgb8Unorm));
    }

    #[test]
    fn from_features_masks_off_irrelevant_features() {
        // An unrelated feature must not leak into the captured set.
        let set = CompressionFeatureSet::from_features(
            Features::TEXTURE_COMPRESSION_ASTC | Features::DEPTH_CLIP_CONTROL,
        );
        assert!(set.astc() && !set.bc() && !set.etc2());
        assert!(set.supports(Ktx2Format::Astc8x8RgbaUnormSrgb));
    }

    #[test]
    fn fully_capable_device_supports_every_concrete_variant() {
        let set = CompressionFeatureSet::from_features(
            Features::TEXTURE_COMPRESSION_BC
                | Features::TEXTURE_COMPRESSION_ASTC
                | Features::TEXTURE_COMPRESSION_ETC2
                | Features::TEXTURE_FORMAT_16BIT_NORM,
        );
        assert!(set.bc() && set.astc() && set.etc2());
        // Unsupported stays unsampleable even on a fully-capable device.
        assert!(!set.supports(Ktx2Format::Unsupported(42)));
        for fmt in [
            Ktx2Format::Rgba16Unorm,
            Ktx2Format::Bc6hRgbSfloat,
            Ktx2Format::Astc4x4RgbaUnorm,
            Ktx2Format::Etc2Rgba8UnormSrgb,
        ] {
            assert!(set.supports(fmt), "full device should sample {fmt:?}");
        }
    }
}

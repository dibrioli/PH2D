//! Color profile tagging for image I/O. **Cap FROZEN at 8 variants** by
//! ADR-0054 §2.1 arch-gate `color_profile_variant_count_is_capped`.
//!
//! Per ADR-0054 §2.3 the internal Painter canonical is OKLCH/linear-f32
//! (see [`ph2d_color`]). This enum names *encoded* color spaces seen on
//! the wire — what the bytes claim before conversion. The importer
//! recognizes the profile (from embedded ICC or format default), converts
//! to internal linear/OKLCH, and the exporter does the reverse using the
//! caller's [`crate::ExportOpts::color_profile`].

/// Color profile attached to an [`crate::ImageBuffer`] or
/// [`crate::ExportOpts`].
///
/// `Custom` preserves the ICC byte blob byte-exact for byte-exact
/// round-trip (PSD/TIFF profiles are user-authored; corrupting them on
/// re-export is a data-loss bug).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ColorProfile {
    /// IEC 61966-2-1 sRGB. **Wave 1 default** (assumed when no ICC is
    /// embedded — sufficient for foto comum + synthetic fixtures).
    #[default]
    Srgb,
    /// SMPTE DCI-P3 D65 (Apple Display P3 variant). iPad shoot + modern
    /// monitors. Activated in W2 with `qcms` parsing.
    DisplayP3,
    /// Adobe RGB (1998). Adobe ecosystem (PS/Lightroom). W2.
    AdobeRgb,
    /// ProPhoto RGB. ROMM. Wide-gamut photography pipeline. W2.
    ProPhoto,
    /// Linear Rec.709 primaries. HDR scene-linear (EXR/HDR Radiance). W3.
    LinearRec709,
    /// Linear Rec.2020 primaries. HDR wide-gamut (AVIF-HDR/JXL-HDR). W3.
    LinearRec2020,
    /// Arbitrary ICC profile preserved byte-exact for round-trip
    /// fidelity (PSD/TIFF user-authored profiles).
    Custom(Vec<u8>),
    /// Source did not declare a profile and no fallback was inferable.
    /// Most importers default to `Srgb` semantically rather than emit
    /// `Unknown`, so this variant signals "explicitly absent".
    Unknown,
}

impl ColorProfile {
    /// `true` if this profile is linear-light (encoded as scene-linear
    /// floats). Determines whether HDR tone-mapping is needed on export
    /// to an LDR-only format.
    #[must_use]
    pub fn is_linear(&self) -> bool {
        matches!(self, Self::LinearRec709 | Self::LinearRec2020)
    }

    /// `true` if this profile has a transfer function approximating
    /// sRGB (gamma ~2.2). Used by exporters to skip a redundant
    /// gamma-encode step.
    #[must_use]
    pub fn is_srgb_like(&self) -> bool {
        matches!(self, Self::Srgb | Self::DisplayP3)
    }
}

//! Import / export options. **`ExportOpts` cap FROZEN at 6 fields** by
//! ADR-0054 §2.1 arch-gate `export_opts_field_count_is_capped`.
//!
//! `ImportOpts` is intentionally smaller (3 fields) because importers
//! choose almost everything from the source bytes — opts only convey
//! *caller preferences* the importer might honour or ignore.

use crate::ColorProfile;

/// Hints to [`crate::ImageImporter::supports`]. The two flavours cover
/// the two real-world recognition paths: peek at magic bytes (preferred,
/// no filename needed) or fall back to extension (fast-path when the
/// shell already knows the path).
///
/// **Bytes contract**: the slice MUST be ≥ 32 bytes (or the full file if
/// shorter — i.e. EOF). 32 covers every format's recognition window in
/// W1–W3: PNG signature 8, JPEG SOI 2, WebP RIFF+size+WEBP 12, GIF 6,
/// PSD header 26, AVIF ftyp box up to ~28, EXR magic 4, JXL signature
/// 12. Importer impls MAY check `src.len()` and return
/// `MagicMatch::None` defensively.
#[derive(Debug, Clone, Copy)]
pub enum MagicHint<'a> {
    /// First N bytes of the file (≥ 32 or EOF).
    Bytes(&'a [u8]),
    /// File extension without leading `.`, lowercase (`"png"`, `"jpeg"`,
    /// `"tiff"`). The importer is free to ignore this and only honour
    /// magic bytes — `Bytes` is authoritative. Returned as
    /// [`MagicMatch::Weak`] by convention so a `Bytes` match elsewhere
    /// wins dispatch.
    Extension(&'a str),
}

/// Confidence returned by [`crate::ImageImporter::supports`]. Audit
/// finding A-M1 (2026-05-26): with raw `bool`, two importers that match
/// the same `MagicHint` (e.g. PSB+PSD share a magic prefix; TIFF+DNG
/// share `II*\0`) dispatched first-wins by registry order — wrong
/// importer could win silently. With confidence, [`crate::ImporterRegistry::find_for`]
/// returns the strongest match across all importers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MagicMatch {
    /// Magic bytes uniquely identify this format. Wins dispatch over
    /// any [`Weak`](Self::Weak) match.
    Strong,
    /// Extension or partial magic match — usable when no
    /// [`Strong`](Self::Strong) is available.
    Weak,
    /// This importer does not recognize the hint.
    #[default]
    None,
}

/// Caller preferences for [`crate::ImageImporter::import`]. Importers
/// MAY ignore any field whose semantics they do not support; for
/// example `preserve_vector: true` against a JPEG importer is a no-op,
/// not an error.
///
/// **Color-profile policy** (audit A-H5): if `target_color_profile` is
/// `Some` and the importer cannot satisfy it (W1 has no ICC pipeline
/// yet — that lands W2.0.1), the importer SHOULD set
/// [`ColorProfileStrictness::Lenient`] in caller code to skip silently
/// or use [`ColorProfileStrictness::Strict`] to receive
/// [`crate::Error::Unsupported`] instead.
#[derive(Debug, Clone, Default)]
pub struct ImportOpts {
    /// If `Some`, convert the decoded buffer's color profile to this on
    /// import (single conversion at the boundary). `None` keeps the
    /// source profile and defers conversion to the engine.
    pub target_color_profile: Option<ColorProfile>,
    /// `Strict` => importer returns `Error::Unsupported` if it cannot
    /// satisfy `target_color_profile`. `Lenient` (default) => silently
    /// keeps source profile. ADR-0054 §2.3 W1 = `Lenient` is the
    /// realistic shipping default; W2.0.1+ may flip per-project.
    pub color_profile_strictness: ColorProfileStrictness,
    /// `true` = try to preserve layer structure if the format carries
    /// it (PSD/ORA → `Layered`); `false` = flatten on decode (always
    /// `Flat`/`FlatHdr`).
    pub preserve_layers: bool,
    /// `true` = try to preserve vector structure if the format carries
    /// it (SVG/PDF → `Vector`); `false` = rasterize on decode at
    /// importer's default DPI.
    pub preserve_vector: bool,
}

/// How strictly the importer must honour
/// [`ImportOpts::target_color_profile`]. See `ImportOpts` doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorProfileStrictness {
    /// Silently keep source profile if conversion is unavailable
    /// (default — covers W1 where no ICC pipeline exists yet).
    #[default]
    Lenient,
    /// Fail with [`crate::Error::Unsupported`] if conversion is not
    /// available.
    Strict,
}

/// Caller specification for [`crate::ImageExporter::export`].
/// **Cap FROZEN at 6 fields** by arch-gate.
#[derive(Debug, Clone)]
pub struct ExportOpts {
    /// Format identifier. Dispatched against
    /// [`crate::ImageExporter::supports_format`]. Audit A-H1 (2026-05-26):
    /// `String` was replaced with this enum so a typo (`"jepg"`) is a
    /// compile error, not a silent `find_for` miss.
    pub format: ExportFormat,
    /// Quality knob ∈ [0, 100] for lossy formats (JPEG/WebP-lossy/
    /// AVIF/JXL). Ignored by lossless formats; exporter sets sane
    /// defaults if asked for an out-of-range value.
    pub quality: u8,
    /// Target color profile of the exported bytes. Source pixels are
    /// converted to this on export. `Custom(icc)` writes the bytes
    /// byte-exact into the file's ICC chunk for round-trip fidelity.
    pub color_profile: ColorProfile,
    /// `true` = export layers if the format carries them (PSD/ORA);
    /// `false` = flatten before encode.
    pub preserve_layers: bool,
    /// HDR tone-mapping operator applied when exporting a `FlatHdr`
    /// source to an LDR-only format. `None` = refuse with
    /// [`crate::Error::HdrUnsupported`] instead of silently clipping.
    pub tone_map: ToneMap,
    /// Metadata policy (audit A-M3): was `bool` (all-or-nothing),
    /// promoted to enum so privacy-conscious users can strip EXIF
    /// GPS/serial without dropping ICC.
    pub metadata: MetadataPolicy,
}

impl Default for ExportOpts {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            quality: 90,
            color_profile: ColorProfile::Srgb,
            preserve_layers: false,
            tone_map: ToneMap::None,
            metadata: MetadataPolicy::All,
        }
    }
}

/// HDR → LDR tone-mapping operator used by [`ExportOpts::tone_map`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneMap {
    /// No tone-mapping — exporter refuses HDR→LDR with
    /// [`crate::Error::HdrUnsupported`].
    #[default]
    None,
    /// Reinhard operator (`x / (1 + x)` per channel). Cheap, neutral.
    Reinhard,
    /// ACES filmic (Knarkowicz approximation). More cinematic.
    Aces,
}

/// Metadata policy for [`ExportOpts::metadata`]. Audit A-M3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetadataPolicy {
    /// Write all available metadata (EXIF, XMP, ICC, …).
    #[default]
    All,
    /// Strip privacy-sensitive metadata (GPS, camera serial,
    /// timestamps, software-version) but keep ICC + color metadata.
    StripPrivacy,
    /// Write no metadata.
    None,
}

/// Format identifier used in [`ExportOpts::format`] and
/// [`crate::ImageExporter::supports_format`]. Covers every format in
/// the W1–W3 fan-out plus `Ph2dNative` (W1.T5). Adding a format =
/// adding a variant + the corresponding `crates/ph2d-imageio-<slug>/`.
/// Not arch-gated (data model, not trait surface — see ADR-0054 §2.1
/// non-capped clarification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// W1.T1 — Portable Network Graphics (lossless RGBA).
    Png,
    /// W1.T2 — JPEG (lossy RGB).
    Jpeg,
    /// W1.T3 — WebP (lossy + lossless + alpha + anim).
    Webp,
    /// W1.T4 — GIF (paletted, animation).
    Gif,
    /// W1.T5 — PH2D native (postcard + blake3, layers + history).
    Ph2dNative,
    /// W2.T1 — OpenRaster (Krita/MyPaint layered ZIP).
    Ora,
    /// W2.T2 — Tagged Image File Format (16-bit, CMYK, multi-page).
    Tiff,
    /// W2.T3 — Animated PNG (lossless anim + alpha).
    Apng,
    /// W2.T4 — Photoshop Document (multi-layer, blend modes).
    Psd,
    /// W3.T1 — JPEG XL (HDR, lossless, modern).
    Jxl,
    /// W3.T2 — OpenEXR (HDR float linear, multi-channel).
    Exr,
    /// W3.T3 — Radiance HDR (RGBE encoding, IBL/skybox).
    HdrRadiance,
    /// W3.T4 — AV1 Image File (HDR, modern, royalty-light).
    Avif,
    /// W3.T5 — Scalable Vector Graphics (BezPath, paint stack).
    Svg,
}

impl ExportFormat {
    /// Canonical lowercase file extension without leading dot.
    /// Drives file-dialog filter derivation (audit C-H1 follow-up
    /// for W1.T1).
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Ph2dNative => "ph2d",
            Self::Ora => "ora",
            Self::Tiff => "tiff",
            Self::Apng => "apng",
            Self::Psd => "psd",
            Self::Jxl => "jxl",
            Self::Exr => "exr",
            Self::HdrRadiance => "hdr",
            Self::Avif => "avif",
            Self::Svg => "svg",
        }
    }

    /// IANA MIME type. Used by browser-target shells (web) +
    /// clipboard rich-paste paths.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
            Self::Ph2dNative => "application/x-ph2d",
            Self::Ora => "image/openraster",
            Self::Tiff => "image/tiff",
            Self::Apng => "image/apng",
            Self::Psd => "image/vnd.adobe.photoshop",
            Self::Jxl => "image/jxl",
            Self::Exr => "image/x-exr",
            Self::HdrRadiance => "image/vnd.radiance",
            Self::Avif => "image/avif",
            Self::Svg => "image/svg+xml",
        }
    }
}

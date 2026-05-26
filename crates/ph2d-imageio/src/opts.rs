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
#[derive(Debug, Clone, Copy)]
pub enum MagicHint<'a> {
    /// First N bytes of the file (16 is typically enough for every
    /// format in W1–W3).
    Bytes(&'a [u8]),
    /// File extension without leading `.`, lowercase (`"png"`, `"jpeg"`,
    /// `"tiff"`). The importer is free to ignore this and only honour
    /// magic bytes — `Bytes` is authoritative.
    Extension(&'a str),
}

/// Caller preferences for [`crate::ImageImporter::import`]. Importers
/// MAY ignore any field whose semantics they do not support; for
/// example `preserve_vector: true` against a JPEG importer is a no-op,
/// not an error.
#[derive(Debug, Clone, Default)]
pub struct ImportOpts {
    /// If `Some`, convert the decoded buffer's color profile to this on
    /// import (single conversion at the boundary). `None` keeps the
    /// source profile and defers conversion to the engine.
    pub target_color_profile: Option<ColorProfile>,
    /// `true` = try to preserve layer structure if the format carries
    /// it (PSD/ORA → `Layered`); `false` = flatten on decode (always
    /// `Flat`/`FlatHdr`).
    pub preserve_layers: bool,
    /// `true` = try to preserve vector structure if the format carries
    /// it (SVG/PDF → `Vector`); `false` = rasterize on decode at
    /// importer's default DPI.
    pub preserve_vector: bool,
}

/// Caller specification for [`crate::ImageExporter::export`].
/// **Cap FROZEN at 6 fields** by arch-gate.
#[derive(Debug, Clone)]
pub struct ExportOpts {
    /// Format identifier, lowercase (`"png"`, `"jpeg"`, `"webp"`).
    /// Dispatched against [`crate::ImageExporter::supports_format`].
    pub format: String,
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
    /// `true` = preserve EXIF/XMP metadata (only honoured by formats
    /// that carry it: JPEG/TIFF/WebP).
    pub metadata: bool,
}

impl Default for ExportOpts {
    fn default() -> Self {
        Self {
            format: String::new(),
            quality: 90,
            color_profile: ColorProfile::Srgb,
            preserve_layers: false,
            tone_map: ToneMap::None,
            metadata: true,
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

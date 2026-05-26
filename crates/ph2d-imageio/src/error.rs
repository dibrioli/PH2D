//! Image I/O error type. **Cap FROZEN at 8 variants** by ADR-0054 §2.1
//! arch-gate `error_variant_count_is_capped`.
//!
//! Variants name failure *categories*, not formats. Per-format detail goes
//! in the inner `String` payload (decode/encode messages) so this enum
//! stays small.

use std::fmt;

/// Error raised by [`crate::ImageImporter::import`] or
/// [`crate::ImageExporter::export`].
#[derive(Debug)]
pub enum Error {
    /// Bytes are syntactically corrupted or violate the format spec.
    /// Inner string is the importer's own diagnostic — propagated as-is
    /// for user-facing toast / log.
    Decode(String),
    /// Encoder rejected the input (e.g. JPEG asked to encode RGBA → drop
    /// alpha warning, or PNG asked to encode HDR → no path).
    Encode(String),
    /// The file format is recognized but this implementation does not
    /// handle it (e.g. PSD v2.0+ specific layer types, AVIF lossless
    /// without rav1e support).
    Unsupported(String),
    /// Unexpected EOF reading the source bytes.
    Truncated,
    /// Embedded ICC profile is malformed; importer falls back to
    /// `ColorProfile::Unknown` and continues, or returns this error if
    /// the caller's `ImportOpts::target_color_profile` is `Some` and
    /// strict mode is implied.
    IccCorrupted,
    /// Caller asked for `preserve_layers: true` but the source format
    /// is flat (PNG/JPEG/etc.), or vice versa.
    MissingLayer,
    /// Exporter is sRGB-only (PNG-8 / JPEG / GIF / BMP) and received a
    /// `DecodedImage::FlatHdr` with `ToneMap::None` — refuse instead of
    /// silently clipping highlights.
    HdrUnsupported,
    /// Escape hatch. Used sparingly; prefer the specific variants above.
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Decode(s) => write!(f, "image decode error: {s}"),
            Error::Encode(s) => write!(f, "image encode error: {s}"),
            Error::Unsupported(s) => write!(f, "unsupported format detail: {s}"),
            Error::Truncated => f.write_str("unexpected EOF reading image bytes"),
            Error::IccCorrupted => f.write_str("embedded ICC profile is corrupted"),
            Error::MissingLayer => f.write_str("layer information missing or mismatched"),
            Error::HdrUnsupported => {
                f.write_str("HDR image cannot be exported to this LDR-only format without tone-map")
            }
            Error::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

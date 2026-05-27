//! Image I/O error type. **Cap RAISED to 11 variants** by audit
//! finding A-H4 (2026-05-26); ADR-0054 §2.1 arch-gate
//! `error_variant_count_is_capped`.
//!
//! Variants name failure *categories*, not formats. Per-format detail
//! goes in the inner `String` payload (decode/encode messages) so this
//! enum stays small.

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
    /// without rav1e support). Also raised when
    /// [`crate::ImportOpts::color_profile_strictness`] is `Strict` and
    /// the importer cannot satisfy the target profile (audit A-H5).
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
    /// Allocation refused — e.g. EXR 16K×16K float = 4 GiB exceeds
    /// process/budget limit. Added by audit A-H4 so callers can
    /// distinguish "we can't decode this file size" from
    /// `Decode(String)` and skip retry. (HR-13 memory budget surface.)
    OutOfMemory,
    /// Source declares dimensions that exceed our hard cap (decompression
    /// bomb defence — PNG IHDR claiming 65535×65535 with 1 byte/pixel
    /// would fill 4 GiB). Inner string carries the offending dimension
    /// for log. Added by audit A-H4.
    DimensionExceedsLimit,
    /// Long-running decode was cancelled by the caller (e.g. user
    /// hit Esc on a slow PSD parse, or a cancellation token fired).
    /// Importers MAY honour cancellation cooperatively; PR opens W1+.
    /// Added by audit A-H4.
    Cancelled,
    /// Escape hatch. Used sparingly; prefer the specific variants above.
    Custom(String),
}

impl Error {
    /// Fluent message key for user-facing display (HR-15). The shell's
    /// i18n layer resolves these against `locales/<lang>.ftl`. Inner
    /// string payloads (when present) become Fluent variables.
    ///
    /// Audit E-M1 (2026-05-26): `Display` impl returns hardcoded English
    /// for dev/log usage; user-facing UI MUST route through `fluent_key`
    /// + Fluent bundle.
    #[must_use]
    pub const fn fluent_key(&self) -> &'static str {
        match self {
            Self::Decode(_) => "imageio.error.decode",
            Self::Encode(_) => "imageio.error.encode",
            Self::Unsupported(_) => "imageio.error.unsupported",
            Self::Truncated => "imageio.error.truncated",
            Self::IccCorrupted => "imageio.error.icc-corrupted",
            Self::MissingLayer => "imageio.error.missing-layer",
            Self::HdrUnsupported => "imageio.error.hdr-unsupported",
            Self::OutOfMemory => "imageio.error.out-of-memory",
            Self::DimensionExceedsLimit => "imageio.error.dimension-exceeds-limit",
            Self::Cancelled => "imageio.error.cancelled",
            Self::Custom(_) => "imageio.error.custom",
        }
    }
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
            Error::OutOfMemory => {
                f.write_str("image too large to allocate (exceeds process/budget memory)")
            }
            Error::DimensionExceedsLimit => {
                f.write_str("image dimensions exceed hard cap (decompression-bomb defence)")
            }
            Error::Cancelled => f.write_str("decode/encode was cancelled by the caller"),
            Error::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

// Audit B-M1 (2026-05-26): the previously-shipped `From<std::io::Error>`
// impl was unused — every format crate calls
// `.map_err(|e| Error::Decode(e.to_string()))` on `image::ImageError`
// (not `io::Error`). The impl encouraged the wrong refactor (replace
// `image` map_err with `?`, which fails: `image::ImageError ≠ io::Error`)
// and surfaced as dead code. Removed; if a future format crate needs
// `io::Error` propagation it adds a one-line `.map_err` next to its
// own code where the call site is obvious.

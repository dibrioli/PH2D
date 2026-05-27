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
///
/// `#[non_exhaustive]` (audit-7 Lens I HIGH I-4): variants in the cap
/// FROZEN budget may not all be wired up yet (`IccCorrupted`,
/// `Cancelled`, `Custom` currently emitted by zero format crates).
/// Future-proofing the match-exhaustiveness lets us wire them without
/// a breaking change to downstream callers.
#[derive(Debug)]
#[non_exhaustive]
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

impl Error {
    /// Audit-7 Lens I HIGH I-3 (2026-05-26): map an upstream decoder
    /// message to either `Error::Truncated` (if it looks EOF-shaped)
    /// or `Error::Decode(msg)` otherwise. Centralized so every format
    /// crate produces the same semantics — callers that distinguish
    /// "user cancelled download" from "file corrupted" via
    /// `match err { Truncated => retry; Decode => fail }` don't need
    /// to learn per-format quirks.
    ///
    /// EOF signature set is empirical: `image-rs`, `png`, `tiff`,
    /// `psd`, `zip`, `postcard` all emit one of these phrases when
    /// they hit an incomplete stream.
    #[must_use]
    pub fn from_decoder_message(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        let lower = msg.to_lowercase();
        // Audit-8 Lens O O-2 + Lens L L-4 (2026-05-26): refined
        // signature set. `"end of stream"` was a false positive in
        // flate2 corruption paths ("end of stream marker found");
        // tightened to `"unexpected end of stream"` (image-rs/zip
        // exact). Added `"truncated"` / `"incomplete"` /
        // `"premature end"` / `"end-of-file"` (with hyphen) per
        // image-rs, psd, postcard, and zip empirical messages.
        if lower.contains("unexpected end of file")
            || lower.contains("unexpected end-of-file")
            || lower.contains("unexpected eof")
            || lower.contains("eof while parsing")
            || lower.contains("unexpected end of stream")
            || lower.contains("eof reached")
            || lower.contains("not enough data")
            || lower.contains("truncated")
            || lower.contains("incomplete")
            || lower.contains("premature end")
        {
            Self::Truncated
        } else {
            Self::Decode(msg)
        }
    }
}

// Audit B-M1 (2026-05-26): the previously-shipped `From<std::io::Error>`
// impl was unused — every format crate calls
// `.map_err(|e| Error::Decode(e.to_string()))` on `image::ImageError`
// (not `io::Error`). The impl encouraged the wrong refactor (replace
// `image` map_err with `?`, which fails: `image::ImageError ≠ io::Error`)
// and surfaced as dead code. Removed; if a future format crate needs
// `io::Error` propagation it adds a one-line `.map_err` next to its
// own code where the call site is obvious.

#[cfg(test)]
mod tests {
    use super::*;

    /// Audit-8 Lens M (2026-05-26): table-driven gate for the EOF
    /// classifier. Without this, a refactor that drops one of the 10
    /// substring matches (or accidentally case-folds an upstream
    /// signature differently) regresses silently.
    #[test]
    fn from_decoder_message_classifies_known_eof_signals_to_truncated() {
        for sig in [
            "unexpected end of file",
            "Unexpected END OF FILE", // case-insensitive
            "unexpected end-of-file",
            "unexpected eof",
            "EOF while parsing PNG IHDR",
            "got unexpected end of stream",
            "eof reached before frame complete",
            "not enough data for image header",
            "truncated chunk at offset 42",
            "file is incomplete",
            "premature end of zip central directory",
        ] {
            assert!(
                matches!(Error::from_decoder_message(sig), Error::Truncated),
                "expected Truncated for {sig:?}"
            );
        }
    }

    /// Audit-8 Lens M: non-EOF signatures must fall through to
    /// `Decode` — false-positive in flate2's `"end of stream marker"`
    /// (which is NOT EOF but corruption) was the audit-8 O-2 fix.
    #[test]
    fn from_decoder_message_classifies_non_eof_to_decode() {
        for sig in [
            "checksum mismatch in PNG chunk",
            "invalid TIFF tag 0x1234",
            "end of stream marker found", // O-2: not Truncated (corruption)
            "unsupported color type",
            "malformed acTL chunk length",
        ] {
            assert!(
                matches!(Error::from_decoder_message(sig), Error::Decode(_)),
                "expected Decode for {sig:?}"
            );
        }
    }
}

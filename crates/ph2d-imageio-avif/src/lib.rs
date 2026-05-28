#![forbid(unsafe_code)]
//! `ph2d-imageio-avif` — AVIF magic recognition + (deferred) decode
//! + encode (W3.T4).
//!
//! W3.T4 placeholder: recognizes AVIF magic via `ftyp` box detection
//! so the registry dispatches here for `.avif` files; both import
//! and export return `Error::Unsupported` with actionable messages
//! pointing to the next wave when a real client (Painter HDR import
//! demo) triggers full wire-up.
//!
//! Pure-Rust decode path candidates: `avif-decode = "1"` (read-only,
//! built on `dav1d`-equivalent Rust). Pure-Rust encode candidate:
//! `ravif = "0.11"` (rav1e backend). Both add substantial dep
//! footprint (~20 transitive crates each); deferred so the W3
//! fan-out lands at zero dep-bloat for crates the user doesn't
//! invoke yet.
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.
//! Magic-only stub matches the PSD pattern (W2.T4 shipped magic
//! recognition + Unsupported export until a real PSD-write client
//! materialised — same disposition here).

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MagicHint, MagicMatch,
};

/// AVIF magic: ISOBMFF `ftyp` box starting at byte 4, brand `avif`
/// or `avis` (still / sequence) at byte 8.
fn is_avif_magic(b: &[u8]) -> bool {
    if b.len() < 12 {
        return false;
    }
    // Bytes 4-7 = "ftyp".
    if &b[4..8] != b"ftyp" {
        return false;
    }
    // Bytes 8-11 = "avif" (still) or "avis" (sequence).
    matches!(&b[8..12], b"avif" | b"avis")
}

/// Register the AVIF importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(AvifImporter));
}

/// Register the AVIF exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(AvifExporter));
}

/// AVIF import driver. W3.T4 placeholder — recognizes magic but
/// returns Unsupported pointing the caller at the next wave.
pub struct AvifImporter;

impl ImageImporter for AvifImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_avif_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("avif") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        Err(Error::Unsupported(
            "AVIF decode deferred to W3+ — pure-Rust `avif-decode = \"1\"` \
             wire-up lands when first real client (Painter HDR import demo) \
             appears. Convert to PNG/WebP/EXR via external tool (cavif-rs, \
             ffmpeg) for now."
                .into(),
        ))
    }
}

/// AVIF export driver. W3.T4 placeholder.
pub struct AvifExporter;

impl ImageExporter for AvifExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Avif)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "AVIF encode deferred to W3+ — pure-Rust `ravif = \"0.11\"` \
             (rav1e backend) wire-up lands when first real client \
             (Painter HDR export demo) appears."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_avif_ftyp_strong() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_avis_sequence_strong() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b's', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_rejects_heif_with_none() {
        // HEIF same container but brand=mif1/heic — must NOT match.
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', 0x00, 0x00,
        ];
        assert_eq!(
            AvifImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::None
        );
    }

    #[test]
    fn supports_recognizes_avif_extension_weak() {
        assert_eq!(
            AvifImporter.supports(MagicHint::Extension("avif")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = AvifImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_returns_unsupported_with_actionable_message() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("decode deferred");
        let msg = match err {
            Error::Unsupported(s) => s,
            other => panic!("expected Unsupported, got {other:?}"),
        };
        assert!(msg.contains("avif-decode"), "actionable: {msg}");
    }

    /// Audit-13 Lens FF F4 (2026-05-27): cover `avis` sequence brand
    /// import path too — previous test only exercised still `avif`.
    #[test]
    fn import_avis_sequence_returns_unsupported_deferred() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b's',
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("avis sequence decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn exporter_returns_unsupported() {
        // `AvifExporter::export` returns Unsupported BEFORE looking
        // at the image payload — `DecodedImage::Animated(vec![])` is
        // the cheapest variant to construct in a test that doesn't
        // import ph2d-color.
        let err = AvifExporter.export(
            &DecodedImage::Animated(vec![]),
            &ExportOpts {
                format: ExportFormat::Avif,
                ..ExportOpts::default()
            },
        );
        assert!(matches!(err, Err(Error::Unsupported(_))));
    }

    #[test]
    fn exporter_recognizes_avif_format() {
        assert!(AvifExporter.supports_format(ExportFormat::Avif));
        assert!(!AvifExporter.supports_format(ExportFormat::Png));
    }
}

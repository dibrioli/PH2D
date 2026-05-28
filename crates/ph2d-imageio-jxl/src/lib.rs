#![forbid(unsafe_code)]
//! `ph2d-imageio-jxl` — JPEG XL magic recognition + (deferred)
//! decode/encode (W3.T1).
//!
//! W3.T1 placeholder: recognizes JXL magic (ISOBMFF container OR
//! raw codestream) so the registry dispatches here for `.jxl` files;
//! both import and export return `Error::Unsupported` with
//! actionable messages. Real decode lands when first real client
//! appears via `jxl-oxide = "0.10"` (pure-Rust); real encode lands
//! when a pure-Rust JXL encoder materialises (`zune-jxl`?).
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MagicHint, MagicMatch,
};

/// JXL ISOBMFF container magic (12 bytes).
const JXL_ISOBMFF: [u8; 12] = [
    0x00, 0x00, 0x00, 0x0C, b'J', b'X', b'L', b' ', 0x0D, 0x0A, 0x87, 0x0A,
];
/// JXL codestream (raw, no container).
const JXL_CODESTREAM: [u8; 2] = [0xFF, 0x0A];

pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(JxlImporter));
}

pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(JxlExporter));
}

pub struct JxlImporter;

impl ImageImporter for JxlImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b)
                if b.starts_with(&JXL_ISOBMFF) || b.starts_with(&JXL_CODESTREAM) =>
            {
                MagicMatch::Strong
            }
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("jxl") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        Err(Error::Unsupported(
            "JXL decode deferred to W3+ — pure-Rust `jxl-oxide = \"0.10\"` \
             wire-up lands with first real client (Painter HDR import \
             demo). Convert to PNG/WebP via external tool (cjxl, ffmpeg) \
             for now."
                .into(),
        ))
    }
}

pub struct JxlExporter;

impl ImageExporter for JxlExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Jxl)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "JXL encode deferred — `jxl-oxide` is decode-only as of 0.10. \
             Round-trip via PNG (LDR) or EXR (HDR); native JXL encode \
             lands when a pure-Rust encoder materialises."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_jxl_isobmff_strong() {
        assert_eq!(
            JxlImporter.supports(MagicHint::Bytes(&JXL_ISOBMFF)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_jxl_codestream_strong() {
        let bytes = [0xFF, 0x0A, 0x00, 0x01];
        assert_eq!(
            JxlImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_jxl_extension_weak() {
        assert_eq!(
            JxlImporter.supports(MagicHint::Extension("jxl")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = JxlImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_returns_unsupported_deferred() {
        let err = JxlImporter
            .import(&JXL_ISOBMFF, &ImportOpts::default())
            .expect_err("decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    /// Audit-13 Lens FF F4 (2026-05-27): cover codestream path too —
    /// previous test only exercised ISOBMFF.
    #[test]
    fn import_codestream_returns_unsupported_deferred() {
        let bytes = [0xFF, 0x0A, 0x00, 0x01, 0x02, 0x03];
        let err = JxlImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("codestream decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn exporter_recognizes_jxl_format() {
        assert!(JxlExporter.supports_format(ExportFormat::Jxl));
        assert!(!JxlExporter.supports_format(ExportFormat::Png));
    }

    #[test]
    fn exporter_returns_unsupported_deferred() {
        let err = JxlExporter
            .export(
                &DecodedImage::Animated(vec![]),
                &ExportOpts {
                    format: ExportFormat::Jxl,
                    ..ExportOpts::default()
                },
            )
            .expect_err("encode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}

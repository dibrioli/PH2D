#![forbid(unsafe_code)]
//! `ph2d-imageio-exr` — OpenEXR magic recognition + (deferred)
//! decode + encode (W3.T2).
//!
//! W3.T2 placeholder: recognizes EXR magic so the registry
//! dispatches here for `.exr` files; both import and export return
//! `Error::Unsupported` with actionable messages. Real decode/encode
//! lands when the first real client (Blender HDRi import demo)
//! materialises — `exr` 1.x API uses a closure-based pixel walker
//! that needs non-trivial RefCell out-parameters; defer until the
//! shape of `FlatHdr` consumers stabilises.
//!
//! Pure-Rust dep: `exr = "1"` (already in Cargo.toml).
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.
//! Magic-only stub matches the PSD pattern (W2.T4 shipped recognition
//! + Unsupported until a real PSD-write client materialised).

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MagicHint, MagicMatch,
};

/// EXR magic bytes: `0x76 0x2F 0x31 0x01` (little-endian `0x01312f76`).
const EXR_MAGIC: [u8; 4] = [0x76, 0x2F, 0x31, 0x01];

/// Register the EXR importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(ExrImporter));
}

/// Register the EXR exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(ExrExporter));
}

/// EXR import driver. W3.T2 placeholder.
pub struct ExrImporter;

impl ImageImporter for ExrImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&EXR_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("exr") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        Err(Error::Unsupported(
            "EXR decode deferred to W3+ — `exr = \"1\"` is in Cargo.toml \
             but the closure-based pixel walker API needs RefCell out-\
             parameters that don't justify the LOC in the magic-recognition \
             pass. Wire-up lands with first real client (Blender HDRi \
             import demo)."
                .into(),
        ))
    }
}

/// EXR export driver. W3.T2 placeholder.
pub struct ExrExporter;

impl ImageExporter for ExrExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Exr)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "EXR encode deferred to W3+ — `exr` 1.x API requires \
             SpecificChannels construction; wire-up lands with first \
             real client (Painter HDR export demo). Round-trip via \
             `.ph2d-native` preserves FlatHdr losslessly today."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_exr_magic_strong() {
        let bytes = [0x76, 0x2F, 0x31, 0x01, 0x00];
        assert_eq!(
            ExrImporter.supports(MagicHint::Bytes(&bytes)),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_exr_extension_weak() {
        assert_eq!(
            ExrImporter.supports(MagicHint::Extension("exr")),
            MagicMatch::Weak
        );
        assert_eq!(
            ExrImporter.supports(MagicHint::Extension("EXR")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = ExrImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_returns_unsupported_deferred() {
        let bytes = [0x76, 0x2F, 0x31, 0x01, 0x00];
        let err = ExrImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn exporter_recognizes_exr_format() {
        assert!(ExrExporter.supports_format(ExportFormat::Exr));
        assert!(!ExrExporter.supports_format(ExportFormat::Png));
    }

    #[test]
    fn exporter_returns_unsupported_deferred() {
        let err = ExrExporter
            .export(
                &DecodedImage::Animated(vec![]),
                &ExportOpts {
                    format: ExportFormat::Exr,
                    ..ExportOpts::default()
                },
            )
            .expect_err("encode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}

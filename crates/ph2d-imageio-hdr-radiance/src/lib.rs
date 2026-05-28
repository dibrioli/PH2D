#![forbid(unsafe_code)]
//! `ph2d-imageio-hdr-radiance` — Radiance HDR (`.hdr`) magic
//! recognition + (deferred) decode/encode (W3.T3).
//!
//! W3.T3 placeholder: recognizes Radiance HDR magic
//! (`#?RADIANCE\n` or legacy `#?RGBE\n`) so the registry dispatches
//! here for `.hdr` files; both import and export return
//! `Error::Unsupported` with actionable messages. Real decode/
//! encode lands when first real client (IBL skybox import demo)
//! materialises — `image` 0.25 `hdr` feature provides the codec.
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MagicHint, MagicMatch,
};

const HDR_MAGIC: &[u8] = b"#?RADIANCE";
const HDR_MAGIC_LEGACY: &[u8] = b"#?RGBE";

pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(HdrRadianceImporter));
}

pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(HdrRadianceExporter));
}

pub struct HdrRadianceImporter;

impl ImageImporter for HdrRadianceImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(HDR_MAGIC) || b.starts_with(HDR_MAGIC_LEGACY) => {
                MagicMatch::Strong
            }
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("hdr") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        Err(Error::Unsupported(
            "Radiance HDR decode deferred to W3+ — `image` 0.25 `hdr` \
             feature wire-up lands with first real client (IBL skybox \
             import demo)."
                .into(),
        ))
    }
}

pub struct HdrRadianceExporter;

impl ImageExporter for HdrRadianceExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::HdrRadiance)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "Radiance HDR encode deferred to W3+ — wire-up lands with \
             first real client (Painter HDR export demo)."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_hdr_magic_strong() {
        assert_eq!(
            HdrRadianceImporter.supports(MagicHint::Bytes(b"#?RADIANCE\n")),
            MagicMatch::Strong
        );
        assert_eq!(
            HdrRadianceImporter.supports(MagicHint::Bytes(b"#?RGBE\n")),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_hdr_extension_weak() {
        assert_eq!(
            HdrRadianceImporter.supports(MagicHint::Extension("hdr")),
            MagicMatch::Weak
        );
        assert_eq!(
            HdrRadianceImporter.supports(MagicHint::Extension("HDR")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = HdrRadianceImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_returns_unsupported_deferred() {
        let err = HdrRadianceImporter
            .import(
                b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n",
                &ImportOpts::default(),
            )
            .expect_err("decode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn exporter_recognizes_hdr_format() {
        assert!(HdrRadianceExporter.supports_format(ExportFormat::HdrRadiance));
        assert!(!HdrRadianceExporter.supports_format(ExportFormat::Png));
    }

    #[test]
    fn exporter_returns_unsupported_deferred() {
        let err = HdrRadianceExporter
            .export(
                &DecodedImage::Animated(vec![]),
                &ExportOpts {
                    format: ExportFormat::HdrRadiance,
                    ..ExportOpts::default()
                },
            )
            .expect_err("encode deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}

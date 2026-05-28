#![forbid(unsafe_code)]
//! `ph2d-imageio-avif` — AVIF decode (W3.T4).
//!
//! Pure-Rust decode via `avif-decode = "1"` (libaom-based — wraps
//! `aom_decode` + `avif-parse` + `yuv` for chroma upsampling).
//! Decodes still AVIF (`ftyp avif`) + sequence AVIF (`ftyp avis`,
//! first-frame only — Animated bridge deferred). Output is RGBA8
//! flat path (16-bit AVIF quantises to 8-bit via top-byte drop;
//! HDR PQ/HLG would require a FlatHdr bridge — deferred to W3+).
//!
//! Encode is **permanent-deferred** — pure-Rust encode via
//! `ravif = "0.13"` pulls ~50 transitive crates (rav1e dep tree);
//! wire-up only when first real export client (Painter HDR export
//! demo) appears.
//!
//! Audit-14 wave-2.1 (2026-05-27): real decode replaces the magic-
//! only-stub. Same pattern as JXL/EXR/HDR — strict subset shipped,
//! HDR/multi-frame deferred until canonical FlatHdr/Animated
//! bridges land. Mismatched HEIF brands (`mif1`/`heic`) rejected
//! at magic-byte level (silent-data-corruption defence).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};

/// AVIF magic: ISOBMFF `ftyp` box starting at byte 4, brand `avif`
/// (still) or `avis` (sequence) at byte 8.
fn is_avif_magic(b: &[u8]) -> bool {
    if b.len() < 12 {
        return false;
    }
    if &b[4..8] != b"ftyp" {
        return false;
    }
    matches!(&b[8..12], b"avif" | b"avis")
}

pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(AvifImporter));
}

pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(AvifExporter));
}

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
        // Audit-14 wave-2.1: defend against accidental decode of
        // HEIF (`mif1`/`heic`) — magic check already rejects them
        // but `Decoder::from_avif` might be permissive in some
        // edge cases. Double-check is cheap.
        if !is_avif_magic(src) {
            return Err(Error::Decode(
                "AVIF: ftyp brand is not `avif`/`avis` (HEIF would need ph2d-imageio-heif)".into(),
            ));
        }
        let decoder = avif_decode::Decoder::from_avif(src)
            .map_err(|e| Error::from_decoder_message(format!("AVIF decode init: {e}")))?;
        let image = decoder
            .to_image()
            .map_err(|e| Error::from_decoder_message(format!("AVIF decode pixels: {e}")))?;
        // Convert avif-decode's Image enum to our RGBA8 buffer.
        // 16-bit channels quantise to 8-bit via top-byte drop
        // (documented loss; same policy as PNG W1.T1).
        let (width, height, pixels) = match image {
            avif_decode::Image::Rgb8(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| SrgbRgba([p.r, p.g, p.b, 255]))
                    .collect();
                (w as u32, h as u32, pixels)
            }
            avif_decode::Image::Rgb16(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| SrgbRgba([(p.r >> 8) as u8, (p.g >> 8) as u8, (p.b >> 8) as u8, 255]))
                    .collect();
                (w as u32, h as u32, pixels)
            }
            avif_decode::Image::Rgba8(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| SrgbRgba([p.r, p.g, p.b, p.a]))
                    .collect();
                (w as u32, h as u32, pixels)
            }
            avif_decode::Image::Rgba16(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| {
                        SrgbRgba([
                            (p.r >> 8) as u8,
                            (p.g >> 8) as u8,
                            (p.b >> 8) as u8,
                            (p.a >> 8) as u8,
                        ])
                    })
                    .collect();
                (w as u32, h as u32, pixels)
            }
            avif_decode::Image::Gray8(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| SrgbRgba([p.0, p.0, p.0, 255]))
                    .collect();
                (w as u32, h as u32, pixels)
            }
            avif_decode::Image::Gray16(img) => {
                let (w, h) = (img.width(), img.height());
                let pixels: Vec<SrgbRgba> = img
                    .pixels()
                    .map(|p| {
                        let c = (p.0 >> 8) as u8;
                        SrgbRgba([c, c, c, 255])
                    })
                    .collect();
                (w as u32, h as u32, pixels)
            }
        };
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "AVIF has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// AVIF export driver. Permanent-deferred — `ravif = "0.13"` pulls
/// rav1e (~50 transitive crates); wire-up only when first real
/// export client (Painter HDR export demo) appears.
pub struct AvifExporter;

impl ImageExporter for AvifExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Avif)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "AVIF encode deferred to W3+ — pure-Rust `ravif = \"0.13\"` \
             (rav1e backend) pulls ~50 transitive crates; wire-up lands \
             with first real export client (Painter HDR export demo). \
             Convert via cavif CLI tool externally for now."
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

    /// Audit-14 wave-2.1 (2026-05-27): HEIF brand (`mif1`) with
    /// otherwise-valid ISOBMFF header must be rejected at the
    /// double-check inside `import`, not silently fed to
    /// avif-decode (which might be permissive).
    #[test]
    fn import_rejects_heif_brand_with_decode() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', 0x00, 0x00,
            0x00, 0x00,
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("HEIF must be rejected");
        match err {
            Error::Decode(msg) => assert!(
                msg.contains("HEIF") || msg.contains("ftyp"),
                "expected actionable HEIF rejection message, got: {msg}"
            ),
            other => panic!("expected Decode, got: {other:?}"),
        }
    }

    #[test]
    fn import_rejects_truncated_avif_with_actionable_error() {
        // ftyp avif header but no actual content.
        let bytes = [
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
        ];
        let err = AvifImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("truncated avif must error");
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
    }

    #[test]
    fn exporter_recognizes_avif_format() {
        assert!(AvifExporter.supports_format(ExportFormat::Avif));
        assert!(!AvifExporter.supports_format(ExportFormat::Png));
    }

    #[test]
    fn exporter_returns_unsupported_deferred() {
        let err = AvifExporter
            .export(
                &DecodedImage::Animated(vec![]),
                &ExportOpts {
                    format: ExportFormat::Avif,
                    ..ExportOpts::default()
                },
            )
            .expect_err("encode deferred");
        match err {
            Error::Unsupported(msg) => assert!(
                msg.contains("ravif"),
                "expected actionable ravif message: {msg}"
            ),
            other => panic!("expected Unsupported, got: {other:?}"),
        }
    }
}

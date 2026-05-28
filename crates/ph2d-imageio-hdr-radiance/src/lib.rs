#![forbid(unsafe_code)]
//! `ph2d-imageio-hdr-radiance` — Radiance HDR (`.hdr`) decode + encode
//! (W3.T3).
//!
//! Pure-Rust via `image` 0.25 `hdr` feature. Radiance RGBE format
//! stores HDR as 8-bit RGB + 8-bit shared exponent — older than
//! OpenEXR, simpler, still widely used for IBL skybox content
//! (Greg Ward, "High Dynamic Range Imaging" 2005).
//!
//! ### What W3.T3 covers
//!
//! - Magic: `#?RADIANCE\n` ASCII signature + legacy `#?RGBE\n`.
//! - Decode: RGBE → `ImageBuffer<LinearRgba>` via
//!   `image::codecs::hdr::HdrDecoder`. Alpha synthesized to 1.0
//!   (RGBE has no alpha channel).
//! - Encode: float32 RGB → RGBE via
//!   `image::codecs::hdr::HdrEncoder` (drops alpha — RGBE is RGB-only).
//! - HR-1: pure-Rust, no C bindings.
//! - HR-13: dimension cap via `ph2d_imageio::MAX_RASTER_DIMENSION`.
//!
//! Audit-13 W3.T1.5 wave-2 (2026-05-27): real impl replaces the
//! magic-only-stub from cc97cd4 — `image` 0.25 hdr feature is a
//! single dep + the decode/encode API is uniform with PNG/JPEG/WEBP.

use image::ImageDecoder;
use ph2d_color::LinearRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
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
        let cursor = std::io::Cursor::new(src);
        let decoder = image::codecs::hdr::HdrDecoder::new(cursor)
            .map_err(|e| Error::from_decoder_message(format!("HDR decode init: {e}")))?;
        let (width, height) = decoder.dimensions();
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "HDR has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // HdrDecoder produces ColorType::Rgb32F (3 floats per pixel).
        // Allocate the exact byte buffer + read.
        let total = decoder.total_bytes() as usize;
        let mut buf = vec![0u8; total];
        decoder
            .read_image(&mut buf)
            .map_err(|e| Error::from_decoder_message(format!("HDR decode pixels: {e}")))?;
        // Reinterpret bytes as f32 triples (3 floats per pixel, native
        // endian — image-rs decodes to host floats). RGBE has no alpha
        // → synthesize 1.0.
        let pixel_count = (width as usize) * (height as usize);
        let mut pixels = Vec::with_capacity(pixel_count);
        for chunk in buf.chunks_exact(12) {
            // Each pixel = 3 floats = 12 bytes.
            let r = f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let g = f32::from_ne_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            let b = f32::from_ne_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
            pixels.push(LinearRgba::new(r, g, b, 1.0));
        }
        Ok(DecodedImage::FlatHdr(ImageBuffer {
            width,
            height,
            pixels,
            color_profile: ColorProfile::LinearRec709,
        }))
    }
}

pub struct HdrRadianceExporter;

impl ImageExporter for HdrRadianceExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::HdrRadiance)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::FlatHdr(b) => b,
            DecodedImage::Flat(_) => {
                return Err(Error::Unsupported(
                    "Radiance HDR requires FlatHdr (HDR float32). \
                     Use PNG/JPEG for LDR content."
                        .into(),
                ));
            }
            DecodedImage::Layered(_) => return Err(Error::MissingLayer),
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "Radiance HDR is single-frame; export each frame separately".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "Radiance HDR is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        // RGBE drops alpha — Radiance HDR spec has no alpha channel.
        let rgb_pixels: Vec<image::Rgb<f32>> = buf
            .pixels
            .iter()
            .map(|p| image::Rgb([p.r(), p.g(), p.b()]))
            .collect();
        let mut out: Vec<u8> = Vec::new();
        let encoder = image::codecs::hdr::HdrEncoder::new(&mut out);
        encoder
            .encode(&rgb_pixels, buf.width as usize, buf.height as usize)
            .map_err(|e| Error::Encode(format!("HDR encode: {e}")))?;
        Ok(out)
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
    fn import_rejects_wrong_magic_with_decode() {
        let err = HdrRadianceImporter
            .import(b"\xDE\xAD\xBE\xEF\x00\x01\x02", &ImportOpts::default())
            .expect_err("wrong magic");
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
    }

    #[test]
    fn exporter_recognizes_hdr_format() {
        assert!(HdrRadianceExporter.supports_format(ExportFormat::HdrRadiance));
        assert!(!HdrRadianceExporter.supports_format(ExportFormat::Png));
    }

    /// Audit-13 wave-2 (2026-05-27): real round-trip test now that
    /// HDR decode+encode are wired. Build 2×2 fixture with non-
    /// trivial HDR values (>1.0 to confirm float32 preservation).
    #[test]
    fn roundtrip_2x2_preserves_hdr_within_rgbe_quantization() {
        let original = ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![
                LinearRgba::new(0.5, 1.0, 2.0, 1.0),
                LinearRgba::new(1.5, 0.25, 0.75, 1.0),
                LinearRgba::new(0.1, 0.2, 0.3, 1.0),
                LinearRgba::new(4.0, 2.0, 1.0, 1.0),
            ],
            color_profile: ColorProfile::LinearRec709,
        };
        let bytes = HdrRadianceExporter
            .export(
                &DecodedImage::FlatHdr(original.clone()),
                &ExportOpts {
                    format: ExportFormat::HdrRadiance,
                    ..ExportOpts::default()
                },
            )
            .expect("HDR encode");
        // Magic should be at start of file.
        assert!(bytes.starts_with(b"#?RADIANCE"), "magic must be present");

        let decoded = HdrRadianceImporter
            .import(&bytes, &ImportOpts::default())
            .expect("HDR re-decode");
        let DecodedImage::FlatHdr(reconst) = decoded else {
            panic!("expected FlatHdr after re-decode");
        };
        assert_eq!(reconst.width, 2);
        assert_eq!(reconst.height, 2);
        // RGBE quantizes to 8-bit shared exponent → ~1% precision
        // loss is expected. Assert each channel within 5% tolerance
        // (covers both quantization and any encoder/decoder rounding).
        for (i, (orig, got)) in original
            .pixels
            .iter()
            .zip(reconst.pixels.iter())
            .enumerate()
        {
            let tolerance = 0.05;
            for (label, (o, g)) in [
                ("r", (orig.r(), got.r())),
                ("g", (orig.g(), got.g())),
                ("b", (orig.b(), got.b())),
            ] {
                let abs_diff = (o - g).abs();
                let rel = abs_diff / o.max(0.001);
                assert!(
                    rel < tolerance,
                    "pixel {i} {label}: orig={o}, got={g}, rel_err={rel} (tol {tolerance})"
                );
            }
            // Alpha is synthesized to 1.0 on decode (RGBE has no alpha).
            assert_eq!(got.a(), 1.0, "pixel {i}: alpha should be 1.0");
        }
    }
}

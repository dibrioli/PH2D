#![forbid(unsafe_code)]
//! `ph2d-imageio-webp` — WebP decode + lossless encode (W1.T3).
//!
//! Pure-Rust via `image` 0.25's bundled `image-webp` backend.
//!
//! ### What W1.T3 covers
//!
//! - Magic-byte recognition by the canonical 12-byte
//!   `RIFF [size] WEBP` header.
//! - Decode both lossy and lossless WebP (the `image-webp` decoder
//!   handles both). Alpha preserved.
//! - **Lossless encode only.** `image` 0.25's pure-Rust WebP encoder
//!   is lossless-only; lossy encode requires a libwebp C binding which
//!   would violate HR-1. Documented at the boundary so callers know
//!   `opts.quality` is ignored for WebP today.
//! - Same dimension cap (32K) + Truncated + Decode guards as PNG/JPEG.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (W2+)
//!
//! - Lossy WebP encode (waits for pure-Rust upstream or HR-1 lift).
//! - Animated WebP (would return `DecodedImage::Animated` instead of
//!   `Flat`; lands when GIF/APNG land — same shape).
//! - ICC profile chunk parsing (W2.0.1).
//! - EXIF/XMP metadata preservation.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};
// Bomb-defence cap hoisted to `ph2d_imageio::MAX_RASTER_DIMENSION`
// per audit B-H2.

/// Recognise the 12-byte WebP container header: `RIFF` + 4-byte size
/// (any value) + `WEBP`.
fn is_webp_magic(b: &[u8]) -> bool {
    b.len() >= 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP"
}

/// Register the WebP importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(WebpImporter));
}

/// Register the WebP exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(WebpExporter));
}

/// WebP import driver.
pub struct WebpImporter;

impl ImageImporter for WebpImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_webp_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("webp") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        let img =
            image::load_from_memory_with_format(src, image::ImageFormat::WebP).map_err(|e| {
                // Audit-8 Lens L L-3 (2026-05-26): migrated to helper.
                Error::from_decoder_message(e.to_string())
            })?;
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "WebP has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        let rgba = img.to_rgba8();
        let pixels: Vec<SrgbRgba> = rgba
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// WebP export driver. **Lossless only** in W1.T3 (see crate doc).
pub struct WebpExporter;

impl ImageExporter for WebpExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Webp)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => return Err(Error::MissingLayer),
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "animated WebP not supported in W1.T3 — single-frame only".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "WebP is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        // Audit D-E4: checked_mul to refuse silently-truncated buffer
        // on 32-bit `usize` overflow.
        let byte_len = buf.pixels.len().checked_mul(4).ok_or(Error::OutOfMemory)?;
        let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
        for p in &buf.pixels {
            rgba8.extend_from_slice(&p.0);
        }
        let img_buf =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(buf.width, buf.height, rgba8)
                .ok_or_else(|| {
                    Error::Encode(format!(
                        "pixel count {} does not match {}×{}",
                        buf.pixels.len(),
                        buf.width,
                        buf.height
                    ))
                })?;
        let mut out: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut out);
        // Lossless WebP via image-webp backend. `opts.quality` is
        // documented as ignored for this format until lossy encode
        // lands (W2+).
        img_buf
            .write_to(&mut cursor, image::ImageFormat::WebP)
            .map_err(|e| Error::Encode(e.to_string()))?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_2x2() -> ImageBuffer<SrgbRgba> {
        ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![
                SrgbRgba([255, 0, 0, 255]),
                SrgbRgba([0, 255, 0, 200]),
                SrgbRgba([0, 0, 255, 100]),
                SrgbRgba([128, 128, 128, 50]),
            ],
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_webp_magic_strong() {
        let imp = WebpImporter;
        // Minimal RIFF/WEBP header.
        let mut hdr = b"RIFF".to_vec();
        hdr.extend_from_slice(&[0; 4]); // size placeholder
        hdr.extend_from_slice(b"WEBP");
        assert_eq!(imp.supports(MagicHint::Bytes(&hdr)), MagicMatch::Strong);

        // RIFF but not WEBP (could be WAV / AVI) — must NOT match.
        let mut riff_wav = b"RIFF".to_vec();
        riff_wav.extend_from_slice(&[0; 4]);
        riff_wav.extend_from_slice(b"WAVE");
        assert_eq!(imp.supports(MagicHint::Bytes(&riff_wav)), MagicMatch::None);

        // Too short — must NOT match.
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"RIFF\0\0\0\0WEB")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_webp_extension_case_insensitive() {
        let imp = WebpImporter;
        for ext in &["webp", "WEBP", "WebP"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak,);
        }
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_webp_format() {
        let exp = WebpExporter;
        assert!(exp.supports_format(ExportFormat::Webp));
        assert!(!exp.supports_format(ExportFormat::Png));
        assert!(!exp.supports_format(ExportFormat::Jpeg));
    }

    /// Lossless WebP round-trip MUST preserve pixels bit-exact
    /// including alpha. `image-webp` 0.2 lossless path supports alpha.
    #[test]
    fn roundtrip_lossless_preserves_pixels_bit_exact() {
        let original = fixture_2x2();
        let bytes = WebpExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Webp,
                    ..ExportOpts::default()
                },
            )
            .expect("WebP export should succeed");
        assert!(is_webp_magic(&bytes), "output bytes are not WebP");

        let decoded = WebpImporter
            .import(&bytes, &ImportOpts::default())
            .expect("WebP import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("expected Flat");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.pixels.len(), 4);
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(
                got.0, want.0,
                "pixel {i} drifted across lossless WebP round-trip"
            );
        }
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = WebpImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty must fail");
        assert!(matches!(err, Error::Truncated));
        assert_eq!(err.fluent_key(), "imageio.error.truncated");
    }

    /// HR-5 byte-exact determinism (audit D-E2 / agent E-H3): WebP
    /// lossless encode must be byte-exact for the same input. If this
    /// fails post-upgrade, `image-webp` 0.2 introduced a
    /// non-deterministic state (threading, hash salt) — document an
    /// exception explicitly rather than letting drift sneak in.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = fixture_2x2();
        let opts = ExportOpts {
            format: ExportFormat::Webp,
            ..ExportOpts::default()
        };
        let bytes_a = WebpExporter
            .export(&DecodedImage::Flat(original.clone()), &opts)
            .expect("first WebP export");
        let bytes_b = WebpExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("second WebP export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: WebP lossless export must be byte-exact deterministic."
        );
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = WebpExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Webp,
                    ..ExportOpts::default()
                },
            )
            .expect_err("WebP must refuse HDR without tone-map");
        assert!(matches!(err, Error::HdrUnsupported));
    }
}

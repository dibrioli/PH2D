#![forbid(unsafe_code)]
//! `ph2d-imageio-png` — PNG decode + encode (W0.T5 stub; W1.T1 full).
//!
//! First `crates/ph2d-imageio-*` format crate to land. Proves the W0
//! vertical end-to-end: `&[u8]` PNG bytes → [`DecodedImage::Flat`]
//! round-trip via [`image`] 0.25's pure-Rust PNG codec.
//!
//! ### What this stub covers (W0.T5)
//!
//! - Magic-byte + extension `supports()` recognition.
//! - 8-bit RGBA decode + encode (the workhorse `ImageBuffer<SrgbRgba>`
//!   wire shape).
//! - sRGB color profile assumed (no ICC parsing) — per ADR-0054 §2.3
//!   W1 policy.
//! - Lossless bit-exact round-trip test against a 2×2 fixture.
//!
//! ### What W1.T1 will add (out of W0 scope)
//!
//! - 16-bit decode + encode (`PngColorType::Rgba16`).
//! - sBIT chunk preservation.
//! - ICC profile chunk parsing (W2.0.1 ICC pipeline activation).
//! - Quality-vs-speed encoder tuning options via `ExportOpts`.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportOpts, ExporterRegistry, ImageBuffer, ImageExporter,
    ImageImporter, ImportOpts, ImporterRegistry, MagicHint,
};

/// Register the PNG importer with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_importers`] by
/// `cargo run -p ph2d-imageio-sync`.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(PngImporter));
}

/// Register the PNG exporter with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_exporters`].
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(PngExporter));
}

/// PNG import driver. Stateless — one instance shared across the
/// importer registry.
pub struct PngImporter;

/// PNG's 8-byte signature: `\x89 P N G \r \n \x1a \n`.
const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

impl ImageImporter for PngImporter {
    fn supports(&self, hint: MagicHint<'_>) -> bool {
        match hint {
            MagicHint::Bytes(b) => b.starts_with(&PNG_MAGIC),
            MagicHint::Extension(ext) => ext.eq_ignore_ascii_case("png"),
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        // `with_format` skips magic-byte autodetect inside `image` — we
        // already vetted the format via `supports()`, so this saves
        // ~µs of redundant inspection and avoids the rare case where
        // `image` would dispatch to a decoder we didn't enable.
        let img = image::load_from_memory_with_format(src, image::ImageFormat::Png)
            .map_err(|e| Error::Decode(e.to_string()))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let pixels: Vec<SrgbRgba> = rgba
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            // W0.T5 assumes sRGB (ADR-0054 §2.3 W1 policy). W2.0.1
            // expands to honour iCCP chunk when present.
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// PNG export driver. Stateless.
pub struct PngExporter;

impl ImageExporter for PngExporter {
    fn supports_format(&self, fmt: &str) -> bool {
        fmt.eq_ignore_ascii_case("png")
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => {
                // PNG is single-layer; the caller should flatten via the
                // engine's composite pipeline before reaching this
                // exporter. Returning `MissingLayer` is the "you asked
                // PNG to do what only PSD/ORA can" signal.
                return Err(Error::MissingLayer);
            }
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "PNG is single-frame; export to APNG instead".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "PNG is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        let mut rgba8: Vec<u8> = Vec::with_capacity(buf.pixels.len() * 4);
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
        img_buf
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| Error::Encode(e.to_string()))?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_png_magic_bytes() {
        let imp = PngImporter;
        assert!(imp.supports(MagicHint::Bytes(&PNG_MAGIC)));
        // Extra bytes after magic still match.
        let mut padded = PNG_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert!(imp.supports(MagicHint::Bytes(&padded)));
        // Wrong magic doesn't match.
        assert!(!imp.supports(MagicHint::Bytes(&[0xff, 0xd8, 0xff, 0xe0])));
        // Empty doesn't match.
        assert!(!imp.supports(MagicHint::Bytes(&[])));
    }

    #[test]
    fn supports_recognizes_png_extension_case_insensitive() {
        let imp = PngImporter;
        assert!(imp.supports(MagicHint::Extension("png")));
        assert!(imp.supports(MagicHint::Extension("PNG")));
        assert!(imp.supports(MagicHint::Extension("Png")));
        assert!(!imp.supports(MagicHint::Extension("jpeg")));
        assert!(!imp.supports(MagicHint::Extension("")));
    }

    #[test]
    fn exporter_recognizes_png_format_case_insensitive() {
        let exp = PngExporter;
        assert!(exp.supports_format("png"));
        assert!(exp.supports_format("PNG"));
        assert!(!exp.supports_format("jpeg"));
    }

    /// The core W0.T5 vertical: synthesize a 2×2 fixture in memory,
    /// export it as PNG bytes, re-import, assert pixels are bit-exact.
    /// PNG is lossless, so blake3 of the pixels round-trips clean —
    /// HR-6 derived (asset = hash blake3 of conteúdo, not bytes-on-wire).
    #[test]
    fn roundtrip_2x2_image_preserves_pixels_bit_exact() {
        let original_pixels = vec![
            SrgbRgba([255, 0, 0, 255]),     // red
            SrgbRgba([0, 255, 0, 255]),     // green
            SrgbRgba([0, 0, 255, 255]),     // blue
            SrgbRgba([128, 128, 128, 200]), // grey with semi-alpha
        ];
        let buffer = ImageBuffer::<SrgbRgba> {
            width: 2,
            height: 2,
            pixels: original_pixels.clone(),
            color_profile: ColorProfile::Srgb,
        };
        let bytes = PngExporter
            .export(
                &DecodedImage::Flat(buffer),
                &ExportOpts {
                    format: "png".into(),
                    ..ExportOpts::default()
                },
            )
            .expect("PNG export should succeed");
        // PNG signature check on the produced bytes.
        assert!(bytes.starts_with(&PNG_MAGIC), "output bytes are not PNG");

        let decoded = PngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("PNG import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("PNG decode should yield Flat, got something else");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.color_profile, ColorProfile::Srgb);
        assert_eq!(
            out.pixels.len(),
            original_pixels.len(),
            "pixel count must match",
        );
        for (i, (got, want)) in out.pixels.iter().zip(original_pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "pixel {i} drifted across round-trip");
        }
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        // HR-6 derived: silent clipping of HDR to LDR is a data-loss
        // bug. The exporter refuses unless the caller opts into a
        // ToneMap (none configured in this test).
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = PngExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: "png".into(),
                    ..ExportOpts::default()
                },
            )
            .expect_err("PNG must refuse HDR without tone-map");
        assert!(
            matches!(err, Error::HdrUnsupported),
            "expected HdrUnsupported, got {err:?}"
        );
    }
}

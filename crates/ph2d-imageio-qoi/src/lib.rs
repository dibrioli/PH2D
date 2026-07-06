#![forbid(unsafe_code)]
//! `ph2d-imageio-qoi` — QOI (Quite OK Image, `.qoi`) decode + encode
//! (`line/imageio`).
//!
//! Pure-Rust via `image` 0.25 `qoi` feature. QOI is a lossless RGB/RGBA
//! format with a tiny single-pass codec — very fast decode, making it a
//! good import format *and* a good cook target. Decode converges on
//! `ImageBuffer<SrgbRgba>` via `to_rgba8`; export emits 4-channel QOI
//! (alpha preserved, bit-exact round-trip).
//!
//! ### What this covers
//!
//! - `supports()`: `qoif` 4-byte magic → **Strong** (a genuinely unique
//!   signature, unlike TGA); extension (`.qoi`) → `Weak`.
//! - Decode: QOI RGB → alpha 255; QOI RGBA → alpha preserved.
//! - Encode: 8-bit RGBA → 4-channel QOI via `image::ImageBuffer::write_to`.
//! - Decompression-bomb defence via `ph2d_imageio::MAX_RASTER_DIMENSION`.
//! - HR-1: pure-Rust, no C bindings. HR-5: byte-exact deterministic
//!   encode. HR-6: lossless round-trip (bit-exact pixels + alpha).
//!
//! ### Out of scope
//!
//! - The QOI `colorspace` byte (all-sRGB vs all-linear channel tag) is
//!   informational only; `image` decodes pixels identically either way.
//!   Colour-profile honouring waits on the W2.0.1 ICC pipeline; sRGB is
//!   assumed on decode (ADR-0054 §2.3 W1 policy).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};

/// QOI's 4-byte signature: `q o i f`. Unique enough to be a `Strong`
/// match (`image` 0.25 uses the same bytes for autodetect).
const QOI_MAGIC: &[u8] = b"qoif";

/// Register the QOI importer with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_importers`] by
/// `cargo run -p ph2d-imageio-sync`.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(QoiImporter));
}

/// Register the QOI exporter with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_exporters`].
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(QoiExporter));
}

/// QOI import driver. Stateless — one instance shared across the
/// importer registry.
pub struct QoiImporter;

impl ImageImporter for QoiImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(QOI_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("qoi") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // `with_format` skips redundant magic autodetect — we already
        // vetted the `qoif` signature via `supports()`.
        let img = image::load_from_memory_with_format(src, image::ImageFormat::Qoi)
            .map_err(|e| Error::from_decoder_message(e.to_string()))?;
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "QOI has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // `to_rgba8` widens QOI-RGB (alpha → 255) and passes QOI-RGBA
        // through unchanged into the canonical 8-bit RGBA wire shape.
        let rgba = img.to_rgba8();
        let pixels: Vec<SrgbRgba> = rgba
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            // ADR-0054 §2.3 W1 policy: sRGB assumed. W2.0.1 expands when
            // the ICC pipeline lands.
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// QOI export driver. Stateless.
pub struct QoiExporter;

impl ImageExporter for QoiExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Qoi)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => {
                // QOI is single-layer; flatten via the engine's composite
                // pipeline before reaching this exporter.
                return Err(Error::MissingLayer);
            }
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "QOI is single-frame; export each frame separately".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "QOI is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        // checked_mul prevents 32-bit silent truncation (mirror of PNG
        // audit D-E4).
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
        img_buf
            .write_to(&mut cursor, image::ImageFormat::Qoi)
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
                SrgbRgba([0, 255, 0, 255]),
                SrgbRgba([0, 0, 255, 255]),
                SrgbRgba([128, 128, 128, 200]),
            ],
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_qoi_magic_strong() {
        let imp = QoiImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(QOI_MAGIC)),
            MagicMatch::Strong
        );
        let mut padded = QOI_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Strong);
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0x89, b'P', b'N', b'G'])),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_qoi_extension_weak() {
        let imp = QoiImporter;
        assert_eq!(imp.supports(MagicHint::Extension("qoi")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("QOI")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_qoi_format() {
        let exp = QoiExporter;
        assert!(exp.supports_format(ExportFormat::Qoi));
        assert!(!exp.supports_format(ExportFormat::Png));
        assert!(!exp.supports_format(ExportFormat::Tga));
    }

    /// Core vertical: synthesize a 2×2 fixture, export QOI, re-import,
    /// assert pixels bit-exact. QOI is lossless → clean round-trip
    /// (HR-6 derived).
    #[test]
    fn roundtrip_2x2_image_preserves_pixels_bit_exact() {
        let original = fixture_2x2();
        let bytes = QoiExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Qoi,
                    ..ExportOpts::default()
                },
            )
            .expect("QOI export should succeed");
        assert!(bytes.starts_with(QOI_MAGIC), "output bytes are not QOI");

        let decoded = QoiImporter
            .import(&bytes, &ImportOpts::default())
            .expect("QOI import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("QOI decode should yield Flat");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.color_profile, ColorProfile::Srgb);
        assert_eq!(out.pixels.len(), original.pixels.len());
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "pixel {i} drifted across round-trip");
        }
    }

    /// QOI natively carries alpha — mixed transparency must round-trip
    /// bit-exact, not clamp to 0/255.
    #[test]
    fn roundtrip_preserves_alpha_channel() {
        let original = ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![
                SrgbRgba([10, 20, 30, 0]),      // fully transparent
                SrgbRgba([40, 50, 60, 64]),     // ~25% alpha
                SrgbRgba([70, 80, 90, 191]),    // ~75% alpha
                SrgbRgba([100, 110, 120, 255]), // opaque
            ],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = QoiExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Qoi,
                    ..ExportOpts::default()
                },
            )
            .expect("QOI export with alpha");
        let DecodedImage::Flat(out) = QoiImporter
            .import(&bytes, &ImportOpts::default())
            .expect("QOI import with alpha")
        else {
            panic!("expected Flat");
        };
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "alpha pixel {i} drifted");
        }
    }

    /// Minimal canonical decode: a 1×1 QOI round-trips to the exact pixel.
    #[test]
    fn decode_minimal_1x1() {
        let one = ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![SrgbRgba([12, 34, 56, 78])],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = QoiExporter
            .export(
                &DecodedImage::Flat(one),
                &ExportOpts {
                    format: ExportFormat::Qoi,
                    ..ExportOpts::default()
                },
            )
            .expect("encode 1×1 QOI");
        assert!(bytes.starts_with(QOI_MAGIC));
        let DecodedImage::Flat(out) = QoiImporter
            .import(&bytes, &ImportOpts::default())
            .expect("decode 1×1 QOI")
        else {
            panic!("expected Flat");
        };
        assert_eq!(out.width, 1);
        assert_eq!(out.height, 1);
        assert_eq!(out.pixels.len(), 1);
        assert_eq!(out.pixels[0].0, [12, 34, 56, 78]);
    }

    /// HR-5 determinism: exporting the same buffer twice is byte-exact.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let buf = fixture_2x2();
        let opts = ExportOpts {
            format: ExportFormat::Qoi,
            ..ExportOpts::default()
        };
        let bytes_a = QoiExporter
            .export(&DecodedImage::Flat(buf.clone()), &opts)
            .expect("first export");
        let bytes_b = QoiExporter
            .export(&DecodedImage::Flat(buf), &opts)
            .expect("second export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: QOI export must be byte-exact for the same input"
        );
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = QoiImporter
            .import(&[], &ImportOpts::default())
            .expect_err("must refuse empty");
        assert!(matches!(err, Error::Truncated));
        assert_eq!(err.fluent_key(), "imageio.error.truncated");
    }

    #[test]
    fn import_rejects_corrupted_bytes_as_decode_or_truncated() {
        // Magic present (so supports() passes) but header truncated —
        // decoder fails.
        let err = QoiImporter
            .import(b"qoif\x00\x00", &ImportOpts::default())
            .expect_err("corrupted bytes must fail");
        assert!(
            matches!(err, Error::Decode(_) | Error::Truncated),
            "expected Decode or Truncated, got {err:?}"
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
        let err = QoiExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Qoi,
                    ..ExportOpts::default()
                },
            )
            .expect_err("QOI must refuse HDR without tone-map");
        assert!(matches!(err, Error::HdrUnsupported), "got {err:?}");
        assert_eq!(err.fluent_key(), "imageio.error.hdr-unsupported");
    }
}

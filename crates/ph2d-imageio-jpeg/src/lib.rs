#![forbid(unsafe_code)]
//! `ph2d-imageio-jpeg` — JPEG decode + encode (W1.T2).
//!
//! Pure-Rust via `image` 0.25's bundled `jpeg-decoder` + `jpeg-encoder`.
//! Magic-byte recognition by JPEG's universal SOI prefix `FF D8 FF`
//! (covers JFIF / Exif / SPIFF / raw flavours). Alpha is **dropped on
//! export** because JPEG has no alpha channel — `image` composites
//! against a black background automatically.
//!
//! ### What W1.T2 covers
//!
//! - Magic-byte (Strong) + extension (Weak) `supports()`. Accepts both
//!   `"jpeg"` and `"jpg"` extensions.
//! - Decode via `image::load_from_memory_with_format`; `to_rgba8`
//!   widening synthesises alpha=255 for the RGB source.
//! - Encode honours `ExportOpts::quality` (0-100, clamped to a
//!   reasonable 1-100 range — `image::codecs::jpeg::JpegEncoder` panics
//!   on 0).
//! - Decompression-bomb defence via the same `MAX_DIMENSION` cap as PNG.
//! - HR-15: errors carry [`ph2d_imageio::Error::fluent_key`].
//!
//! ### Out of scope (W2)
//!
//! - EXIF orientation 1-8 honouring on import. Needs `kamadak-exif` or
//!   `image` 0.26's reader API; W1.T2 returns pixels as-stored. UI can
//!   show a warning toast via `imageio.warn.exif-orientation-ignored`
//!   once the chain is wired.
//! - EXIF metadata preservation on re-export. Drops by default.
//! - JFIF density / DPI metadata.
//! - Progressive JPEG encode tuning.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};
// Bomb-defence cap hoisted to `ph2d_imageio::MAX_RASTER_DIMENSION`
// per audit B-H2 (was 4× duplicated across raster crates).

/// JPEG's universal Start-of-Image marker prefix. Variations follow
/// in the next 2 bytes (`FF E0`=JFIF, `FF E1`=Exif, `FF DB`=raw with
/// quant table first). Recognising the first 3 bytes is the canonical
/// cheap check.
const JPEG_MAGIC_PREFIX: [u8; 3] = [0xFF, 0xD8, 0xFF];

/// Register the JPEG importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(JpegImporter));
}

/// Register the JPEG exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(JpegExporter));
}

/// JPEG import driver. Stateless.
pub struct JpegImporter;

impl ImageImporter for JpegImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&JPEG_MAGIC_PREFIX) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) => {
                let lower = ext.to_ascii_lowercase();
                if lower == "jpeg" || lower == "jpg" {
                    MagicMatch::Weak
                } else {
                    MagicMatch::None
                }
            }
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        let img =
            image::load_from_memory_with_format(src, image::ImageFormat::Jpeg).map_err(|e| {
                // Audit-8 Lens L L-3 (2026-05-26): migrated to
                // contract-level `Error::from_decoder_message` helper.
                Error::from_decoder_message(e.to_string())
            })?;
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "JPEG has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // JPEG sources have no alpha; `to_rgba8` synthesises 255 for
        // the missing channel uniformly.
        let rgba = img.to_rgba8();
        let pixels: Vec<SrgbRgba> = rgba
            .chunks_exact(4)
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect();
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            // W1: sRGB assumed (ADR-0054 §2.3); W2.0.1 reads embedded
            // Adobe ICC marker (APP2) when ICC pipeline lands.
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// JPEG export driver. Stateless.
pub struct JpegExporter;

impl ImageExporter for JpegExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Jpeg)
    }

    fn export(&self, img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => {
                return Err(Error::MissingLayer);
            }
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "JPEG is single-frame; encode each frame separately".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "JPEG is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };
        // JPEG has no alpha; the encoder composites against black
        // implicitly. Document this in tests so a re-import + alpha
        // mismatch isn't a surprise.
        // Audit D-E4: `pixels.len() * 4` overflows on 32-bit targets
        // for `width*height ≥ 1G pixels`. Off-hot path so the check is
        // cheap; refuse instead of allocating a truncated buffer.
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
        // Clamp quality to 1..=100 — JpegEncoder panics at 0 and
        // saturates at 100. Default `ExportOpts::quality = 90` is the
        // sane shipping value.
        let quality = opts.quality.clamp(1, 100);
        let mut out: Vec<u8> = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
        encoder
            .encode_image(&img_buf)
            .map_err(|e| Error::Encode(e.to_string()))?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_4x4() -> ImageBuffer<SrgbRgba> {
        // 4×4 with a few distinct values so JPEG quantisation has
        // something to chew but the gross structure is recognisable
        // post-roundtrip.
        let mut pixels = Vec::with_capacity(16);
        for y in 0..4 {
            for x in 0..4 {
                pixels.push(SrgbRgba([
                    (x * 64) as u8,
                    (y * 64) as u8,
                    128,
                    255, // alpha will be dropped on export
                ]));
            }
        }
        ImageBuffer {
            width: 4,
            height: 4,
            pixels,
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_jpeg_magic_strong() {
        let imp = JpegImporter;
        // JFIF
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0])),
            MagicMatch::Strong
        );
        // Exif
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0xFF, 0xD8, 0xFF, 0xE1, 0, 0])),
            MagicMatch::Strong
        );
        // Raw quant-table-first
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0xFF, 0xD8, 0xFF, 0xDB, 0, 0])),
            MagicMatch::Strong
        );
        // PNG magic — must NOT match
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0x89, b'P', b'N', b'G'])),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_both_jpeg_and_jpg_extensions_case_insensitive() {
        let imp = JpegImporter;
        for ext in &["jpeg", "jpg", "JPEG", "JPG", "Jpeg", "Jpg"] {
            assert_eq!(
                imp.supports(MagicHint::Extension(ext)),
                MagicMatch::Weak,
                "extension {ext} should match Weak",
            );
        }
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_jpeg_format() {
        let exp = JpegExporter;
        assert!(exp.supports_format(ExportFormat::Jpeg));
        assert!(!exp.supports_format(ExportFormat::Png));
        assert!(!exp.supports_format(ExportFormat::Webp));
    }

    /// JPEG is lossy by definition — we don't assert bit-exact pixels
    /// after round-trip. Instead we check the structure: dimensions
    /// preserved, magic header on output, alpha becomes 255 (JPEG
    /// composites against black + has no alpha channel).
    #[test]
    fn roundtrip_4x4_preserves_dimensions_and_drops_alpha() {
        let original = fixture_4x4();
        let bytes = JpegExporter
            .export(
                &DecodedImage::Flat(original),
                &ExportOpts {
                    format: ExportFormat::Jpeg,
                    quality: 95,
                    ..ExportOpts::default()
                },
            )
            .expect("JPEG export should succeed");
        assert!(
            bytes.starts_with(&JPEG_MAGIC_PREFIX),
            "output bytes are not JPEG (no SOI marker)"
        );

        let decoded = JpegImporter
            .import(&bytes, &ImportOpts::default())
            .expect("JPEG import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("expected Flat");
        };
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 4);
        assert_eq!(out.pixels.len(), 16);
        // All alphas come back as 255 — JPEG dropped the channel.
        for (i, p) in out.pixels.iter().enumerate() {
            assert_eq!(p.0[3], 255, "pixel {i} alpha should be 255 post-JPEG");
        }
    }

    /// Quality knob actually changes encoded size (quality 10 should
    /// produce smaller bytes than quality 95 on the same input).
    #[test]
    fn quality_knob_affects_encoded_size() {
        let original = fixture_4x4();
        let high = JpegExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Jpeg,
                    quality: 95,
                    ..ExportOpts::default()
                },
            )
            .expect("high quality export");
        let low = JpegExporter
            .export(
                &DecodedImage::Flat(original),
                &ExportOpts {
                    format: ExportFormat::Jpeg,
                    quality: 10,
                    ..ExportOpts::default()
                },
            )
            .expect("low quality export");
        // Sanity: not identical bytes. 4×4 is too small for guaranteed
        // monotonic size delta, but the byte content MUST differ.
        assert_ne!(
            high, low,
            "high vs low quality should produce different bytes"
        );
    }

    /// Quality 0 must NOT panic — exporter clamps to 1.
    #[test]
    fn quality_zero_does_not_panic() {
        let original = fixture_4x4();
        let _ = JpegExporter
            .export(
                &DecodedImage::Flat(original),
                &ExportOpts {
                    format: ExportFormat::Jpeg,
                    quality: 0,
                    ..ExportOpts::default()
                },
            )
            .expect("quality 0 should clamp, not panic");
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = JpegImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty must fail");
        assert!(matches!(err, Error::Truncated));
        assert_eq!(err.fluent_key(), "imageio.error.truncated");
    }

    /// HR-5 byte-exact determinism (audit D-E2 / agent E-H2): exporting
    /// the same input twice must produce identical bytes. The `image`
    /// 0.25 JpegEncoder MUST NOT embed timestamp/comment chunks; this
    /// test guards future regression if upstream adds them.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = fixture_4x4();
        let opts = ExportOpts {
            format: ExportFormat::Jpeg,
            quality: 90,
            ..ExportOpts::default()
        };
        let bytes_a = JpegExporter
            .export(&DecodedImage::Flat(original.clone()), &opts)
            .expect("first JPEG export");
        let bytes_b = JpegExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("second JPEG export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: JPEG export must be byte-exact for the same input + opts. \
             If this fails, image 0.25 started emitting a non-deterministic \
             chunk (timestamp/comment); pin via direct `jpeg-encoder` API."
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
        let err = JpegExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Jpeg,
                    ..ExportOpts::default()
                },
            )
            .expect_err("JPEG must refuse HDR without tone-map");
        assert!(matches!(err, Error::HdrUnsupported));
    }
}

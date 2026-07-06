#![forbid(unsafe_code)]
//! `ph2d-imageio-tga` — Truevision TGA (`.tga`) decode + encode
//! (`line/imageio`).
//!
//! Pure-Rust via `image` 0.25 `tga` feature. TGA (Targa) is a classic
//! game-art interchange format: uncompressed or RLE, 8/16/24/32-bit,
//! optional alpha. Every colour type converges on `ImageBuffer<SrgbRgba>`
//! via `to_rgba8` on import; export emits 32-bit RGBA (alpha preserved).
//!
//! ### What this covers
//!
//! - `supports()` recognition: extension (`.tga` / `.tpic`) + optional
//!   v2 file-footer signature. **Never `Strong`** — TGA has no header
//!   magic (its first bytes are colour-map / image-type fields that
//!   overlap valid data), and the v2 footer is optional (v1 files and
//!   `image` 0.25's own encoder omit it). Claiming `Strong` would let
//!   TGA hijack dispatch for unrelated files.
//! - Decode: any TGA colour type → 8-bit RGBA (grayscale → R=G=B,
//!   colour-mapped → expanded, alpha preserved or 255 when absent).
//! - Encode: 8-bit RGBA → 32-bit TGA via `image::ImageBuffer::write_to`.
//! - Decompression-bomb defence via `ph2d_imageio::MAX_RASTER_DIMENSION`.
//! - HR-1: pure-Rust, no C bindings. HR-5: byte-exact deterministic
//!   encode. HR-6: lossless round-trip (bit-exact pixels).
//!
//! ### Out of scope
//!
//! - 16-bit-per-channel TGA is quantised to 8-bit by `image` at import
//!   (documented loss; TGA's 16-bit mode is actually 5-5-5-1 packed, not
//!   16-bit channels — no precision-preservation client today).
//! - Colour-profile honouring waits on the W2.0.1 ICC pipeline; sRGB is
//!   assumed on decode (ADR-0054 §2.3 W1 policy).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};

/// Truevision v2 file-footer signature — the last 18 bytes of a TGA 2.0
/// file: `TRUEVISION-XFILE.` + `\0`. Present only when a writer opts into
/// the v2 footer (many, including `image` 0.25, don't), so a match here
/// is `Weak`, never `Strong`.
const TGA_FOOTER_SIG: &[u8] = b"TRUEVISION-XFILE.\0";

/// Register the TGA importer with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_importers`] by
/// `cargo run -p ph2d-imageio-sync`.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(TgaImporter));
}

/// Register the TGA exporter with `reg`. Codegen-wired into
/// [`ph2d_imageio_registry_init::register_all_exporters`].
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(TgaExporter));
}

/// TGA import driver. Stateless — one instance shared across the
/// importer registry.
pub struct TgaImporter;

impl ImageImporter for TgaImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            // TGA has no header magic. If the caller handed us the whole
            // file and it carries the optional v2 footer, that is a Weak
            // signal — never Strong (the footer is optional; a Strong
            // claim would hijack dispatch for any file ending in it).
            MagicHint::Bytes(b) if b.ends_with(TGA_FOOTER_SIG) => MagicMatch::Weak,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext)
                if ext.eq_ignore_ascii_case("tga") || ext.eq_ignore_ascii_case("tpic") =>
            {
                MagicMatch::Weak
            }
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // `with_format` skips `image`'s magic autodetect — TGA has none,
        // so autodetect would fail outright; we already vetted via the
        // shell's `supports()` dispatch.
        let img = image::load_from_memory_with_format(src, image::ImageFormat::Tga)
            .map_err(|e| Error::from_decoder_message(e.to_string()))?;
        let (width, height) = (img.width(), img.height());
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "TGA has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // `to_rgba8` widens every TGA colour type (grayscale, colour-
        // mapped, 24-bit RGB, 32-bit RGBA) into the same 8-bit RGBA wire
        // shape. Alpha is synthesised to 255 when the source has none.
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

/// TGA export driver. Stateless.
pub struct TgaExporter;

impl ImageExporter for TgaExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Tga)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let buf = match img {
            DecodedImage::Flat(b) => b,
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => {
                // TGA is single-layer; flatten via the engine's composite
                // pipeline before reaching this exporter.
                return Err(Error::MissingLayer);
            }
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "TGA is single-frame; export each frame separately".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "TGA is raster; rasterize the VectorDoc first".into(),
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
            .write_to(&mut cursor, image::ImageFormat::Tga)
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
    fn supports_recognizes_tga_extension_weak() {
        let imp = TgaImporter;
        // TGA identification is extension-driven → Weak (a Bytes match on
        // another format elsewhere wins dispatch).
        assert_eq!(imp.supports(MagicHint::Extension("tga")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("TGA")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("tpic")), MagicMatch::Weak);
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("")), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_v2_footer_weak_never_strong() {
        let imp = TgaImporter;
        // A buffer ending with the v2 footer → Weak. Never Strong: TGA
        // has no reliable magic, and the footer is optional.
        let mut buf = vec![0u8; 40];
        buf.extend_from_slice(TGA_FOOTER_SIG);
        assert_eq!(imp.supports(MagicHint::Bytes(&buf)), MagicMatch::Weak);
    }

    #[test]
    fn supports_rejects_headerless_bytes_as_none() {
        let imp = TgaImporter;
        // Random 32-byte header with no footer → None. TGA must NOT
        // weak-match arbitrary bytes or it hijacks dispatch.
        assert_eq!(
            imp.supports(MagicHint::Bytes(&[0x00; 32])),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_tga_format() {
        let exp = TgaExporter;
        assert!(exp.supports_format(ExportFormat::Tga));
        assert!(!exp.supports_format(ExportFormat::Png));
        assert!(!exp.supports_format(ExportFormat::Qoi));
    }

    /// Core vertical: synthesize a 2×2 fixture, export TGA, re-import,
    /// assert pixels bit-exact. TGA (uncompressed/RLE) is lossless, so
    /// the round-trip is clean — HR-6 derived.
    #[test]
    fn roundtrip_2x2_image_preserves_pixels_bit_exact() {
        let original = fixture_2x2();
        let bytes = TgaExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Tga,
                    ..ExportOpts::default()
                },
            )
            .expect("TGA export should succeed");

        let decoded = TgaImporter
            .import(&bytes, &ImportOpts::default())
            .expect("TGA import should succeed");
        let DecodedImage::Flat(out) = decoded else {
            panic!("TGA decode should yield Flat");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.color_profile, ColorProfile::Srgb);
        assert_eq!(out.pixels.len(), original.pixels.len());
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "pixel {i} drifted across round-trip");
        }
    }

    /// TGA 32-bit carries a real alpha channel — a fixture mixing fully
    /// transparent, semi-transparent, and opaque pixels must survive the
    /// round-trip bit-exact (not clamped to 0/255).
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
        let bytes = TgaExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Tga,
                    ..ExportOpts::default()
                },
            )
            .expect("TGA export with alpha");
        let DecodedImage::Flat(out) = TgaImporter
            .import(&bytes, &ImportOpts::default())
            .expect("TGA import with alpha")
        else {
            panic!("expected Flat");
        };
        for (i, (got, want)) in out.pixels.iter().zip(original.pixels.iter()).enumerate() {
            assert_eq!(got.0, want.0, "alpha pixel {i} drifted");
        }
    }

    /// Minimal canonical decode: a 1×1 TGA round-trips to the exact pixel.
    #[test]
    fn decode_minimal_1x1() {
        let one = ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![SrgbRgba([12, 34, 56, 78])],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = TgaExporter
            .export(
                &DecodedImage::Flat(one),
                &ExportOpts {
                    format: ExportFormat::Tga,
                    ..ExportOpts::default()
                },
            )
            .expect("encode 1×1 TGA");
        let DecodedImage::Flat(out) = TgaImporter
            .import(&bytes, &ImportOpts::default())
            .expect("decode 1×1 TGA")
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
            format: ExportFormat::Tga,
            ..ExportOpts::default()
        };
        let bytes_a = TgaExporter
            .export(&DecodedImage::Flat(buf.clone()), &opts)
            .expect("first export");
        let bytes_b = TgaExporter
            .export(&DecodedImage::Flat(buf), &opts)
            .expect("second export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: TGA export must be byte-exact for the same input"
        );
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = TgaImporter
            .import(&[], &ImportOpts::default())
            .expect_err("must refuse empty");
        assert!(matches!(err, Error::Truncated));
        assert_eq!(err.fluent_key(), "imageio.error.truncated");
    }

    #[test]
    fn import_rejects_garbage_bytes_as_decode_or_truncated() {
        // Too short to be even a TGA header (18 bytes) → decoder error.
        let err = TgaImporter
            .import(&[0, 0, 2, 0, 0], &ImportOpts::default())
            .expect_err("garbage bytes must fail");
        assert!(
            matches!(err, Error::Decode(_) | Error::Truncated),
            "expected Decode or Truncated, got {err:?}"
        );
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        // Silent HDR→LDR clipping is data loss; refuse without a tone-map.
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = TgaExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Tga,
                    ..ExportOpts::default()
                },
            )
            .expect_err("TGA must refuse HDR without tone-map");
        assert!(matches!(err, Error::HdrUnsupported), "got {err:?}");
        assert_eq!(err.fluent_key(), "imageio.error.hdr-unsupported");
    }
}

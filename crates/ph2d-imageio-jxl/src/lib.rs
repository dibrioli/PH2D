#![forbid(unsafe_code)]
//! `ph2d-imageio-jxl` — JPEG XL decode (W3.T1).
//!
//! Pure-Rust via `jxl-oxide` 0.10. JXL is the next-gen lossless +
//! lossy format (HDR + alpha + animation in spec). W3.T1 ships a
//! **strict subset**: LDR Flat path only, first frame only, sRGB
//! output via explicit `request_color_encoding` (wide-gamut JXLs
//! are gamut-mapped to sRGB before quantisation), Gray/GrayA/RGB/
//! RGBA only. **Rejected** with `Error::Unsupported`:
//! - HDR JXL (`hdr_type() == Some(Pq | Hlg)`).
//! - Multi-frame JXL (animation → `num_loaded_keyframes() > 1`).
//! - CMYK / CMYKa (silent-corruption-prevention; needs colour
//!   conversion that isn't wired yet).
//!
//! Encode is permanent-deferred — jxl-oxide is decode-only as of
//! 0.10; native JXL encode lands when a pure-Rust encoder
//! materialises (`zune-jxl`?).
//!
//! ### What W3.T1 covers
//!
//! - Magic: ISOBMFF container `\0\0\0\x0cJXL \r\n\x87\n` OR
//!   codestream `\xff\x0a` raw.
//! - Decode: first frame → `ImageBuffer<SrgbRgba>` (LDR Flat path).
//!   Multi-frame defers to Animated bridge (W3+); HDR JXL defers
//!   to FlatHdr bridge (W3+).
//! - Encode: Error::Unsupported (jxl-oxide is decode-only).
//! - HR-1: pure-Rust, no C bindings.
//!
//! Audit-13 W3.T1.5 wave-2 (2026-05-27): real decode replaces the
//! magic-only-stub from cc97cd4.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
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
        let mut img = jxl_oxide::JxlImage::builder()
            .read(std::io::Cursor::new(src))
            .map_err(|e| Error::from_decoder_message(format!("JXL decode init: {e}")))?;
        let width = img.width();
        let height = img.height();
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "JXL has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // Audit-14 Lens MM CRITICAL (2026-05-27): JXL native encoding
        // may be Display-P3, Rec2020, BT.2100 PQ, etc. — quantising
        // those f32 buffers as if they were sRGB would mislabel
        // wide-gamut content. Request sRGB explicitly so the
        // decoder applies gamut conversion before we touch pixels;
        // `ColorProfile::Srgb` is then truthful.
        img.request_color_encoding(jxl_oxide::EnumColourEncoding::srgb(
            jxl_oxide::RenderingIntent::Relative,
        ));
        // Audit-14 Lens MM HIGH (2026-05-27): reject HDR JXL (PQ/HLG
        // transfer functions) explicitly. Silent clamp to [0,1]
        // would lose the entire luminance range.
        if img.hdr_type().is_some() {
            return Err(Error::Unsupported(format!(
                "JXL is HDR ({:?}); W3.T1 ships LDR Flat path only. \
                 FlatHdr bridge deferred to W3+ (Painter HDR import demo).",
                img.hdr_type()
            )));
        }
        // Audit-14 Lens MM HIGH (2026-05-27): reject multi-frame JXL.
        // `render_frame(0)` would silently drop the rest; Animated
        // bridge is deferred to W3+.
        let num_frames = img.num_loaded_keyframes();
        if num_frames > 1 {
            return Err(Error::Unsupported(format!(
                "JXL has {num_frames} keyframes; W3.T1 ships single-frame only. \
                 Animated bridge deferred to W3+."
            )));
        }
        // Render first frame. Audit-14 Lens MM + KK note: with
        // `request_color_encoding(srgb)` above, jxl-oxide
        // gamut-maps CMYK/Cmyka sources to sRGB before the pixel
        // stream — so `Render::stream().channels()` will be 3/4
        // (Rgb/Rgba) for any input, never 4-but-actually-CMYK.
        // The previous Cmyk-channel collision (channel-count match
        // misinterpreting C/M/Y/K as R/G/B/A) is structurally
        // prevented by the request_color_encoding above; no
        // post-render check needed.
        let render = img
            .render_frame(0)
            .map_err(|e| Error::from_decoder_message(format!("JXL render frame 0: {e}")))?;
        let mut stream = render.stream();
        let channels = stream.channels() as usize;
        let pixel_count = (width as usize) * (height as usize);
        let mut planar = vec![0.0_f32; pixel_count * channels];
        let written = stream.write_to_buffer(&mut planar);
        // Audit-14 Lens MM MEDIUM (2026-05-27): the previous guard
        // `written == 0` accepted partial fills (1..len-1) as
        // success. `ImageStream::write_to_buffer` returns the
        // number of f32 samples actually written; if it's not the
        // full buffer the trailing pixels are garbage. Require
        // exact fill.
        if written != planar.len() {
            return Err(Error::Decode(format!(
                "JXL render stream wrote {written} samples; \
                 expected {} ({pixel_count} pixels × {channels} channels)",
                planar.len()
            )));
        }
        // Quantize f32 [0..1] → u8 for the LDR Flat path. HDR JXL
        // (out-of-range floats) clamp to [0,1] — FlatHdr bridge in
        // W3+ when ImportOpts.color_profile_target gains LinearHdr.
        let pixels: Vec<SrgbRgba> = match channels {
            1 => planar
                .iter()
                .map(|&v| {
                    let c = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    SrgbRgba([c, c, c, 255])
                })
                .collect(),
            2 => planar
                .chunks_exact(2)
                .map(|c| {
                    let v = (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    let a = (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    SrgbRgba([v, v, v, a])
                })
                .collect(),
            3 => planar
                .chunks_exact(3)
                .map(|c| {
                    SrgbRgba([
                        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        255,
                    ])
                })
                .collect(),
            4 => planar
                .chunks_exact(4)
                .map(|c| {
                    SrgbRgba([
                        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                        (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                    ])
                })
                .collect(),
            n => {
                return Err(Error::Unsupported(format!(
                    "JXL {n}-channel image; W3.T1 ships 1/2/3/4 channel only"
                )));
            }
        };
        Ok(DecodedImage::Flat(ImageBuffer {
            width,
            height,
            pixels,
            color_profile: ColorProfile::Srgb,
        }))
    }
}

/// JXL export driver. Permanent-deferred — jxl-oxide is decode-only.
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

    /// Audit-13 wave-2: real decode test would require a JXL fixture
    /// (jxl-oxide is decode-only — we can't synthesize one in-process
    /// without an encoder). Magic-byte-then-truncated path is the
    /// best in-tree coverage; full real-fixture decode is a
    /// W3.T1.6 follow-up when a canonical 1×1 JXL fixture is
    /// generated externally (e.g. via `cjxl` CLI tool) and copied
    /// to `tests/fixtures/`.
    #[test]
    fn import_truncated_jxl_returns_decode_error() {
        let err = JxlImporter
            .import(&JXL_ISOBMFF, &ImportOpts::default())
            .expect_err("truncated container");
        // Either Decode (parse failed mid-header) or Truncated (EOF
        // signature matched the helper). Both are non-panic outcomes.
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
    }

    #[test]
    fn import_truncated_codestream_returns_decode_error() {
        let bytes = [0xFF, 0x0A, 0x00, 0x01, 0x02, 0x03];
        let err = JxlImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("truncated codestream");
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
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

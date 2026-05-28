#![forbid(unsafe_code)]
//! `ph2d-imageio-exr` — OpenEXR decode (W3.T2).
//!
//! Pure-Rust via `exr` 1.x. OpenEXR is the canonical float32 HDR
//! format for VFX / IBL / scene-linear pipelines (Blender / Houdini /
//! Nuke). Single-part RGBA float32 → [`DecodedImage::FlatHdr`].
//!
//! ### What W3.T2 covers
//!
//! - Magic: `\x76\x2f\x31\x01` (little-endian `0x01312f76`).
//! - Decode: first-RGBA-layer via `exr::prelude::read().rgba_channels`
//!   builder. Closure-based pixel walker — we capture
//!   `(width, Vec<LinearRgba>)` as the typed `Pixels` parameter so
//!   the set_pixel callback has row-stride available.
//! - Export: **deferred** to W3+ — the `exr` 1.x
//!   `SpecificChannels::Image` construction wants typed callback
//!   wires that lock down to first real export client. Returns
//!   `IoError::Unsupported` with actionable message.
//! - HR-1 + HR-9: pure-Rust, no C bindings.
//! - HR-13: dimension cap via `ph2d_imageio::MAX_RASTER_DIMENSION`.
//!
//! ### Out of scope (W3+ amendment)
//!
//! - Multi-part EXR (lightmaps, cubemap arrays, deep samples).
//! - Custom channel layouts beyond RGBA (Z-depth, normals, etc).
//! - Tile-based + DWA/DWB compression.
//!
//! Audit-13 W3.T1.5 wave-2 (2026-05-27): real decode replaces the
//! magic-only-stub from cc97cd4.

use ph2d_color::LinearRgba;
// Alias `Error` from the contract crate — inside `import`/`export`
// we use `exr::prelude::*` which shadows `Error`. Audit-13 wave-2.
use ph2d_imageio::Error as IoError;
use ph2d_imageio::{
    ColorProfile, DecodedImage, ExportFormat, ExportOpts, ExporterRegistry, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, MAX_RASTER_DIMENSION, MagicHint,
    MagicMatch,
};

/// EXR magic bytes: `0x76 0x2F 0x31 0x01` (little-endian `0x01312f76`).
const EXR_MAGIC: [u8; 4] = [0x76, 0x2F, 0x31, 0x01];

pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(ExrImporter));
}

pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(ExrExporter));
}

pub struct ExrImporter;

/// Per-decode state passed as `Pixels` to the `exr` builder: row-
/// stride width + flat RGBA buffer. The setter computes
/// `y*width + x` to write into the right slot.
struct ExrPixels {
    width: usize,
    pixels: Vec<LinearRgba>,
}

impl ImageImporter for ExrImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&EXR_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("exr") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, IoError> {
        if src.is_empty() {
            return Err(IoError::Truncated);
        }
        use exr::prelude::*;
        // First-RGBA-layer reader. Closure shapes per `exr` 1.x docs:
        // - create(resolution, _channels) -> Pixels
        // - set_pixel(&mut Pixels, position, (R,G,B,A))
        // We capture width in the `Pixels` struct so the setter has
        // row-stride available (the API doesn't pass width to the
        // setter directly).
        let img = read()
            .no_deep_data()
            .largest_resolution_level()
            .rgba_channels(
                |resolution, _channels| -> ExrPixels {
                    let w = resolution.width();
                    let h = resolution.height();
                    ExrPixels {
                        width: w,
                        pixels: vec![LinearRgba::default(); w * h],
                    }
                },
                |state: &mut ExrPixels,
                 position: Vec2<usize>,
                 (r, g, b, a): (f32, f32, f32, f32)| {
                    let idx = position.y() * state.width + position.x();
                    if idx < state.pixels.len() {
                        state.pixels[idx] = LinearRgba::new(r, g, b, a);
                    }
                },
            )
            .first_valid_layer()
            .all_attributes()
            .from_buffered(std::io::Cursor::new(src))
            .map_err(|e| IoError::from_decoder_message(format!("EXR decode: {e}")))?;

        let state = img.layer_data.channel_data.pixels;
        let width = state.width as u32;
        let height = (state.pixels.len() / state.width.max(1)) as u32;
        if width == 0 || height == 0 {
            return Err(IoError::Decode(format!(
                "EXR has zero-sized dimension: {width}×{height}"
            )));
        }
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(IoError::DimensionExceedsLimit);
        }
        Ok(DecodedImage::FlatHdr(ImageBuffer {
            width,
            height,
            pixels: state.pixels,
            color_profile: ColorProfile::LinearRec709,
        }))
    }
}

/// EXR export driver. Deferred — encode wire-up lands with first
/// real export client.
pub struct ExrExporter;

impl ImageExporter for ExrExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Exr)
    }

    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, IoError> {
        Err(IoError::Unsupported(
            "EXR encode deferred to W3+ — `exr` 1.x `SpecificChannels::Image` \
             construction requires typed callback wires that lock down to \
             first real export client (Painter HDR save demo). Round-trip \
             via `.ph2d-native` preserves FlatHdr losslessly today."
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
        assert!(matches!(err, IoError::Truncated));
    }

    #[test]
    fn import_rejects_wrong_magic_with_decode() {
        let err = ExrImporter
            .import(
                &[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05],
                &ImportOpts::default(),
            )
            .expect_err("wrong magic");
        assert!(matches!(err, IoError::Decode(_) | IoError::Truncated));
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
        assert!(matches!(err, IoError::Unsupported(_)));
    }

    /// Audit-13 wave-2 (2026-05-27): real decode test. We synthesize
    /// a 2×2 EXR via `exr` 1.x encoder (it's the only way to get a
    /// canonical EXR fixture without a binary file), then re-import
    /// and verify pixel values survive within float32 precision.
    #[test]
    fn import_2x2_exr_round_trip_via_exr_encoder() {
        use exr::prelude::*;
        // Build a 2×2 RGBA EXR in memory.
        let width: usize = 2;
        let height: usize = 2;
        let original: [(f32, f32, f32, f32); 4] = [
            (0.5, 1.0, 2.0, 1.0),
            (1.5, 0.25, 0.75, 1.0),
            (0.1, 0.2, 0.3, 0.5),
            (4.0, 2.0, 1.0, 1.0),
        ];
        let mut bytes: Vec<u8> = Vec::new();
        Image::from_channels(
            (width, height),
            SpecificChannels::rgba(|pos: Vec2<usize>| original[pos.y() * width + pos.x()]),
        )
        .write()
        .to_buffered(std::io::Cursor::new(&mut bytes))
        .expect("EXR encode fixture");

        // Decode via our importer.
        let decoded = ExrImporter
            .import(&bytes, &ImportOpts::default())
            .expect("EXR re-decode");
        let DecodedImage::FlatHdr(buf) = decoded else {
            panic!("expected FlatHdr after EXR decode");
        };
        assert_eq!(buf.width, 2);
        assert_eq!(buf.height, 2);
        assert_eq!(buf.pixels.len(), 4);
        // EXR is f32 lossless (no quantization) — assert near-equal
        // (epsilon for half-float pipeline drift if exr promotes
        // through internal conversion).
        for (i, (orig, got)) in original.iter().zip(buf.pixels.iter()).enumerate() {
            let eps = 0.001;
            assert!(
                (orig.0 - got.r()).abs() < eps,
                "px {i} R: {} vs {}",
                orig.0,
                got.r()
            );
            assert!(
                (orig.1 - got.g()).abs() < eps,
                "px {i} G: {} vs {}",
                orig.1,
                got.g()
            );
            assert!(
                (orig.2 - got.b()).abs() < eps,
                "px {i} B: {} vs {}",
                orig.2,
                got.b()
            );
            assert!(
                (orig.3 - got.a()).abs() < eps,
                "px {i} A: {} vs {}",
                orig.3,
                got.a()
            );
        }
    }
}

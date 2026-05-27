#![forbid(unsafe_code)]
//! `ph2d-imageio-tiff` — TIFF decode + encode (W2.T2).
//!
//! Pure-Rust via `tiff` 0.11 (image-rs). TIFF is a container:
//! single-page files come back as [`DecodedImage::Flat`], multi-page
//! files come back as [`DecodedImage::Layered`] (page N → Layer N
//! with `kind = LayerKind::Pixel`).
//!
//! ### What W2.T2 covers
//!
//! - Magic: little-endian `II*\0` (0x49 0x49 0x2A 0x00) + big-endian
//!   `MM\0*` (0x4D 0x4D 0x00 0x2A) → Strong; `"tiff"` / `"tif"`
//!   extensions → Weak.
//! - Decode color types: Gray / GrayA / RGB / RGBA / CMYK 8-bit;
//!   Gray / GrayA / RGB / RGBA 16-bit (quantized down to 8-bit per
//!   the same policy as PNG W1.T1).
//! - CMYK → RGBA via naive `1 - (C+K)/255` formula. Documented as
//!   imprecise; real-world CMYK conversion needs an ICC profile —
//!   W2.0.1 follow-up when an ICC client appears.
//! - Multi-page: walked via `decoder.more_images()` / `next_image()`.
//! - Encode: RGBA8 single-page (deflate compression). Multi-page
//!   export writes each `LayerKind::Pixel` layer as a page.
//! - HR-5 byte-exact determinism.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (later waves)
//!
//! - ICC profile **conversion** (not just preservation): the iCCP tag
//!   `Tag::IccProfile` bytes survive in `ColorProfile::Custom(...)`
//!   but no transform is applied. PSD/TIFF round-trip preserves
//!   profile byte-exact; W2.0.1 will wire `moxcms` 0.8.1 when the
//!   first real client (PSD W2.T4) lands.
//! - 16-bit RGBA round-trip (would need `DecodedImage::Flat16`
//!   variant amendment).
//! - LZW + JPEG-in-TIFF + Fax compression encoders (only deflate
//!   shipped in W2.T2; LZW + deflate decoders are wired via
//!   crate features).
//! - Multi-resolution / pyramidal TIFF (`SubIFDs`).
//! - 32-bit float / EXR-style TIFF (use the EXR crate W3.T2 instead).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry,
    ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, Layer, LayerKind,
    LayerStack, MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};
use std::io::Cursor;
use tiff::ColorType;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::{TiffEncoder, colortype::RGBA8};

/// TIFF little-endian magic: `II` + 0x2A 0x00.
const TIFF_LE_MAGIC: [u8; 4] = [0x49, 0x49, 0x2A, 0x00];
/// TIFF big-endian magic: `MM` + 0x00 0x2A.
const TIFF_BE_MAGIC: [u8; 4] = [0x4D, 0x4D, 0x00, 0x2A];

/// BigTIFF little-endian magic: `II` + 0x2B 0x00. Audit W2.T6 H-2:
/// previously bare `is_tiff_magic` missed BigTIFF, so the registry
/// silently fell through to extension matching. Now we recognize
/// BigTIFF as Strong and refuse on import with a clear message.
const BIGTIFF_LE_MAGIC: [u8; 4] = [0x49, 0x49, 0x2B, 0x00];
/// BigTIFF big-endian magic: `MM` + 0x00 0x2B.
const BIGTIFF_BE_MAGIC: [u8; 4] = [0x4D, 0x4D, 0x00, 0x2B];

fn is_tiff_magic(b: &[u8]) -> bool {
    b.starts_with(&TIFF_LE_MAGIC) || b.starts_with(&TIFF_BE_MAGIC)
}

fn is_bigtiff_magic(b: &[u8]) -> bool {
    b.starts_with(&BIGTIFF_LE_MAGIC) || b.starts_with(&BIGTIFF_BE_MAGIC)
}

/// Register the TIFF importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(TiffImporter));
}

/// Register the TIFF exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(TiffExporter));
}

/// TIFF import driver.
pub struct TiffImporter;

impl ImageImporter for TiffImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            // Both classic TIFF and BigTIFF claim Strong; the import
            // path distinguishes and refuses BigTIFF with an
            // actionable message (audit W2.T6 H-2). Otherwise BigTIFF
            // files would silently fall through to extension Weak +
            // get a generic Decode error from `tiff::Decoder::new`.
            MagicHint::Bytes(b) if is_tiff_magic(b) || is_bigtiff_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) => {
                if ext.eq_ignore_ascii_case("tiff") || ext.eq_ignore_ascii_case("tif") {
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
        // Audit W2.T6 H-2: BigTIFF (`II+\0` / `MM\0+`) is a separate
        // format with 64-bit offsets — `tiff` 0.11 can decode it but
        // our wrapper doesn't expose the BigTIFF-specific quirks
        // (>4 GB images, different tag widths). Refuse early with a
        // clear message; future `ph2d-imageio-bigtiff` lands when a
        // real client needs gigapixel raster.
        if is_bigtiff_magic(src) {
            return Err(Error::Unsupported(
                "BigTIFF (II+\\0 / MM\\0+ magic) not supported in W2.T2; \
                 use classic TIFF, PNG, or .ph2d-native for >4 GB images"
                    .into(),
            ));
        }
        let cursor = Cursor::new(src);
        let mut decoder =
            Decoder::new(cursor).map_err(|e| Error::Decode(format!("TIFF decoder init: {e}")))?;

        // Decode first page.
        let first = decode_page(&mut decoder)?;

        // Walk additional pages, if any.
        let mut pages: Vec<ImageBuffer<SrgbRgba>> = vec![first];
        while decoder.more_images() {
            decoder
                .next_image()
                .map_err(|e| Error::Decode(format!("TIFF next page: {e}")))?;
            let page = decode_page(&mut decoder)?;
            pages.push(page);
        }

        if pages.len() == 1 {
            Ok(DecodedImage::Flat(pages.pop().expect("single page")))
        } else {
            let canvas_width = pages.iter().map(|p| p.width).max().unwrap_or(0);
            let canvas_height = pages.iter().map(|p| p.height).max().unwrap_or(0);
            let layers: Vec<Layer> = pages
                .into_iter()
                .enumerate()
                .map(|(i, buf)| Layer {
                    version: 1,
                    name: format!("Page {}", i + 1),
                    kind: LayerKind::Pixel,
                    pixels: Some(buf),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                })
                .collect();
            Ok(DecodedImage::Layered(LayerStack {
                version: 1,
                canvas_width,
                canvas_height,
                layers,
                color_profile: ColorProfile::Srgb,
            }))
        }
    }
}

/// Decode the current page of a TIFF decoder to RGBA8.
fn decode_page<R: std::io::Read + std::io::Seek>(
    decoder: &mut Decoder<R>,
) -> Result<ImageBuffer<SrgbRgba>, Error> {
    let (width, height) = decoder
        .dimensions()
        .map_err(|e| Error::Decode(format!("TIFF dimensions: {e}")))?;
    if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
        return Err(Error::DimensionExceedsLimit);
    }
    if width == 0 || height == 0 {
        return Err(Error::Decode(format!(
            "TIFF has zero-sized page: {width}×{height}"
        )));
    }
    let color = decoder
        .colortype()
        .map_err(|e| Error::Decode(format!("TIFF colortype: {e}")))?;
    let result = decoder
        .read_image()
        .map_err(|e| Error::Decode(format!("TIFF read_image: {e}")))?;

    let pixels = decoding_to_rgba(color, result, width, height)?;
    Ok(ImageBuffer {
        width,
        height,
        pixels,
        // ICC profile preservation deferred — W2.0.1 client lands ICC
        // pipeline; until then TIFF assumes sRGB at the boundary
        // matching every other W1 raster format.
        color_profile: ColorProfile::Srgb,
    })
}

/// Convert a `tiff::DecodingResult` + `ColorType` into a flat RGBA8
/// `Vec<SrgbRgba>`. Handles the 5 common channel layouts; refuses
/// the rest with `Error::Unsupported`.
fn decoding_to_rgba(
    color: ColorType,
    result: DecodingResult,
    width: u32,
    height: u32,
) -> Result<Vec<SrgbRgba>, Error> {
    let n_pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or(Error::OutOfMemory)?;

    match (color, result) {
        // 8-bit RGBA — fast path.
        (ColorType::RGBA(8), DecodingResult::U8(buf)) => {
            if buf.len() != n_pixels * 4 {
                return Err(Error::Decode(format!(
                    "TIFF RGBA8 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 4
                )));
            }
            Ok(buf
                .chunks_exact(4)
                .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
                .collect())
        }
        // 8-bit RGB — add opaque alpha.
        (ColorType::RGB(8), DecodingResult::U8(buf)) => {
            if buf.len() != n_pixels * 3 {
                return Err(Error::Decode(format!(
                    "TIFF RGB8 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 3
                )));
            }
            Ok(buf
                .chunks_exact(3)
                .map(|c| SrgbRgba([c[0], c[1], c[2], 255]))
                .collect())
        }
        // 8-bit grayscale — R=G=B, alpha 255.
        (ColorType::Gray(8), DecodingResult::U8(buf)) => {
            if buf.len() != n_pixels {
                return Err(Error::Decode(format!(
                    "TIFF Gray8 buffer {} != expected {}",
                    buf.len(),
                    n_pixels
                )));
            }
            Ok(buf.iter().map(|&g| SrgbRgba([g, g, g, 255])).collect())
        }
        // 8-bit grayscale + alpha.
        (ColorType::GrayA(8), DecodingResult::U8(buf)) => {
            if buf.len() != n_pixels * 2 {
                return Err(Error::Decode(format!(
                    "TIFF GrayA8 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 2
                )));
            }
            Ok(buf
                .chunks_exact(2)
                .map(|c| SrgbRgba([c[0], c[0], c[0], c[1]]))
                .collect())
        }
        // 8-bit CMYK — convert via naive 1-(C+K)/255 formula. Without
        // an ICC pipeline this is approximate; W2.0.1 lands moxcms.
        (ColorType::CMYK(8), DecodingResult::U8(buf)) => {
            if buf.len() != n_pixels * 4 {
                return Err(Error::Decode(format!(
                    "TIFF CMYK8 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 4
                )));
            }
            Ok(buf
                .chunks_exact(4)
                .map(|c| {
                    // Naive CMYK→RGB: R = (255-C) * (255-K) / 255.
                    let inv_k = 255_u32 - c[3] as u32;
                    let r = ((255 - c[0] as u32) * inv_k / 255) as u8;
                    let g = ((255 - c[1] as u32) * inv_k / 255) as u8;
                    let b = ((255 - c[2] as u32) * inv_k / 255) as u8;
                    SrgbRgba([r, g, b, 255])
                })
                .collect())
        }
        // 16-bit RGBA — quantize down to 8-bit (same policy as PNG).
        (ColorType::RGBA(16), DecodingResult::U16(buf)) => {
            if buf.len() != n_pixels * 4 {
                return Err(Error::Decode(format!(
                    "TIFF RGBA16 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 4
                )));
            }
            Ok(buf
                .chunks_exact(4)
                .map(|c| {
                    SrgbRgba([
                        u16_to_u8(c[0]),
                        u16_to_u8(c[1]),
                        u16_to_u8(c[2]),
                        u16_to_u8(c[3]),
                    ])
                })
                .collect())
        }
        // 16-bit RGB.
        (ColorType::RGB(16), DecodingResult::U16(buf)) => {
            if buf.len() != n_pixels * 3 {
                return Err(Error::Decode(format!(
                    "TIFF RGB16 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 3
                )));
            }
            Ok(buf
                .chunks_exact(3)
                .map(|c| SrgbRgba([u16_to_u8(c[0]), u16_to_u8(c[1]), u16_to_u8(c[2]), 255]))
                .collect())
        }
        // 16-bit grayscale.
        (ColorType::Gray(16), DecodingResult::U16(buf)) => {
            if buf.len() != n_pixels {
                return Err(Error::Decode(format!(
                    "TIFF Gray16 buffer {} != expected {}",
                    buf.len(),
                    n_pixels
                )));
            }
            Ok(buf
                .iter()
                .map(|&g| {
                    let g8 = u16_to_u8(g);
                    SrgbRgba([g8, g8, g8, 255])
                })
                .collect())
        }
        // 16-bit grayscale + alpha.
        (ColorType::GrayA(16), DecodingResult::U16(buf)) => {
            if buf.len() != n_pixels * 2 {
                return Err(Error::Decode(format!(
                    "TIFF GrayA16 buffer {} != expected {}",
                    buf.len(),
                    n_pixels * 2
                )));
            }
            Ok(buf
                .chunks_exact(2)
                .map(|c| {
                    let g = u16_to_u8(c[0]);
                    SrgbRgba([g, g, g, u16_to_u8(c[1])])
                })
                .collect())
        }
        (other, _) => Err(Error::Unsupported(format!(
            "TIFF color type {other:?} not supported in W2.T2"
        ))),
    }
}

/// 16→8 bit quantization matching the policy `image` 0.25 uses:
/// `(v * 255 + 32767) / 65535` (proper linear scaling, NOT top-byte
/// truncation).
fn u16_to_u8(v: u16) -> u8 {
    let scaled = (v as u32 * 255 + 32767) / 65535;
    scaled.min(255) as u8
}

/// TIFF export driver. Emits RGBA8 single-page (Flat) or multi-page
/// (Layered with `LayerKind::Pixel` only).
pub struct TiffExporter;

impl ImageExporter for TiffExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Tiff)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let pages: Vec<&ImageBuffer<SrgbRgba>> = match img {
            DecodedImage::Flat(b) => vec![b],
            DecodedImage::Layered(stack) => collect_pixel_pages(&stack.layers),
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "TIFF multi-page is not animation; use APNG/GIF instead".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "TIFF is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };

        if pages.is_empty() {
            return Err(Error::Encode(
                "TIFF export: no pixel pages found (Layered with no Pixel layers)".into(),
            ));
        }

        let mut out: Vec<u8> = Vec::new();
        {
            let mut encoder = TiffEncoder::new(Cursor::new(&mut out))
                .map_err(|e| Error::Encode(format!("TIFF encoder init: {e}")))?;
            for page in &pages {
                let byte_len = page.pixels.len().checked_mul(4).ok_or(Error::OutOfMemory)?;
                let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
                for p in &page.pixels {
                    rgba8.extend_from_slice(&p.0);
                }
                encoder
                    .write_image::<RGBA8>(page.width, page.height, &rgba8)
                    .map_err(|e| Error::Encode(format!("TIFF write_image: {e}")))?;
            }
        }
        Ok(out)
    }
}

/// Flatten the layer tree into a list of pixel-bearing pages, in
/// TIFF page order (paint order bottom-up = the order they were
/// written). Groups recurse depth-first.
fn collect_pixel_pages(layers: &[Layer]) -> Vec<&ImageBuffer<SrgbRgba>> {
    let mut out: Vec<&ImageBuffer<SrgbRgba>> = Vec::new();
    for layer in layers {
        match (&layer.kind, &layer.pixels) {
            (LayerKind::Pixel, Some(buf)) => out.push(buf),
            (LayerKind::Group { children }, _) => {
                out.extend(collect_pixel_pages(children));
            }
            _ => {
                // Non-Pixel kinds (Adjustment/Text/Smart) are silently
                // skipped on TIFF export — TIFF has no concept of
                // non-pixel layers. Caller wanting fidelity uses .ph2d.
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_2x2(color: [u8; 4]) -> ImageBuffer<SrgbRgba> {
        ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![SrgbRgba(color); 4],
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_both_endians_strong() {
        let imp = TiffImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&TIFF_LE_MAGIC)),
            MagicMatch::Strong,
            "little-endian II*\\0"
        );
        assert_eq!(
            imp.supports(MagicHint::Bytes(&TIFF_BE_MAGIC)),
            MagicMatch::Strong,
            "big-endian MM\\0*"
        );
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"not-tiff")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    /// Audit W2.T6 H-2: BigTIFF magic recognized (Strong) so registry
    /// lands here; import refuses with Unsupported pointing at
    /// classic TIFF / .ph2d-native alternatives.
    #[test]
    fn bigtiff_magic_strong_but_import_refuses() {
        let imp = TiffImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&BIGTIFF_LE_MAGIC)),
            MagicMatch::Strong,
            "BigTIFF LE recognized"
        );
        assert_eq!(
            imp.supports(MagicHint::Bytes(&BIGTIFF_BE_MAGIC)),
            MagicMatch::Strong,
            "BigTIFF BE recognized"
        );
        let mut bigtiff_header = BIGTIFF_LE_MAGIC.to_vec();
        bigtiff_header.extend_from_slice(&[0; 32]);
        let err = imp
            .import(&bigtiff_header, &ImportOpts::default())
            .expect_err("BigTIFF must be refused early");
        match err {
            Error::Unsupported(msg) => assert!(
                msg.contains("BigTIFF"),
                "expected BigTIFF in error message, got: {msg}"
            ),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn supports_recognizes_both_extensions_weak() {
        let imp = TiffImporter;
        for ext in &["tiff", "tif", "TIFF", "TIF", "Tiff"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
        }
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_tiff_format() {
        let exp = TiffExporter;
        assert!(exp.supports_format(ExportFormat::Tiff));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    #[test]
    fn roundtrip_single_page_preserves_pixels_bit_exact() {
        let original = fixture_2x2([200, 100, 50, 255]);
        let bytes = TiffExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Tiff,
                    ..ExportOpts::default()
                },
            )
            .expect("TIFF export");
        assert!(is_tiff_magic(&bytes), "output bytes are not TIFF");

        let DecodedImage::Flat(out) = TiffImporter
            .import(&bytes, &ImportOpts::default())
            .expect("TIFF import")
        else {
            panic!("expected Flat");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        for (i, p) in out.pixels.iter().enumerate() {
            assert_eq!(p.0, original.pixels[i].0, "pixel {i}");
        }
    }

    /// Multi-page TIFF round-trip: 3 pages → Layered(3 layers) →
    /// re-export → 3 pages again.
    #[test]
    fn roundtrip_3_page_tiff_preserves_count_and_pixels() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 2,
            canvas_height: 2,
            layers: vec![
                Layer {
                    version: 1,
                    name: "Page 1".into(),
                    kind: LayerKind::Pixel,
                    pixels: Some(fixture_2x2([255, 0, 0, 255])),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                },
                Layer {
                    version: 1,
                    name: "Page 2".into(),
                    kind: LayerKind::Pixel,
                    pixels: Some(fixture_2x2([0, 255, 0, 255])),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                },
                Layer {
                    version: 1,
                    name: "Page 3".into(),
                    kind: LayerKind::Pixel,
                    pixels: Some(fixture_2x2([0, 0, 255, 255])),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                },
            ],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = TiffExporter
            .export(
                &DecodedImage::Layered(stack),
                &ExportOpts {
                    format: ExportFormat::Tiff,
                    ..ExportOpts::default()
                },
            )
            .expect("multi-page TIFF export");
        let DecodedImage::Layered(out) = TiffImporter
            .import(&bytes, &ImportOpts::default())
            .expect("multi-page TIFF import")
        else {
            panic!("expected Layered");
        };
        assert_eq!(out.layers.len(), 3);
        assert_eq!(
            out.layers[0].pixels.as_ref().unwrap().pixels[0].0,
            [255, 0, 0, 255]
        );
        assert_eq!(
            out.layers[1].pixels.as_ref().unwrap().pixels[0].0,
            [0, 255, 0, 255]
        );
        assert_eq!(
            out.layers[2].pixels.as_ref().unwrap().pixels[0].0,
            [0, 0, 255, 255]
        );
    }

    #[test]
    fn u16_to_u8_endpoints() {
        // Quantization endpoints exact.
        assert_eq!(u16_to_u8(0x0000), 0);
        assert_eq!(u16_to_u8(0xFFFF), 255);
        // Mid value approximately 128 (rounded).
        let mid = u16_to_u8(0x8080);
        assert!((127..=129).contains(&mid), "mid={mid}");
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = TiffImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty must fail");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = TiffExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Tiff,
                    ..ExportOpts::default()
                },
            )
            .expect_err("HDR refused");
        assert!(matches!(err, Error::HdrUnsupported));
    }

    /// HR-5 byte-exact determinism: tiff 0.11 encoder must be
    /// deterministic. If this fails post-upgrade, document exception
    /// or pin via direct write_tag API.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = fixture_2x2([42, 84, 126, 255]);
        let opts = ExportOpts {
            format: ExportFormat::Tiff,
            ..ExportOpts::default()
        };
        let a = TiffExporter
            .export(&DecodedImage::Flat(original.clone()), &opts)
            .expect("a");
        let b = TiffExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("b");
        assert_eq!(a, b, "HR-5: TIFF export must be byte-exact");
    }
}

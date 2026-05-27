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

/// Strict BigTIFF detection: not just the 4-byte magic, but the
/// BigTIFF spec's bytes 4-7 (offset-size word = 0x0008, reserved =
/// 0x0000). Audit nova M-N3 (2026-05-26): the 4-byte magic alone
/// can false-positive on random binary files that happen to start
/// with `II+\0`.
fn is_bigtiff_magic(b: &[u8]) -> bool {
    if b.len() < 8 {
        return false;
    }
    if b.starts_with(&BIGTIFF_LE_MAGIC) {
        // offset-size LE = 0x0008 at bytes 4-5; reserved 0x0000 at 6-7.
        return b[4] == 0x08 && b[5] == 0x00 && b[6] == 0x00 && b[7] == 0x00;
    }
    if b.starts_with(&BIGTIFF_BE_MAGIC) {
        // offset-size BE = 0x0008 at bytes 4-5; reserved 0x0000 at 6-7.
        return b[4] == 0x00 && b[5] == 0x08 && b[6] == 0x00 && b[7] == 0x00;
    }
    false
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

        // Walk additional pages, if any. Audit-7 Lens J CRITICAL J-2:
        // cap page count — hostile TIFF with 10000 IFDs chained would
        // attempt 10000 × MAX_RASTER_DIMENSION² alloc. Cap = 256 covers
        // any plausible multi-page real-world TIFF (catalogue scans
        // rarely exceed 100 pages); higher counts should split files.
        const MAX_PAGES: usize = 256;
        let mut pages: Vec<ImageBuffer<SrgbRgba>> = vec![first];
        while decoder.more_images() {
            if pages.len() >= MAX_PAGES {
                return Err(Error::Decode(format!(
                    "TIFF has more than MAX_PAGES={MAX_PAGES} IFDs; \
                     refuse to walk further (potential DoS vector)"
                )));
            }
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

    /// Audit W2.T6 H-2 + nova M-N3 (2026-05-26): BigTIFF requires
    /// offset-size 0x0008 + reserved 0x0000 at bytes 4-7. Test both:
    /// (a) full spec-compliant header → Strong + Unsupported,
    /// (b) magic-only (bytes 4-7 garbage) → NOT Strong (avoids false
    /// positive on random binary files starting with `II+\0`).
    #[test]
    fn bigtiff_magic_requires_spec_bytes_strong_then_unsupported() {
        let imp = TiffImporter;
        // Full-spec BigTIFF LE: II + 0x2B 0x00 + 0x08 0x00 + 0x00 0x00.
        let mut full_le = BIGTIFF_LE_MAGIC.to_vec();
        full_le.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]);
        full_le.extend_from_slice(&[0; 24]);
        assert_eq!(
            imp.supports(MagicHint::Bytes(&full_le)),
            MagicMatch::Strong,
            "BigTIFF LE spec-compliant recognized"
        );

        // Full-spec BigTIFF BE: MM + 0x00 0x2B + 0x00 0x08 + 0x00 0x00.
        let mut full_be = BIGTIFF_BE_MAGIC.to_vec();
        full_be.extend_from_slice(&[0x00, 0x08, 0x00, 0x00]);
        full_be.extend_from_slice(&[0; 24]);
        assert_eq!(
            imp.supports(MagicHint::Bytes(&full_be)),
            MagicMatch::Strong,
            "BigTIFF BE spec-compliant recognized"
        );

        // Magic-only with garbage bytes 4-7 → NOT Strong (avoid
        // false-positive on coincidental binary).
        let mut magic_only = BIGTIFF_LE_MAGIC.to_vec();
        magic_only.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(
            imp.supports(MagicHint::Bytes(&magic_only)),
            MagicMatch::None,
            "BigTIFF magic without spec bytes 4-7 must NOT false-positive"
        );

        // Spec-compliant header passes through to import → Unsupported.
        let err = imp
            .import(&full_le, &ImportOpts::default())
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
    /// W3 pre-gate fixture (audit-5 Lens A MEDIUM + audit-6 Lens C HIGH):
    /// exercise the CMYK8 decode branch with a real TIFF fixture
    /// produced by `tiff` 0.11 encoder. Four arms cover endpoint, K-only,
    /// cross-term, and pure-black:
    /// - K=0 (pure-cyan): exercises the C-only path.
    /// - K-only (K=128, C=M=Y=0): exercises K-attenuates-everything.
    /// - cross-term (C=K=128): exercises both factors multiplied — the
    ///   real stress of "naive CMYK→RGB" (without this arm we'd only
    ///   exercise trivial 255*inv_k/255 paths).
    /// - K=255 pure-black: detects `255-K` underflow / missing-K bugs.
    ///
    /// Formula is exact integer (no rounding): `(255-C)*(255-K)/255`.
    /// Audit-6 Lens C demand: tolerances were ±1 — masked off-by-one
    /// from `>>8` regressions. Now `assert_eq!` exact.
    #[test]
    fn cmyk8_decode_via_naive_conversion() {
        use tiff::encoder::{TiffEncoder, colortype::CMYK8};

        fn encode_cmyk_1x1(cmyk: [u8; 4]) -> Vec<u8> {
            let mut bytes: Vec<u8> = Vec::new();
            {
                let mut enc = TiffEncoder::new(Cursor::new(&mut bytes)).expect("encoder init");
                enc.write_image::<CMYK8>(1, 1, &cmyk).expect("write CMYK8");
            }
            bytes
        }
        fn decode_one(bytes: &[u8]) -> [u8; 4] {
            let DecodedImage::Flat(out) = TiffImporter
                .import(bytes, &ImportOpts::default())
                .expect("CMYK decode")
            else {
                panic!("expected Flat");
            };
            out.pixels[0].0
        }

        // Arm 1: pure-cyan K=0 → (255-255)*(255-0)/255=0, (255-0)*(255-0)/255=255.
        assert_eq!(
            decode_one(&encode_cmyk_1x1([255, 0, 0, 0])),
            [0, 255, 255, 255],
            "arm 1 pure-cyan K=0"
        );

        // Arm 2: K-only (K=128, C=M=Y=0) → all = (255*127)/255 = 127.
        assert_eq!(
            decode_one(&encode_cmyk_1x1([0, 0, 0, 128])),
            [127, 127, 127, 255],
            "arm 2 K-only K=128"
        );

        // Arm 3: cross-term C=K=128 → R = (127*127)/255 = 16129/255 = 63
        // (integer truncation). G = B = (255*127)/255 = 127 (no C/M/Y).
        // This arm catches regressions where the decoder loses one of
        // the two factors (would emit 127 or 0).
        assert_eq!(
            decode_one(&encode_cmyk_1x1([128, 0, 0, 128])),
            [63, 127, 127, 255],
            "arm 3 cross-term C=K=128"
        );

        // Arm 4: pure-black K=255 → all = X*0/255 = 0. Catches
        // `255-K` underflow or missing-K-term bugs.
        assert_eq!(
            decode_one(&encode_cmyk_1x1([0, 0, 0, 255])),
            [0, 0, 0, 255],
            "arm 4 pure-black K=255"
        );
    }

    /// W3 pre-gate fixture (audit-5 Lens B + audit-6 Lens C HIGH):
    /// exercise the 16-bit RGBA decode branch validating the
    /// quantization formula `(v*255+32767)/65535` at endpoints + mid.
    ///
    /// Formula is exact integer (no rounding):
    /// - 0x0000 → (0*255+32767)/65535 = 32767/65535 = 0
    /// - 0x4040 → (16448*255+32767)/65535 = 4227007/65535 = 64
    /// - 0x8080 → (32896*255+32767)/65535 = 8421247/65535 = 128
    /// - 0xC0C0 → (49344*255+32767)/65535 = 12614287/65535 = 192
    /// - 0xFFFF → (65535*255+32767)/65535 = 16744192/65535 = 255
    ///
    /// Audit-6 Lens C demand: tolerances were ±1 — masked off-by-one
    /// regressions like `(v*255)/65535` (without `+32767`). Now
    /// `assert_eq!` exact across the sweep + an alpha-channel arm
    /// proving alpha is NOT synthesized-to-255 (regression guard).
    #[test]
    fn rgba16_decode_quantizes_to_8bit() {
        use tiff::encoder::{TiffEncoder, colortype::RGBA16};

        fn encode_rgba16(rgba: [u16; 4]) -> Vec<u8> {
            let mut bytes: Vec<u8> = Vec::new();
            {
                let mut enc = TiffEncoder::new(Cursor::new(&mut bytes)).expect("encoder init");
                enc.write_image::<RGBA16>(1, 1, &rgba)
                    .expect("write RGBA16");
            }
            bytes
        }
        fn decode_one(bytes: &[u8]) -> [u8; 4] {
            let DecodedImage::Flat(out) = TiffImporter
                .import(bytes, &ImportOpts::default())
                .expect("RGBA16 decode")
            else {
                panic!("expected Flat");
            };
            out.pixels[0].0
        }

        // 5-arm sweep — endpoints + mid-low + midpoint + mid-high.
        // Alpha opaque at 0xFFFF so it doesn't perturb RGB asserts.
        for (label, raw, expected) in [
            ("endpoint low 0x0000", 0x0000_u16, 0_u8),
            ("mid-low 0x4040", 0x4040_u16, 64_u8),
            ("midpoint 0x8080", 0x8080_u16, 128_u8),
            ("mid-high 0xC0C0", 0xC0C0_u16, 192_u8),
            ("endpoint high 0xFFFF", 0xFFFF_u16, 255_u8),
        ] {
            let v = decode_one(&encode_rgba16([raw, raw, raw, 0xFFFF]));
            assert_eq!(
                v,
                [expected, expected, expected, 255],
                "{label}: expected RGBA=[{expected};3, 255]"
            );
        }

        // Alpha-channel arm: alpha=0x8080 must quantize to 128 (NOT
        // synthesized to 255). Catches a regression where the RGBA16
        // fast-path replaces c[3] with literal 255.
        let v = decode_one(&encode_rgba16([0xFFFF, 0xFFFF, 0xFFFF, 0x8080]));
        assert_eq!(v, [255, 255, 255, 128], "alpha=0x8080 must round to 128");
    }

    /// W3 pre-gate (LOCAL drift guard, Mac aarch64 only): pin blake3
    /// hash of canonical TIFF export. Single-platform pin — tiff
    /// 0.11 encoder uses SIMD-divergent paths per target, so bytes
    /// differ between `aarch64-apple-darwin` /
    /// `x86_64-unknown-linux-gnu` / `x86_64-pc-windows-msvc`.
    /// CI matrix passes via `#[cfg]` gate; multi-platform pinning
    /// deferred (entry:
    /// `crates/ph2d-imageio-tiff/src/lib.rs::export_golden_blake3_local_drift_pinned_macos_silicon`).
    // CONVENTION (audit-6 Lens B HIGH): keep `const GOLDEN_BLAKE3`
    // inside the fn body. See ph2d-imageio-png/src/lib.rs for full
    // rationale.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn export_golden_blake3_local_drift_pinned_macos_silicon() {
        let original = fixture_2x2([200, 100, 50, 255]);
        let opts = ExportOpts {
            format: ExportFormat::Tiff,
            ..ExportOpts::default()
        };
        let bytes = TiffExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("TIFF golden export");
        let hash = blake3::hash(&bytes);
        const GOLDEN_BLAKE3: &str =
            "9f1a5aceb551cd5e1be277da9c50de091d5fe45a99ddb2b65bc1af1ecee33d17";
        assert_eq!(
            hash.to_hex().to_string(),
            GOLDEN_BLAKE3,
            "LOCAL drift (macOS aarch64): TIFF export blake3 drifted"
        );
    }

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

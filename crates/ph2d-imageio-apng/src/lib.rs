#![forbid(unsafe_code)]
//! `ph2d-imageio-apng` — Animated PNG decode (W2.T3) + single-frame
//! encode.
//!
//! Pure-Rust via `png` 0.18 direct — `image` 0.25's `DynamicImage`
//! decodes only the first frame of an APNG and opaques the
//! animation-control + frame-control chunks away. We need direct
//! access to:
//!
//! - `info.animation_control` → `Option<AnimationControl { num_frames, num_plays }>`.
//! - `reader.info().frame_control` per frame → `FrameControl {
//!   x_offset, y_offset, delay_num, delay_den, dispose_op, blend_op }`.
//!
//! Single-frame encode reuses `image` 0.25's PNG codec for parity
//! with `ph2d-imageio-png` byte output. Multi-frame APNG encode is
//! deferred to W3+; emitting an APNG today would mean hand-writing
//! `acTL`/`fcTL`/`fdAT` chunks, and there's no Painter client that
//! needs to *save* animations until the timeline UI lands.
//!
//! ### What W2.T3 covers
//!
//! - Magic: PNG `\x89PNG\r\n\x1a\n` **plus an `acTL` chunk** found
//!   within the first 64 KB → `Strong` (audit W2.T6 C-1 fix). Plain
//!   PNG (no `acTL`) → `None` so the dedicated PNG importer takes
//!   over via registry dispatch. `"apng"` extension (Weak).
//! - Decode: walk every `acTL.num_frames` frame via
//!   `reader.next_frame()`; per-frame `delay_ms`, `offset_xy`,
//!   `dispose_op`, `blend_op` extracted and preserved in `AnimFrame`.
//! - Single-frame APNG (no `acTL`) → `DecodedImage::Flat`. Multi-
//!   frame → `DecodedImage::Animated(Vec<AnimFrame>)`.
//! - Encode: `Flat` → single-frame PNG bytes (same byte output as
//!   `ph2d-imageio-png`). `Animated` → `Error::Unsupported` until W3+.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (W3+)
//!
//! - **Multi-frame APNG encode**: needs hand-written `acTL` +
//!   `fcTL` + `fdAT` chunk construction (no maintained pure-Rust
//!   encoder crate as of 2026-05). Implementation lands when the
//!   Painter timeline UI is the first real client.
//! - tRNS chunk preservation across frames.
//! - 16-bit APNG (would need `DecodedImage::Flat16` variant).
//! - `default_image`-only optimisation (APNG spec allows a static
//!   "fallback" image distinct from the first animation frame).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    AnimBlendOp, AnimFrame, ColorProfile, DecodedImage, DisposeOp, Error, ExportFormat, ExportOpts,
    ExporterRegistry, ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry,
    MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};
use std::io::Cursor;

/// PNG's 8-byte signature (APNG is layered on top of plain PNG).
const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Look for an `acTL` chunk in the first few KB of a PNG byte buffer.
/// APNG spec requires `acTL` to appear BEFORE the first `IDAT` (so the
/// chunk is in the header region, easy to scan). We scan up to 64 KB
/// defensively — enough to cover IHDR + iCCP + sBIT + sRGB + cHRM +
/// gAMA + pHYs + tEXt headers without parsing them properly.
///
/// Audit W2.T6 C-1: without this sniff, APNG importer would claim
/// `Strong` on every PNG byte stream and steal dispatch from the
/// dedicated PNG importer (registry FIFO + alphabetical codegen
/// order). With the sniff, plain PNG → `None` here, dispatch falls
/// through to the PNG importer cleanly.
fn has_actl_chunk(b: &[u8]) -> bool {
    let scan_end = b.len().min(65_536);
    // `acTL` 4-byte ASCII tag — searchable as a window.
    b[..scan_end].windows(4).any(|w| w == b"acTL")
}

/// Register the APNG importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(ApngImporter));
}

/// Register the APNG exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(ApngExporter));
}

/// APNG import driver.
pub struct ApngImporter;

impl ImageImporter for ApngImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            // Audit W2.T6 C-1 (2026-05-26): the previous version claimed
            // `Strong` on bare PNG magic, but **registry FIFO** + the
            // codegen-sorted order (`apng` before `png` alphabetical)
            // meant every `.png` file routed via `MagicHint::Bytes`
            // landed here instead of the dedicated PNG importer —
            // performance regression and silent semantic shift. Fix:
            // **sniff for the `acTL` chunk** inline; only claim Strong
            // when this is actually an APNG.
            MagicHint::Bytes(b) if b.starts_with(&PNG_MAGIC) && has_actl_chunk(b) => {
                MagicMatch::Strong
            }
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("apng") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        let mut decoder = png::Decoder::new(Cursor::new(src));
        // Force the decoder to expand sub-8-bit / palette / grayscale
        // into RGB and strip 16-bit down to 8-bit, so every frame
        // comes back in a uniform color shape.
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder
            .read_info()
            .map_err(|e| Error::Decode(format!("APNG read_info: {e}")))?;

        let info = reader.info();
        let (width, height) = (info.width, info.height);
        if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        if width == 0 || height == 0 {
            return Err(Error::Decode(format!(
                "APNG has zero-sized dimension: {width}×{height}"
            )));
        }

        let animation_control = info.animation_control().copied();
        let post_expand_color = info.color_type;
        drop(info); // releases the borrow before next_frame

        match animation_control {
            None => {
                // Plain PNG (no acTL).
                let frame = read_one_frame(&mut reader, width, height, post_expand_color)?;
                Ok(DecodedImage::Flat(frame.image))
            }
            Some(act) => {
                // Multi-frame APNG.
                let mut anim: Vec<AnimFrame> = Vec::with_capacity(act.num_frames as usize);
                for _ in 0..act.num_frames {
                    let frame = read_one_frame(&mut reader, width, height, post_expand_color)?;
                    anim.push(frame);
                }
                if anim.is_empty() {
                    return Err(Error::Decode("APNG acTL claims 0 frames".into()));
                }
                if anim.len() == 1 {
                    Ok(DecodedImage::Flat(anim.pop().expect("len==1").image))
                } else {
                    Ok(DecodedImage::Animated(anim))
                }
            }
        }
    }
}

/// Read one PNG/APNG frame into an `AnimFrame`. Reads the per-frame
/// fcTL (if present) to populate offset / delay / dispose / blend
/// metadata; for plain (non-animated) PNG the fcTL is `None` and we
/// fall back to defaults.
fn read_one_frame(
    reader: &mut png::Reader<Cursor<&[u8]>>,
    canvas_width: u32,
    canvas_height: u32,
    post_expand_color: png::ColorType,
) -> Result<AnimFrame, Error> {
    let buffer_size = reader.output_buffer_size().ok_or_else(|| {
        Error::Decode("APNG output_buffer_size not available before frame info".into())
    })?;
    let mut buf = vec![0u8; buffer_size];
    let output_info = reader
        .next_frame(&mut buf)
        .map_err(|e| Error::Decode(format!("APNG next_frame: {e}")))?;

    // Per-frame metadata via fcTL (animation control sub-chunks).
    let info = reader.info();
    let fc = info.frame_control().copied();
    drop(info);

    let (frame_w, frame_h) = (output_info.width, output_info.height);
    let offset_xy = fc
        .map(|f| (f.x_offset as i32, f.y_offset as i32))
        .unwrap_or((0, 0));
    let delay_ms = fc
        .map(|f| {
            if f.delay_den == 0 {
                // Per APNG spec, delay_den == 0 → treat as 100 (so
                // delay_num is in units of 1/100 sec).
                f.delay_num as u32 * 10
            } else {
                f.delay_num as u32 * 1000 / f.delay_den as u32
            }
        })
        .unwrap_or(0);
    let dispose_op = fc
        .map(|f| match f.dispose_op {
            png::DisposeOp::None => DisposeOp::None,
            png::DisposeOp::Background => DisposeOp::Background,
            png::DisposeOp::Previous => DisposeOp::Previous,
        })
        .unwrap_or(DisposeOp::None);
    let blend_op = fc
        .map(|f| match f.blend_op {
            png::BlendOp::Source => AnimBlendOp::Source,
            png::BlendOp::Over => AnimBlendOp::Over,
        })
        .unwrap_or(AnimBlendOp::Source);

    // Convert raw buf → RGBA8 SrgbRgba pixels.
    let pixels = png_pixels_to_rgba(
        &buf[..output_info.buffer_size()],
        post_expand_color,
        frame_w,
        frame_h,
    )?;
    let _ = (canvas_width, canvas_height); // (frame dims may differ from canvas — preserved via offset_xy)

    Ok(AnimFrame {
        image: ImageBuffer {
            width: frame_w,
            height: frame_h,
            pixels,
            color_profile: ColorProfile::Srgb,
        },
        delay_ms,
        offset_xy,
        dispose_op,
        blend_op,
        // tRNS palette-index transparency is currently dropped on
        // import — see crate doc out-of-scope. Most modern APNG ships
        // with proper alpha channel.
        transparent_index: None,
    })
}

fn png_pixels_to_rgba(
    raw: &[u8],
    color: png::ColorType,
    width: u32,
    height: u32,
) -> Result<Vec<SrgbRgba>, Error> {
    let n_pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or(Error::OutOfMemory)?;
    match color {
        png::ColorType::Rgba => {
            if raw.len() != n_pixels * 4 {
                return Err(Error::Decode(format!(
                    "APNG RGBA buffer {} != expected {}",
                    raw.len(),
                    n_pixels * 4
                )));
            }
            Ok(raw
                .chunks_exact(4)
                .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
                .collect())
        }
        png::ColorType::Rgb => {
            if raw.len() != n_pixels * 3 {
                return Err(Error::Decode(format!(
                    "APNG RGB buffer {} != expected {}",
                    raw.len(),
                    n_pixels * 3
                )));
            }
            Ok(raw
                .chunks_exact(3)
                .map(|c| SrgbRgba([c[0], c[1], c[2], 255]))
                .collect())
        }
        png::ColorType::GrayscaleAlpha => {
            if raw.len() != n_pixels * 2 {
                return Err(Error::Decode(format!(
                    "APNG GrayA buffer {} != expected {}",
                    raw.len(),
                    n_pixels * 2
                )));
            }
            Ok(raw
                .chunks_exact(2)
                .map(|c| SrgbRgba([c[0], c[0], c[0], c[1]]))
                .collect())
        }
        png::ColorType::Grayscale => {
            if raw.len() != n_pixels {
                return Err(Error::Decode(format!(
                    "APNG Gray buffer {} != expected {}",
                    raw.len(),
                    n_pixels
                )));
            }
            Ok(raw.iter().map(|&g| SrgbRgba([g, g, g, 255])).collect())
        }
        other => Err(Error::Unsupported(format!(
            "APNG color type {other:?} not supported (W2.T3 covers Gray/GrayA/RGB/RGBA post-EXPAND)"
        ))),
    }
}

/// APNG export driver. Single-frame Flat only in W2.T3 (multi-frame
/// APNG encoding deferred to W3+).
pub struct ApngExporter;

impl ImageExporter for ApngExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Apng)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        match img {
            DecodedImage::Flat(buf) => encode_single_frame(buf),
            DecodedImage::Animated(_) => Err(Error::Unsupported(
                "APNG multi-frame encode deferred to W3+ (no maintained pure-Rust APNG \
                 encoder crate). Use `.ph2d-native` for full-fidelity animation save."
                    .into(),
            )),
            DecodedImage::FlatHdr(_) => Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => Err(Error::MissingLayer),
            DecodedImage::Vector(_) => Err(Error::Unsupported(
                "APNG is raster; rasterize the VectorDoc first".into(),
            )),
        }
    }
}

fn encode_single_frame(buf: &ImageBuffer<SrgbRgba>) -> Result<Vec<u8>, Error> {
    let byte_len = buf.pixels.len().checked_mul(4).ok_or(Error::OutOfMemory)?;
    let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
    for p in &buf.pixels {
        rgba8.extend_from_slice(&p.0);
    }
    let img_buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(buf.width, buf.height, rgba8)
        .ok_or_else(|| {
            Error::Encode(format!(
                "APNG single-frame pixel count {} mismatched {}×{}",
                buf.pixels.len(),
                buf.width,
                buf.height
            ))
        })?;
    let mut out: Vec<u8> = Vec::new();
    img_buf
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| Error::Encode(format!("APNG single-frame PNG encode: {e}")))?;
    Ok(out)
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
    fn supports_only_strong_when_actl_chunk_present() {
        // Audit W2.T6 C-1: plain PNG magic without acTL must NOT
        // dispatch to APNG (registry FIFO would steal PNG traffic).
        let imp = ApngImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&PNG_MAGIC)),
            MagicMatch::None,
            "plain PNG magic alone must not claim Strong"
        );
        // Synthesise a buffer with PNG magic + acTL marker further in.
        let mut with_actl = PNG_MAGIC.to_vec();
        with_actl.extend_from_slice(&[0; 16]); // fake IHDR padding
        with_actl.extend_from_slice(b"acTL"); // chunk tag
        with_actl.extend_from_slice(&[0; 16]); // fake chunk data
        assert_eq!(
            imp.supports(MagicHint::Bytes(&with_actl)),
            MagicMatch::Strong,
            "PNG magic + acTL → Strong (real APNG)"
        );
        // Non-PNG magic → None regardless.
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"not-a-png")),
            MagicMatch::None
        );
    }

    #[test]
    fn supports_recognizes_apng_extension_weak() {
        let imp = ApngImporter;
        for ext in &["apng", "APNG", "Apng"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
        }
        // png/jpeg/gif: APNG crate does NOT claim them.
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
        assert_eq!(imp.supports(MagicHint::Extension("gif")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_apng_format() {
        let exp = ApngExporter;
        assert!(exp.supports_format(ExportFormat::Apng));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    /// Plain PNG (no acTL) decodes to Flat via the APNG path.
    #[test]
    fn import_of_plain_png_yields_flat() {
        // Encode a 2×2 PNG using image crate, then decode via our APNG
        // importer.
        let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 128, 128, 128, 200,
            ],
        )
        .expect("rgba8");
        let mut bytes: Vec<u8> = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("encode plain PNG");
        let decoded = ApngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import plain PNG via APNG path");
        let DecodedImage::Flat(out) = decoded else {
            panic!("plain PNG must be Flat through APNG importer");
        };
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.pixels.len(), 4);
        assert_eq!(out.pixels[0].0, [255, 0, 0, 255]);
    }

    /// Single-frame round-trip: Flat → PNG bytes → Flat (W2.T3 encoder
    /// only supports Flat; this is the workhorse path).
    #[test]
    fn roundtrip_flat_single_frame_bit_exact() {
        let original = fixture_2x2([200, 100, 50, 255]);
        let bytes = ApngExporter
            .export(
                &DecodedImage::Flat(original.clone()),
                &ExportOpts {
                    format: ExportFormat::Apng,
                    ..ExportOpts::default()
                },
            )
            .expect("APNG single-frame export");
        assert!(
            bytes.starts_with(&PNG_MAGIC),
            "output bytes are not PNG/APNG"
        );
        let DecodedImage::Flat(out) = ApngImporter
            .import(&bytes, &ImportOpts::default())
            .expect("APNG import single-frame")
        else {
            panic!("expected Flat");
        };
        for (i, p) in out.pixels.iter().enumerate() {
            assert_eq!(p.0, original.pixels[i].0, "pixel {i}");
        }
    }

    #[test]
    fn export_rejects_animated_with_actionable_unsupported() {
        // Audit-aware: the W2.T3 encoder cannot ship multi-frame APNG.
        // Caller must use .ph2d-native (full fidelity) or wait for W3+.
        let anim = DecodedImage::Animated(vec![
            AnimFrame {
                image: fixture_2x2([255, 0, 0, 255]),
                delay_ms: 100,
                offset_xy: (0, 0),
                dispose_op: DisposeOp::None,
                blend_op: AnimBlendOp::Source,
                transparent_index: None,
            },
            AnimFrame {
                image: fixture_2x2([0, 255, 0, 255]),
                delay_ms: 100,
                offset_xy: (0, 0),
                dispose_op: DisposeOp::None,
                blend_op: AnimBlendOp::Source,
                transparent_index: None,
            },
        ]);
        let err = ApngExporter
            .export(
                &anim,
                &ExportOpts {
                    format: ExportFormat::Apng,
                    ..ExportOpts::default()
                },
            )
            .expect_err("multi-frame APNG must be Unsupported in W2.T3");
        match err {
            Error::Unsupported(msg) => assert!(
                msg.contains("multi-frame"),
                "expected multi-frame note, got: {msg}"
            ),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = ApngExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Apng,
                    ..ExportOpts::default()
                },
            )
            .expect_err("HDR refused");
        assert!(matches!(err, Error::HdrUnsupported));
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = ApngImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    /// HR-5 byte-exact determinism (single-frame path; multi-frame
    /// encoder doesn't exist yet).
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = fixture_2x2([42, 84, 126, 255]);
        let opts = ExportOpts {
            format: ExportFormat::Apng,
            ..ExportOpts::default()
        };
        let a = ApngExporter
            .export(&DecodedImage::Flat(original.clone()), &opts)
            .expect("a");
        let b = ApngExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("b");
        assert_eq!(a, b, "HR-5: APNG single-frame export must be byte-exact");
    }
}

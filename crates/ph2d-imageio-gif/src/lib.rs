#![forbid(unsafe_code)]
//! `ph2d-imageio-gif` — GIF decode + encode (W1.T4).
//!
//! First W1 format that produces [`DecodedImage::Animated`] for
//! multi-frame sources. Single-frame GIFs come back as
//! [`DecodedImage::Flat`] for parity with PNG/JPEG/WebP — callers that
//! treat every GIF as raster can ignore the variant distinction.
//!
//! Pure-Rust via `image` 0.25's `gif` feature (bundled `image-gif`
//! backend).
//!
//! ### What W1.T4 covers
//!
//! - Magic-byte recognition by both `GIF87a` and `GIF89a` signatures.
//! - Single-frame → `Flat`, multi-frame → `Animated(Vec<AnimFrame>)`.
//! - Per-frame `delay_ms`, `offset_xy` preserved on import.
//! - Encode single-frame via the convenience `DynamicImage::write_to`
//!   path; encode multi-frame via `GifEncoder` + repeat-forever.
//! - Same dimension cap (32K) + Truncated + Decode guards.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (W2)
//!
//! - GIF `dispose_method` extraction from frames: `image::Frame` does
//!   NOT surface `DisposalMethod` through its public API (the
//!   underlying `image-gif` crate has it, but the wrapper opaques the
//!   chunk). Imported `AnimFrame::dispose_op` defaults to `None` for
//!   every frame in W1.T4; round-trip preserves visual but loses
//!   semantic disposal info. Reopen when `image` 0.26 surfaces the
//!   field, or switch to `image-gif` direct (deps adjustment).
//! - GIF `transparent_index` extraction — same opaque-wrapper story.
//! - Encoder palette quantisation tuning.
//! - Sub-frame offset on encode (W1.T4 encodes every frame at (0,0);
//!   `AnimFrame::offset_xy` is preserved on import but reset to (0,0)
//!   on export; image crate's `Frame::from_parts` accepts offset but
//!   the rendering side hasn't been verified end-to-end here).

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    AnimBlendOp, AnimFrame, ColorProfile, DecodedImage, DisposeOp, Error, ExportFormat, ExportOpts,
    ExporterRegistry, ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry,
    MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};
// Bomb-defence cap hoisted to `ph2d_imageio::MAX_RASTER_DIMENSION`.

/// Maximum frame count we accept on import. Audit A-Crit2 (2026-05-26):
/// `collect_frames()` on a 1000-frame 1024×1024 GIF allocates ~4 GiB
/// without this cap; `OutOfMemory` downstream is too late for the
/// `Cancelled`/UI-feedback path the shell will wire in W2+. 1024 frames
/// at 4K resolution caps at ~32 GiB worst case (still big, but bounded;
/// W2 will add per-frame streaming via the `image-gif` direct backend
/// to lift this cap.) Real-world GIFs ship < 200 frames typically.
/// Audit-8 Lens O O-1 (2026-05-26): use the contract-level
/// `MAX_ANIMATION_FRAMES` instead of a private constant. Kept as a
/// local alias only to preserve call-site readability without
/// changing semantics — drop in a future cleanup pass.
const MAX_FRAMES: usize = ph2d_imageio::MAX_ANIMATION_FRAMES as usize;

/// GIF87a (`GIF87a`) and GIF89a (`GIF89a`) signatures. Both decoders
/// accept either; we recognise both as Strong.
const GIF87_MAGIC: &[u8] = b"GIF87a";
const GIF89_MAGIC: &[u8] = b"GIF89a";

fn is_gif_magic(b: &[u8]) -> bool {
    b.starts_with(GIF87_MAGIC) || b.starts_with(GIF89_MAGIC)
}

/// Register the GIF importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(GifImporter));
}

/// Register the GIF exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(GifExporter));
}

/// GIF import driver.
pub struct GifImporter;

/// Convert one `image::Frame` to our `ImageBuffer<SrgbRgba>`.
fn frame_to_image_buffer(frame: &image::Frame) -> ImageBuffer<SrgbRgba> {
    let buf = frame.buffer();
    let (width, height) = buf.dimensions();
    let pixels: Vec<SrgbRgba> = buf
        .chunks_exact(4)
        .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
        .collect();
    ImageBuffer {
        width,
        height,
        pixels,
        color_profile: ColorProfile::Srgb,
    }
}

/// Convert one `image::Frame` to our `AnimFrame`.
fn frame_to_anim_frame(frame: &image::Frame) -> AnimFrame {
    let image = frame_to_image_buffer(frame);
    let (numer, denom) = frame.delay().numer_denom_ms();
    // Delay numer/denom: protect against zero denom (image 0.25 returns
    // (numer, 1) for typical GIF delays; defensive saturation here).
    let delay_ms = numer.checked_div(denom).unwrap_or(0);
    AnimFrame {
        image,
        delay_ms,
        offset_xy: (frame.left() as i32, frame.top() as i32),
        // image::Frame doesn't surface DisposalMethod in its public API —
        // defaulted to None. See W2 note in crate doc. round-trip:
        // visual stays correct because image-gif's decoder already
        // composites frames against the disposal policy internally
        // (buffer is the composited result, not the raw payload).
        dispose_op: DisposeOp::None,
        blend_op: AnimBlendOp::Source,
        transparent_index: None,
    }
}

impl ImageImporter for GifImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_gif_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("gif") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        use image::AnimationDecoder;
        let cursor = std::io::Cursor::new(src);
        let decoder = image::codecs::gif::GifDecoder::new(cursor)
            .map_err(|e| {
                // Audit-8 Lens L L-3 (2026-05-26): EOF helper.
                Error::from_decoder_message(e.to_string())
            })?;
        let frames: Vec<image::Frame> = decoder.into_frames().collect_frames().map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unexpected end of file") || msg.contains("EOF") {
                Error::Truncated
            } else {
                Error::Decode(msg)
            }
        })?;

        if frames.is_empty() {
            return Err(Error::Decode("GIF has no frames".into()));
        }

        // Dimension guard against the first frame (all frames are sized
        // against the logical screen in GIF, so first is canonical).
        let (w0, h0) = frames[0].buffer().dimensions();
        if w0 == 0 || h0 == 0 {
            return Err(Error::Decode(format!(
                "GIF has zero-sized dimension: {w0}×{h0}"
            )));
        }
        if w0 > MAX_RASTER_DIMENSION || h0 > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        // Audit A-Crit2: refuse decompression-bomb GIFs claiming
        // tens-of-thousands of frames. `collect_frames` already
        // happened (we have the Vec), so this is mostly a guard
        // against insane sources; the real fix is streaming in W2.
        // Audit-8 Lens L L-1 + Lens O O-1 (2026-05-26): use
        // `Error::Decode` not `DimensionExceedsLimit` — frame count is
        // NOT a dimension. Cap via contract-level
        // `MAX_ANIMATION_FRAMES`. Mirrors APNG's MAX_FRAMES rejection.
        if frames.len() > MAX_FRAMES {
            return Err(Error::Decode(format!(
                "GIF claims {} frames (> MAX_ANIMATION_FRAMES={}); refuse to allocate",
                frames.len(),
                ph2d_imageio::MAX_ANIMATION_FRAMES
            )));
        }

        if frames.len() == 1 {
            Ok(DecodedImage::Flat(frame_to_image_buffer(&frames[0])))
        } else {
            let anim: Vec<AnimFrame> = frames.iter().map(frame_to_anim_frame).collect();
            Ok(DecodedImage::Animated(anim))
        }
    }
}

/// GIF export driver.
pub struct GifExporter;

/// Build an `image::Frame` from one of our `AnimFrame`s. Used by both
/// the single-frame `Flat → GIF` path and the multi-frame `Animated →
/// GIF` path.
fn anim_frame_to_image_frame(f: &AnimFrame) -> Result<image::Frame, Error> {
    // Audit D-E4: checked_mul prevents 32-bit silent truncation.
    let byte_len = f
        .image
        .pixels
        .len()
        .checked_mul(4)
        .ok_or(Error::OutOfMemory)?;
    let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
    for p in &f.image.pixels {
        rgba8.extend_from_slice(&p.0);
    }
    let img_buf =
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(f.image.width, f.image.height, rgba8)
            .ok_or_else(|| {
                Error::Encode(format!(
                    "pixel count {} does not match {}×{}",
                    f.image.pixels.len(),
                    f.image.width,
                    f.image.height
                ))
            })?;
    // Delay numer is ms; denom = 1.
    let delay = image::Delay::from_numer_denom_ms(f.delay_ms, 1);
    Ok(image::Frame::from_parts(
        img_buf,
        f.offset_xy.0.max(0) as u32,
        f.offset_xy.1.max(0) as u32,
        delay,
    ))
}

impl ImageExporter for GifExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Gif)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let mut out: Vec<u8> = Vec::new();
        match img {
            DecodedImage::Flat(buf) => {
                // Single-frame: synthesise a 100ms frame so the GIF is
                // valid (GIF doesn't allow zero-delay in spec-strict
                // mode but most decoders accept it; we pick 100ms as
                // neutral).
                let af = AnimFrame {
                    image: buf.clone(),
                    delay_ms: 100,
                    offset_xy: (0, 0),
                    dispose_op: DisposeOp::None,
                    blend_op: AnimBlendOp::Source,
                    transparent_index: None,
                };
                let frame = anim_frame_to_image_frame(&af)?;
                let mut encoder = image::codecs::gif::GifEncoder::new(&mut out);
                encoder
                    .encode_frame(frame)
                    .map_err(|e| Error::Encode(e.to_string()))?;
            }
            DecodedImage::Animated(frames) => {
                if frames.is_empty() {
                    return Err(Error::Encode("animation has no frames".into()));
                }
                let mut encoder = image::codecs::gif::GifEncoder::new(&mut out);
                encoder
                    .set_repeat(image::codecs::gif::Repeat::Infinite)
                    .map_err(|e| Error::Encode(e.to_string()))?;
                for af in frames {
                    let frame = anim_frame_to_image_frame(af)?;
                    encoder
                        .encode_frame(frame)
                        .map_err(|e| Error::Encode(e.to_string()))?;
                }
            }
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Layered(_) => return Err(Error::MissingLayer),
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "GIF is raster; rasterize the VectorDoc first".into(),
                ));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_frame(rgba: [u8; 4]) -> ImageBuffer<SrgbRgba> {
        ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![SrgbRgba(rgba); 4],
            color_profile: ColorProfile::Srgb,
        }
    }

    #[test]
    fn supports_recognizes_both_gif_magics_strong() {
        let imp = GifImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(GIF87_MAGIC)),
            MagicMatch::Strong
        );
        assert_eq!(
            imp.supports(MagicHint::Bytes(GIF89_MAGIC)),
            MagicMatch::Strong
        );
        // GIF + padding ok
        let mut padded = GIF89_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Strong);
        // Not a GIF
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"PNG\0\0\0\0\0\0\0\0\0")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_gif_extension_case_insensitive() {
        let imp = GifImporter;
        for ext in &["gif", "GIF", "Gif"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak,);
        }
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_gif_format() {
        let exp = GifExporter;
        assert!(exp.supports_format(ExportFormat::Gif));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    /// Single-frame GIF round-trip: Flat → encode → decode → Flat.
    /// GIF is paletted with quantisation, so we don't assert pixel
    /// bit-exactness — only structural recovery.
    #[test]
    fn roundtrip_single_frame_recovers_dimensions() {
        let original = fixture_frame([255, 0, 0, 255]);
        let bytes = GifExporter
            .export(
                &DecodedImage::Flat(original),
                &ExportOpts {
                    format: ExportFormat::Gif,
                    ..ExportOpts::default()
                },
            )
            .expect("GIF export should succeed");
        assert!(is_gif_magic(&bytes), "output bytes are not GIF");
        let decoded = GifImporter
            .import(&bytes, &ImportOpts::default())
            .expect("GIF import should succeed");
        // Single-frame source → image-gif may still wrap in Animated
        // with 1 frame depending on how the encoder finalises. Accept
        // either (`Flat` or `Animated` with len==1).
        match decoded {
            DecodedImage::Flat(b) => {
                assert_eq!(b.width, 2);
                assert_eq!(b.height, 2);
            }
            DecodedImage::Animated(frames) => {
                assert_eq!(frames.len(), 1);
                assert_eq!(frames[0].image.width, 2);
                assert_eq!(frames[0].image.height, 2);
            }
            other => panic!("expected Flat or Animated, got {other:?}"),
        }
    }

    /// Multi-frame GIF round-trip: 3 frames → encode → decode → 3
    /// frames with delays preserved.
    #[test]
    fn roundtrip_3_frame_animation_preserves_count_and_delays() {
        let frames = vec![
            AnimFrame {
                image: fixture_frame([255, 0, 0, 255]),
                delay_ms: 100,
                offset_xy: (0, 0),
                dispose_op: DisposeOp::None,
                blend_op: AnimBlendOp::Source,
                transparent_index: None,
            },
            AnimFrame {
                image: fixture_frame([0, 255, 0, 255]),
                delay_ms: 150,
                offset_xy: (0, 0),
                dispose_op: DisposeOp::None,
                blend_op: AnimBlendOp::Source,
                transparent_index: None,
            },
            AnimFrame {
                image: fixture_frame([0, 0, 255, 255]),
                delay_ms: 200,
                offset_xy: (0, 0),
                dispose_op: DisposeOp::None,
                blend_op: AnimBlendOp::Source,
                transparent_index: None,
            },
        ];
        let bytes = GifExporter
            .export(
                &DecodedImage::Animated(frames),
                &ExportOpts {
                    format: ExportFormat::Gif,
                    ..ExportOpts::default()
                },
            )
            .expect("animated GIF export");
        let decoded = GifImporter
            .import(&bytes, &ImportOpts::default())
            .expect("animated GIF import");
        let DecodedImage::Animated(out) = decoded else {
            panic!("expected Animated");
        };
        assert_eq!(out.len(), 3);
        // GIF time units are 1/100 second so 10ms is the minimum
        // resolution — the encoder rounds. We allow ±10ms slack on
        // every delay rather than asserting exact equality.
        let expected_delays = [100u32, 150, 200];
        for (i, (got, want)) in out.iter().zip(expected_delays.iter()).enumerate() {
            assert!(
                got.delay_ms.abs_diff(*want) <= 10,
                "frame {i} delay = {} but expected {} (±10ms)",
                got.delay_ms,
                want
            );
        }
    }

    #[test]
    fn import_rejects_empty_bytes_as_truncated() {
        let err = GifImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty must fail");
        assert!(matches!(err, Error::Truncated));
    }

    /// HR-5 byte-exact determinism (audit D-E2 / agent E-H4): GIF
    /// palette quantisation is deterministic on `image-gif` 0.13 with
    /// our fixed config. If a future upgrade introduces a randomised
    /// dithering seed or threaded palette pass, this test catches.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = fixture_frame([200, 100, 50, 255]);
        let opts = ExportOpts {
            format: ExportFormat::Gif,
            ..ExportOpts::default()
        };
        let bytes_a = GifExporter
            .export(&DecodedImage::Flat(original.clone()), &opts)
            .expect("first GIF export");
        let bytes_b = GifExporter
            .export(&DecodedImage::Flat(original), &opts)
            .expect("second GIF export");
        assert_eq!(
            bytes_a, bytes_b,
            "HR-5: GIF export must be byte-exact deterministic. \
             If this fails, `image-gif` introduced non-deterministic \
             palette/dither state."
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
        let err = GifExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Gif,
                    ..ExportOpts::default()
                },
            )
            .expect_err("GIF must refuse HDR without tone-map");
        assert!(matches!(err, Error::HdrUnsupported));
    }
}

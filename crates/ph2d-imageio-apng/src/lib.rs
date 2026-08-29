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

/// Walk PNG chunks length-prefixed looking for a real `acTL` chunk
/// type tag at chunk-header position (NOT inside `tEXt`/`iTXt`/`zTXt`
/// chunk data). Audit residual C-1 (2026-05-26): the previous
/// `windows(4).any(...)` substring scan false-positives on any PNG
/// whose textual metadata mentions "acTL" — stealing PNG dispatch
/// silently. Real APNGs put `acTL` at the **chunk header tag**
/// position (bytes 4-7 after the chunk length DWORD).
///
/// PNG chunk format: `[length: u32 BE][type: 4 bytes][data][crc: u32]`.
/// First chunk after the 8-byte magic is at offset 8.
///
/// Stops walking after first IDAT (per APNG spec acTL MUST precede
/// IDAT) or after 64 KB / EOF / malformed chunk — whichever comes
/// first. Off-hot (user-click path).
fn has_actl_chunk(b: &[u8]) -> bool {
    // PNG magic (8 bytes) + first chunk header (8 bytes minimum).
    if b.len() < 16 {
        return false;
    }
    let mut offset = 8_usize; // skip PNG magic
    let scan_limit = b.len().min(65_536);
    while offset + 8 <= scan_limit {
        let length =
            u32::from_be_bytes([b[offset], b[offset + 1], b[offset + 2], b[offset + 3]]) as usize;
        let chunk_type = &b[offset + 4..offset + 8];
        if chunk_type == b"acTL" {
            return true;
        }
        if chunk_type == b"IDAT" {
            // APNG spec: acTL MUST precede first IDAT. If we hit
            // IDAT without seeing acTL, it's a plain PNG.
            return false;
        }
        // Advance to next chunk: 4 length + 4 type + length data + 4 CRC.
        let next_offset = offset
            .checked_add(12)
            .and_then(|n| n.checked_add(length))
            .unwrap_or(usize::MAX);
        if next_offset > scan_limit || next_offset <= offset {
            // Overflow, malformed length, or past scan limit.
            return false;
        }
        offset = next_offset;
    }
    false
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
            .map_err(|e| Error::from_decoder_message(format!("APNG read_info: {e}")))?;

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
        let _ = info; // releases the borrow before next_frame

        match animation_control {
            None => {
                // Plain PNG (no acTL).
                let frame = read_one_frame(&mut reader, width, height, post_expand_color)?;
                Ok(DecodedImage::Flat(frame.image))
            }
            Some(act) => {
                // Multi-frame APNG. Audit-7 Lens J CRITICAL J-1 +
                // audit-8 Lens O O-1 (2026-05-26): hostile acTL with
                // `num_frames = u32::MAX` would attempt ~96 GiB
                // allocation (96B per AnimFrame × u32::MAX). Cap via
                // `MAX_ANIMATION_FRAMES` hoisted to the contract
                // (single source of truth shared with GIF).
                if act.num_frames > ph2d_imageio::MAX_ANIMATION_FRAMES {
                    return Err(Error::Decode(format!(
                        "APNG acTL claims {} frames (> MAX_ANIMATION_FRAMES={}); refuse to allocate",
                        act.num_frames,
                        ph2d_imageio::MAX_ANIMATION_FRAMES
                    )));
                }
                let mut anim: Vec<AnimFrame> = Vec::with_capacity(act.num_frames as usize);
                for _ in 0..act.num_frames {
                    let frame = read_one_frame(&mut reader, width, height, post_expand_color)?;
                    anim.push(frame);
                }
                if anim.is_empty() {
                    return Err(Error::Decode("APNG acTL claims 0 frames".into()));
                }
                if anim.len() == 1 {
                    // Audit-7 Lens G F3: use `.into_iter().next()` over
                    // `.pop().expect()` — survives refactor that drops
                    // the `len == 1` guard above.
                    Ok(DecodedImage::Flat(
                        anim.into_iter()
                            .next()
                            .ok_or_else(|| Error::Decode("APNG empty after dedup".into()))?
                            .image,
                    ))
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
        .map_err(|e| Error::from_decoder_message(format!("APNG next_frame: {e}")))?;

    // Per-frame metadata via fcTL (animation control sub-chunks).
    let info = reader.info();
    let fc = info.frame_control().copied();
    let _ = info;

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
                .as_chunks::<4>()
                .0
                .iter()
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
                .as_chunks::<3>()
                .0
                .iter()
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
                .as_chunks::<2>()
                .0
                .iter()
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
mod tests;

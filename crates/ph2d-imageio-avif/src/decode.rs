//! AVIF decode via `libavif-sys` FFI.
//!
//! All `unsafe` is confined here and in [`crate::encode`]. The C
//! decoder + its YUV→RGB conversion run behind RAII guards
//! ([`Decoder`], [`RgbScratch`]) so every libavif allocation is freed
//! on every exit path, and behind `catch_unwind` so a hostile file
//! that trips an `assert!` inside the parser returns `Error::Decode`
//! rather than aborting the process (audit-15 D11 class).

use std::ffi::CStr;
use std::panic::{AssertUnwindSafe, catch_unwind};

use libavif_sys as sys;
use ph2d_color::{LinearRgba, SrgbRgba};
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ImageBuffer, ImageImporter, ImportOpts, MAX_ICC_PROFILE_LEN,
    MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};

use crate::color::{self, DecodePlan, TransferFn};
use crate::is_avif_magic;

/// Maximum total pixels (`width × height`) a decode will accept —
/// decompression-bomb defence complementing the per-axis
/// `MAX_RASTER_DIMENSION`. `16384² = 268_435_456` matches libavif's own
/// default `imageSizeLimit` and the `MAX_RASTER_DIMENSION` doc envelope
/// (a 32768² image — which passes the single-axis cap — is ~16 GiB and
/// is refused here). Kept local to the AVIF crate; hoist to
/// `ph2d_imageio::limits` if another format needs the same cap.
pub(crate) const MAX_TOTAL_PIXELS: u32 = 16_384 * 16_384;

/// AVIF import driver. Real decode (still + first animation frame +
/// composited grid) → `Flat` (SDR 8-bit) or `FlatHdr` (scene-linear).
pub struct AvifImporter;

impl ImageImporter for AvifImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_avif_magic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("avif") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // audit-15 D11: our source is arbitrary user input
        // (drag-and-drop). Isolate the whole FFI decode so a *Rust-level
        // panic* raised across the boundary becomes an error instead of
        // crashing the process. Caveat: `catch_unwind` cannot stop a
        // hard C `abort()`/trap inside dav1d/libavif — that would still
        // terminate; it is an accepted residual risk (libavif 1.0.4
        // returns error codes for the malformed-box paths rather than
        // aborting).
        match catch_unwind(AssertUnwindSafe(|| decode_inner(src))) {
            Ok(r) => r,
            Err(_) => Err(Error::Decode(
                "AVIF decoder panicked on malformed input".into(),
            )),
        }
    }
}

/// Owns an `avifDecoder` and destroys it on drop.
struct Decoder(*mut sys::avifDecoder);

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: `self.0` came from `avifDecoderCreate` and was
        // null-checked at construction; it is destroyed exactly once
        // (this `Drop`), and never aliased elsewhere.
        unsafe { sys::avifDecoderDestroy(self.0) }
    }
}

/// Owns an `avifRGBImage`'s allocated pixel buffer; frees on drop.
struct RgbScratch(sys::avifRGBImage);

impl Drop for RgbScratch {
    fn drop(&mut self) {
        // SAFETY: `self.0` is either zero-initialised (null `pixels`,
        // which `avifRGBImageFreePixels` tolerates) or holds a buffer
        // from `avifRGBImageAllocatePixels`; freed exactly once.
        unsafe { sys::avifRGBImageFreePixels(&mut self.0) }
    }
}

fn decode_inner(src: &[u8]) -> Result<DecodedImage, Error> {
    // SAFETY: `avifDecoderCreate` returns an owned decoder or null.
    let raw = unsafe { sys::avifDecoderCreate() };
    if raw.is_null() {
        return Err(Error::OutOfMemory);
    }
    let decoder = Decoder(raw);

    // HR-13 / audit-15 D14: cap dimensions *inside* the C parser,
    // before any plane allocation. A fraudulent sequence header
    // claiming 100k×100k is refused at parse, not after a giant alloc.
    // SAFETY: `decoder.0` is non-null; these are plain scalar fields.
    unsafe {
        (*decoder.0).imageDimensionLimit = MAX_RASTER_DIMENSION;
        // Single-axis cap is necessary but NOT sufficient: two sub-cap
        // axes still multiply into a bomb (e.g. 32768×8192 ≈ 268 Mpx →
        // ~4 GiB on the 16 B/px HDR path) from a tiny solid-colour file.
        // Also bound TOTAL pixels (libavif's default `imageSizeLimit`
        // is 16384² = 268 Mpx; we keep that ceiling so a 32768×32768
        // claim, which passes the single-axis cap, is refused at parse).
        (*decoder.0).imageSizeLimit = MAX_TOTAL_PIXELS;
        // Single-threaded: avoids spinning up a codec thread pool for
        // editor/cooking decode and keeps behaviour predictable.
        (*decoder.0).maxThreads = 1;
    }

    // `avifDecoderSetIOMemory` references `src` without copying; `src`
    // is borrowed for the whole of `decode_inner`, outliving the parse
    // + decode below.
    // SAFETY: `src` ptr/len are valid; decoder is non-null.
    let res = unsafe { sys::avifDecoderSetIOMemory(decoder.0, src.as_ptr(), src.len()) };
    check(res, "set memory IO")?;

    // SAFETY: decoder is non-null and has IO configured.
    let res = unsafe { sys::avifDecoderParse(decoder.0) };
    check(res, "parse")?;

    // SAFETY: parse succeeded ⇒ `image` points at the decoder-owned
    // `avifImage`. We only read scalar metadata here.
    let image: *const sys::avifImage = unsafe { (*decoder.0).image };
    if image.is_null() {
        return Err(Error::Decode("AVIF parse produced no image".into()));
    }
    // SAFETY: `image` non-null; all reads are plain fields.
    let (width, height, depth, primaries, transfer, image_count) = unsafe {
        (
            (*image).width,
            (*image).height,
            (*image).depth,
            (*image).colorPrimaries as u16,
            (*image).transferCharacteristics as u16,
            (*decoder.0).imageCount,
        )
    };

    if width == 0 || height == 0 {
        return Err(Error::Decode(format!(
            "AVIF zero-sized dimension: {width}×{height}"
        )));
    }
    // Redundant with `imageDimensionLimit` above, but defends against a
    // future libavif that honours the limit differently.
    if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
        return Err(Error::DimensionExceedsLimit);
    }
    // Total-pixel guard (decompression-bomb defence) BEFORE any
    // `Vec::with_capacity(w*h)`: refuse images whose product exceeds the
    // budget even when both axes are individually under the cap. Mirrors
    // the `imageSizeLimit` set on the C decoder above so we're protected
    // even if a future libavif ignores it.
    if u64::from(width) * u64::from(height) > u64::from(MAX_TOTAL_PIXELS) {
        return Err(Error::DimensionExceedsLimit);
    }
    // Animation (`avis`, imageCount > 1): we decode the **first** frame
    // and tag it as a still. The multi-frame `Animated` bridge is W3+;
    // dropping the rest is a documented limitation, not silent — a
    // single-frame still is the only honest LDR/HDR representation the
    // 5-variant `DecodedImage` cap allows here without the Animated
    // sequence wiring. (grid images are NOT animation: libavif stitches
    // a grid into the one composited primary image below.)
    let _ = image_count;

    // Embedded ICC profile. Per the AVIF/MIAF spec an ICC profile, when
    // present, is authoritative over the `nclx` triplet for the SDR
    // path — preserve it byte-exact as `ColorProfile::Custom` so a
    // round-trip (or a downstream ICC-aware pipeline) keeps the
    // user-authored profile. Oversized ICC (decompression-bomb guard,
    // `MAX_ICC_PROFILE_LEN`) is ignored in favour of the nclx mapping
    // rather than erroring — a too-large ICC is suspicious, not fatal.
    // SAFETY: `image` non-null; `icc` is an `avifRWData {data,size}`.
    let icc: Option<Vec<u8>> = unsafe {
        let d = (*image).icc;
        if d.size > 0 && !d.data.is_null() && d.size <= MAX_ICC_PROFILE_LEN {
            Some(std::slice::from_raw_parts(d.data, d.size).to_vec())
        } else {
            None
        }
    };

    // SAFETY: decoder is non-null, parsed; decodes the first frame into
    // the decoder-owned image.
    let res = unsafe { sys::avifDecoderNextImage(decoder.0) };
    check(res, "decode first image")?;

    match color::classify(depth, primaries, transfer) {
        DecodePlan::Ldr8 { profile } => {
            let pixels = extract_rgba8(image, width, height)?;
            debug_assert_eq!(pixels.len(), (width as usize) * (height as usize));
            // ICC (if any) overrides the nclx-derived profile for SDR.
            let color_profile = icc.map_or(profile, ColorProfile::Custom);
            Ok(DecodedImage::Flat(ImageBuffer {
                width,
                height,
                pixels,
                color_profile,
            }))
        }
        DecodePlan::HdrLinear { profile, transfer } => {
            let pixels = extract_rgba_hdr(image, width, height, transfer)?;
            debug_assert_eq!(pixels.len(), (width as usize) * (height as usize));
            Ok(DecodedImage::FlatHdr(ImageBuffer {
                width,
                height,
                pixels,
                color_profile: profile,
            }))
        }
    }
}

/// Build an `avifRGBImage` (RGBA, given depth/float) from `image`,
/// allocate its pixels, and run the YUV→RGB conversion. The returned
/// guard owns the buffer; read `.0.pixels` / `.0.rowBytes` before it
/// drops.
fn convert_to_rgb(
    image: *const sys::avifImage,
    depth: u32,
    is_float: bool,
) -> Result<RgbScratch, Error> {
    // SAFETY: `avifRGBImage` is plain-old-data (scalars + raw
    // pointers); zero-init is a valid pre-state that `SetDefaults`
    // immediately overwrites.
    let mut rgb: sys::avifRGBImage = unsafe { std::mem::zeroed() };
    // SAFETY: `image` is a valid decoder-owned image; SetDefaults fills
    // width/height/depth/format from it.
    unsafe { sys::avifRGBImageSetDefaults(&mut rgb, image) };
    rgb.format = sys::AVIF_RGB_FORMAT_RGBA;
    rgb.depth = depth;
    rgb.isFloat = sys::avifBool::from(is_float);
    rgb.ignoreAlpha = 0;
    // Request straight (non-premultiplied) alpha output. The engine's
    // `SrgbRgba`/`LinearRgba` are straight-alpha; if the source AVIF
    // stored premultiplied alpha, libavif unpremultiplies for us here.
    rgb.alphaPremultiplied = 0;

    // SAFETY: `rgb` is fully initialised; allocates `width*height*…`.
    let res = unsafe { sys::avifRGBImageAllocatePixels(&mut rgb) };
    // Move into the guard immediately so the buffer is freed even if
    // the conversion below fails or returns early.
    let mut scratch = RgbScratch(rgb);
    check(res, "allocate RGB pixels")?;

    // SAFETY: both images are valid; `scratch.0` has an allocated
    // buffer of the requested geometry.
    let res = unsafe { sys::avifImageYUVToRGB(image, &mut scratch.0) };
    check(res, "YUV→RGB")?;
    Ok(scratch)
}

/// Verify the converted `avifRGBImage`'s geometry matches what the
/// extraction loop assumes: same dimensions, and rows at least
/// `width * bytes_per_pixel` wide (no negative padding). Returns an
/// error rather than risking out-of-bounds reads if a future libavif
/// reports a surprising `rowBytes`/`width`/`height`.
fn check_rgb_geometry(
    rgb: &sys::avifRGBImage,
    w: usize,
    h: usize,
    bytes_per_pixel: usize,
) -> Result<(), Error> {
    if rgb.width as usize != w || rgb.height as usize != h {
        return Err(Error::Decode(format!(
            "AVIF RGB geometry mismatch: image {w}×{h} vs rgb {}×{}",
            rgb.width, rgb.height
        )));
    }
    if (rgb.rowBytes as usize) < w * bytes_per_pixel {
        return Err(Error::Decode(format!(
            "AVIF RGB rowBytes {} < {} (width {w} × {bytes_per_pixel})",
            rgb.rowBytes,
            w * bytes_per_pixel
        )));
    }
    Ok(())
}

/// Extract an 8-bit RGBA buffer (SDR path) as `SrgbRgba` (bytes are
/// already sRGB-encoded — no transfer math needed).
fn extract_rgba8(
    image: *const sys::avifImage,
    width: u32,
    height: u32,
) -> Result<Vec<SrgbRgba>, Error> {
    let scratch = convert_to_rgb(image, 8, false)?;
    let rgb = &scratch.0;
    let row_bytes = rgb.rowBytes as usize;
    let w = width as usize;
    let h = height as usize;
    // Verify libavif's reported geometry matches what we allocate the
    // loop against (the encode path has the same guard). Defends the
    // pointer arithmetic below against a libavif that pads rows or
    // disagrees on dimensions — an explicit error beats reading OOB.
    check_rgb_geometry(rgb, w, h, 4)?;
    let mut pixels = Vec::with_capacity(w * h);
    // SAFETY: `rgb.pixels` is an allocated `h * row_bytes` buffer of
    // u8; we read 4 bytes per pixel within each row's valid span.
    let base = rgb.pixels;
    if base.is_null() {
        return Err(Error::Decode("AVIF RGB buffer is null".into()));
    }
    for y in 0..h {
        for x in 0..w {
            let off = y * row_bytes + x * 4;
            // SAFETY: `off + 3 < h*row_bytes`; row_bytes ≥ w*4.
            let px = unsafe { base.add(off) };
            let r = unsafe { *px };
            let g = unsafe { *px.add(1) };
            let b = unsafe { *px.add(2) };
            let a = unsafe { *px.add(3) };
            pixels.push(SrgbRgba([r, g, b, a]));
        }
    }
    Ok(pixels)
}

/// Extract a scene-linear RGBA buffer (HDR/wide-gamut path). Decodes to
/// 16-bit integer RGBA, normalises each channel to `[0,1]`, then
/// applies the inverse transfer function to RGB (alpha stays linear).
fn extract_rgba_hdr(
    image: *const sys::avifImage,
    width: u32,
    height: u32,
    transfer: TransferFn,
) -> Result<Vec<LinearRgba>, Error> {
    let scratch = convert_to_rgb(image, 16, false)?;
    let rgb = &scratch.0;
    let row_bytes = rgb.rowBytes as usize;
    let w = width as usize;
    let h = height as usize;
    check_rgb_geometry(rgb, w, h, 8)?;
    let mut pixels = Vec::with_capacity(w * h);
    let base = rgb.pixels;
    if base.is_null() {
        return Err(Error::Decode("AVIF RGB buffer is null".into()));
    }
    const MAX16: f32 = 65535.0;
    for y in 0..h {
        for x in 0..w {
            let off = y * row_bytes + x * 8; // 4 channels × u16
            // SAFETY: `off + 7 < h*row_bytes` (geometry checked above).
            // `read_unaligned` so we don't rely on `base` being u16-
            // aligned (it is in practice — `avifAlloc` is malloc-class —
            // but the cast shouldn't assume it). Samples are host-endian.
            let p = unsafe { base.add(off) };
            let r = f32::from(unsafe { p.cast::<u16>().read_unaligned() }) / MAX16;
            let g = f32::from(unsafe { p.add(2).cast::<u16>().read_unaligned() }) / MAX16;
            let b = f32::from(unsafe { p.add(4).cast::<u16>().read_unaligned() }) / MAX16;
            let a = f32::from(unsafe { p.add(6).cast::<u16>().read_unaligned() }) / MAX16;
            pixels.push(LinearRgba::new(
                color::eotf(r, transfer),
                color::eotf(g, transfer),
                color::eotf(b, transfer),
                a,
            ));
        }
    }
    Ok(pixels)
}

/// `Ok` if `res == AVIF_RESULT_OK`, else a semantic [`Error`].
fn check(res: sys::avifResult, ctx: &str) -> Result<(), Error> {
    if res == sys::AVIF_RESULT_OK {
        Ok(())
    } else {
        Err(decode_err(res, ctx))
    }
}

fn decode_err(res: sys::avifResult, ctx: &str) -> Error {
    match res {
        sys::AVIF_RESULT_OUT_OF_MEMORY => Error::OutOfMemory,
        sys::AVIF_RESULT_TRUNCATED_DATA => Error::Truncated,
        _ => Error::from_decoder_message(format!("AVIF {ctx}: {}", result_string(res))),
    }
}

pub(crate) fn result_string(res: sys::avifResult) -> String {
    // SAFETY: `avifResultToString` returns a pointer to a static,
    // NUL-terminated C string for every result code (never null).
    let cstr = unsafe { sys::avifResultToString(res) };
    if cstr.is_null() {
        return format!("result code {res}");
    }
    // SAFETY: `cstr` is a valid NUL-terminated C string (static
    // lifetime inside libavif); we copy it out before returning.
    unsafe { CStr::from_ptr(cstr) }
        .to_string_lossy()
        .into_owned()
}

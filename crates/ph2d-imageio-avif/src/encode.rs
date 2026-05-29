//! AVIF encode via `libavif-sys` + the pure-Rust rav1e codec.
//!
//! All `unsafe` is confined here and in [`crate::decode`], behind RAII
//! guards ([`Encoder`], [`Image`], [`RwData`]) that free every libavif
//! allocation on every exit path, and behind `catch_unwind`.
//!
//! Pixel format is **YUV444** (no chroma subsampling — best fidelity
//! for an editor/cooking exporter). At `quality == 100` the encoder is
//! driven lossless with the **identity** matrix (RGB stored directly,
//! exact 8-bit round-trip); below 100 it uses the BT.709 / BT.2020
//! matrix for the requested colour space. `nclx` (primaries +
//! transfer) is written from the source [`ColorProfile`] so HDR /
//! wide-gamut content round-trips.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::slice;

use libavif_sys as sys;
use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ImageExporter, MAX_RASTER_DIMENSION,
};

use crate::color::{self, TransferFn};
use crate::decode::MAX_TOTAL_PIXELS;

/// Reject oversized source images before any allocation (mirrors the
/// decode-side caps): a `DecodedImage` produced by some other importer
/// could be larger than AVIF/our budget. Also the gate that makes the
/// `width*4*bps`/`*height` size arithmetic below provably non-overflowing.
fn check_encode_dims(width: u32, height: u32) -> Result<(), Error> {
    if width == 0 || height == 0 {
        return Err(Error::Encode("cannot encode a zero-sized image".into()));
    }
    if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
        return Err(Error::DimensionExceedsLimit);
    }
    if u64::from(width) * u64::from(height) > u64::from(MAX_TOTAL_PIXELS) {
        return Err(Error::DimensionExceedsLimit);
    }
    Ok(())
}

/// AVIF export driver. `Flat` (SDR) / `FlatHdr` (HDR) → AVIF.
pub struct AvifExporter;

impl ImageExporter for AvifExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Avif)
    }

    fn export(&self, img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        match catch_unwind(AssertUnwindSafe(|| encode_inner(img, opts))) {
            Ok(r) => r,
            Err(_) => Err(Error::Encode(
                "AVIF encoder panicked (unexpected input)".into(),
            )),
        }
    }
}

/// Plan for the encode: pixel bit depth + nclx triplet. (The transfer
/// *function* used to linearise/encode pixels is applied inside the
/// per-variant `fill` closure; this struct carries only the CICP codes
/// written into the file's `nclx` box.)
struct EncodePlan {
    depth: u32,
    primaries: u16,
    transfer_cicp: u16,
    matrix: u16,
}

/// Writes interleaved RGBA samples (u8 for SDR, host-endian u16 for
/// HDR) into the allocated `avifRGBImage` byte buffer.
type FillFn = Box<dyn Fn(&mut [u8])>;

/// Everything `encode_rgba` needs, resolved per `DecodedImage` variant.
struct EncodeJob {
    width: u32,
    height: u32,
    plan: EncodePlan,
    fill: FillFn,
}

fn encode_inner(img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error> {
    // Quality 0..=100; ≥100 ⇒ lossless (identity matrix, exact RGB).
    let quality = i32::from(opts.quality.min(100));
    let lossless = quality >= 100;

    // Pull geometry + build the per-pixel RGB sample writer up front so
    // the match on `DecodedImage` is the only place that knows the
    // variant. `fill` writes interleaved RGBA samples (u8 for SDR,
    // host-endian u16 for HDR) into the allocated `avifRGBImage`.
    let job: EncodeJob = match img {
        DecodedImage::Flat(buf) => {
            if !buf.is_well_formed() {
                return Err(Error::Encode("source buffer is malformed".into()));
            }
            check_encode_dims(buf.width, buf.height)?;
            let (primaries, transfer_cicp, mut matrix) =
                color::nclx_for_encode(&buf.color_profile, TransferFn::Gamma);
            if lossless {
                matrix = 0; // identity → exact RGB
            }
            let plan = EncodePlan {
                depth: 8,
                primaries,
                transfer_cicp,
                matrix,
            };
            let pixels = buf.pixels.clone();
            let w = buf.width;
            let h = buf.height;
            let fill: FillFn = Box::new(move |dst: &mut [u8]| {
                for (i, px) in pixels.iter().enumerate() {
                    let b = px.as_bytes();
                    dst[i * 4..i * 4 + 4].copy_from_slice(b);
                }
            });
            EncodeJob {
                width: w,
                height: h,
                plan,
                fill,
            }
        }
        DecodedImage::FlatHdr(buf) => {
            if !buf.is_well_formed() {
                return Err(Error::Encode("source buffer is malformed".into()));
            }
            check_encode_dims(buf.width, buf.height)?;
            // HDR is encoded as 10-bit PQ (HDR10) — the AVIF-canonical
            // HDR profile. Primaries follow the buffer's profile
            // (Rec.2020 ⇒ wide gamut, else Rec.709).
            let (primaries, transfer_cicp, mut matrix) =
                color::nclx_for_encode(&buf.color_profile, TransferFn::Pq);
            if lossless {
                matrix = 0;
            }
            let plan = EncodePlan {
                depth: 10,
                primaries,
                transfer_cicp,
                matrix,
            };
            let pixels = buf.pixels.clone();
            let w = buf.width;
            let h = buf.height;
            let fill: FillFn = Box::new(move |dst: &mut [u8]| {
                // Write host-endian u16 samples directly as bytes — no
                // `&mut [u16]` cast, so no alignment assumption on the
                // libavif-owned buffer (matches the decode-side
                // `read_unaligned`). 8 bytes/pixel (4 × u16).
                for (i, px) in pixels.iter().enumerate() {
                    let enc = |lin: f32| -> u16 {
                        let s = color::oetf(lin, TransferFn::Pq);
                        (s * 65535.0 + 0.5) as u16
                    };
                    let off = i * 8;
                    dst[off..off + 2].copy_from_slice(&enc(px.r()).to_ne_bytes());
                    dst[off + 2..off + 4].copy_from_slice(&enc(px.g()).to_ne_bytes());
                    dst[off + 4..off + 6].copy_from_slice(&enc(px.b()).to_ne_bytes());
                    // Alpha is linear coverage, not transfer-encoded.
                    let a = (px.a().clamp(0.0, 1.0) * 65535.0 + 0.5) as u16;
                    dst[off + 6..off + 8].copy_from_slice(&a.to_ne_bytes());
                }
            });
            EncodeJob {
                width: w,
                height: h,
                plan,
                fill,
            }
        }
        DecodedImage::Layered(_) => {
            return Err(Error::Unsupported(
                "AVIF is a single composited image; flatten the layer stack first \
                 (or save .ph2d-native to preserve layers)."
                    .into(),
            ));
        }
        DecodedImage::Animated(_) => {
            return Err(Error::Unsupported(
                "AVIF animation (avis) encode is W3+; export a single frame for now.".into(),
            ));
        }
        DecodedImage::Vector(_) => {
            return Err(Error::Unsupported(
                "AVIF is raster; rasterize the VectorDoc before AVIF export.".into(),
            ));
        }
    };

    encode_rgba(
        job.width, job.height, &job.plan, quality, lossless, &*job.fill,
    )
}

/// Owns an `avifEncoder`.
struct Encoder(*mut sys::avifEncoder);
impl Drop for Encoder {
    fn drop(&mut self) {
        // SAFETY: from `avifEncoderCreate`, null-checked, destroyed once.
        unsafe { sys::avifEncoderDestroy(self.0) }
    }
}

/// Owns an `avifImage`.
struct Image(*mut sys::avifImage);
impl Drop for Image {
    fn drop(&mut self) {
        // SAFETY: from `avifImageCreate`, null-checked, destroyed once.
        unsafe { sys::avifImageDestroy(self.0) }
    }
}

/// Owns an `avifRWData` output buffer.
struct RwData(sys::avifRWData);
impl Drop for RwData {
    fn drop(&mut self) {
        // SAFETY: `avifRWDataFree` tolerates a zeroed (null) buffer and
        // frees an `avifEncoderWrite`-populated one exactly once.
        unsafe { sys::avifRWDataFree(&mut self.0) }
    }
}

/// Owns the encode-side `avifRGBImage` pixel buffer.
struct RgbScratch(sys::avifRGBImage);
impl Drop for RgbScratch {
    fn drop(&mut self) {
        // SAFETY: zeroed (null) or `avifRGBImageAllocatePixels`-owned.
        unsafe { sys::avifRGBImageFreePixels(&mut self.0) }
    }
}

fn encode_rgba(
    width: u32,
    height: u32,
    plan: &EncodePlan,
    quality: i32,
    lossless: bool,
    fill: &dyn Fn(&mut [u8]),
) -> Result<Vec<u8>, Error> {
    // SAFETY: create an owned YUV444 image at the planned depth.
    let raw_img =
        unsafe { sys::avifImageCreate(width, height, plan.depth, sys::AVIF_PIXEL_FORMAT_YUV444) };
    if raw_img.is_null() {
        return Err(Error::OutOfMemory);
    }
    let image = Image(raw_img);
    // SAFETY: image non-null; write nclx + range scalar fields.
    unsafe {
        (*image.0).colorPrimaries = plan.primaries;
        (*image.0).transferCharacteristics = plan.transfer_cicp;
        (*image.0).matrixCoefficients = plan.matrix;
        // Full-range so the identity matrix (lossless) stores RGB
        // without a limited-range squeeze.
        (*image.0).yuvRange = sys::AVIF_RANGE_FULL;
    }

    // Build the RGB source view and fill it from our pixels.
    // SAFETY: POD; SetDefaults overwrites from `image`.
    let mut rgb: sys::avifRGBImage = unsafe { std::mem::zeroed() };
    // SAFETY: image valid.
    unsafe { sys::avifRGBImageSetDefaults(&mut rgb, image.0) };
    rgb.format = sys::AVIF_RGB_FORMAT_RGBA;
    rgb.depth = if plan.depth > 8 { 16 } else { 8 };
    rgb.isFloat = 0;
    rgb.ignoreAlpha = 0;
    // SAFETY: rgb fully initialised; allocate pixel storage.
    let res = unsafe { sys::avifRGBImageAllocatePixels(&mut rgb) };
    let scratch = RgbScratch(rgb);
    check_enc(res, "allocate RGB pixels")?;
    if scratch.0.pixels.is_null() {
        return Err(Error::OutOfMemory);
    }
    // Fill the RGB buffer. `fill` indexes linearly, which is only
    // valid if libavif allocated `rowBytes == width * channels *
    // bytesPerSample` (no per-row padding). It does today; guard so a
    // future libavif that pads can't cause `fill` to write misaligned
    // rows or read OOB — better an explicit error than UB.
    let bytes_per_sample = if scratch.0.depth > 8 { 2 } else { 1 };
    let tight_row = (width as usize) * 4 * bytes_per_sample;
    if scratch.0.rowBytes as usize != tight_row {
        return Err(Error::Encode(format!(
            "AVIF RGB buffer has padded rows ({} != {tight_row}); unsupported",
            scratch.0.rowBytes
        )));
    }
    let total = tight_row * (height as usize);
    // SAFETY: `pixels` points at `height * rowBytes == total` bytes,
    // verified tightly-packed just above.
    let dst = unsafe { slice::from_raw_parts_mut(scratch.0.pixels, total) };
    fill(dst);

    // SAFETY: both valid; converts the RGB buffer into the image YUV.
    let res = unsafe { sys::avifImageRGBToYUV(image.0, &scratch.0) };
    check_enc(res, "RGB→YUV")?;

    // SAFETY: create an owned encoder.
    let raw_enc = unsafe { sys::avifEncoderCreate() };
    if raw_enc.is_null() {
        return Err(Error::OutOfMemory);
    }
    let encoder = Encoder(raw_enc);
    // SAFETY: encoder non-null; scalar fields.
    unsafe {
        (*encoder.0).quality = quality;
        (*encoder.0).qualityAlpha = quality;
        (*encoder.0).maxThreads = 1;
        // Speed 6 = a sane editor default (rav1e 0=slowest..10=fastest).
        (*encoder.0).speed = 6;
        if lossless {
            // Belt-and-braces with the identity matrix set above.
            (*encoder.0).minQuantizer = 0;
            (*encoder.0).maxQuantizer = 0;
            (*encoder.0).minQuantizerAlpha = 0;
            (*encoder.0).maxQuantizerAlpha = 0;
        }
    }

    let mut output = RwData(unsafe { std::mem::zeroed() });
    // SAFETY: encoder + image valid; output is a zeroed avifRWData that
    // the call populates (freed by `RwData`'s Drop).
    let res = unsafe { sys::avifEncoderWrite(encoder.0, image.0, &mut output.0) };
    check_enc(res, "encode")?;

    if output.0.data.is_null() || output.0.size == 0 {
        return Err(Error::Encode("AVIF encoder produced no data".into()));
    }
    // SAFETY: `data` points at `size` valid bytes owned by `output`.
    let bytes = unsafe { slice::from_raw_parts(output.0.data, output.0.size) }.to_vec();
    Ok(bytes)
}

fn check_enc(res: sys::avifResult, ctx: &str) -> Result<(), Error> {
    if res == sys::AVIF_RESULT_OK {
        Ok(())
    } else if res == sys::AVIF_RESULT_OUT_OF_MEMORY {
        Err(Error::OutOfMemory)
    } else {
        Err(Error::Encode(format!(
            "AVIF {ctx}: {}",
            crate::decode::result_string(res)
        )))
    }
}

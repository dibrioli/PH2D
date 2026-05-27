//! Internal: byte → [`Asset`] decoders.
//!
//! Lives behind a private module so we can swap `image` for a
//! lower-level codec (e.g. `png` directly) later without breaking
//! [`AssetDb`] callers. M6 ships PNG only.
//!
//! All decode paths enforce [`MAX_DIMENSION`] / [`MAX_ALLOC_BYTES`]
//! limits via `image::Limits` — protects the watcher thread from a
//! "PNG bomb" (a tiny file declaring 100k × 100k pixels would
//! attempt a ~40 GB allocation during decode).

use crate::asset::Asset;
use crate::error::AssetError;
use image::ImageDecoder;
use ph2d_imageio::{DecodedImage, ImportOpts, ImporterRegistry, MagicHint};
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, OnceLock};

/// Hard ceiling on decoded image dimensions (per side). 8k covers
/// every realistic 2D sprite atlas; anything larger is almost
/// certainly malicious or accidental.
pub(crate) const MAX_DIMENSION: u32 = 8192;

/// Lazy, process-global importer registry populated by
/// [`ph2d_imageio_registry_init::register_all_importers`]. Used by
/// [`decode_image_bytes`] when `image::guess_format` returns a
/// format outside the legacy PNG/WEBP/JPEG fast path.
///
/// ADR-0054 wire-up (2026-05-27): closes the X-HIGH-1 architectural
/// gap flagged by audits 7-12 — `AssetDb::insert_image_bytes` used
/// to bypass the imageio registry entirely, so the audited OOM/
/// NaN/recursion/path-traversal defences applied to TIFF/ORA/APNG/
/// PSD/GIF/.ph2d-native were dead in prod. This registry is the
/// single dispatch point that brings those defences onto the user-
/// import hot path.
fn imageio_registry() -> &'static ImporterRegistry {
    static REG: OnceLock<ImporterRegistry> = OnceLock::new();
    REG.get_or_init(|| {
        let mut reg = ImporterRegistry::default();
        ph2d_imageio_registry_init::register_all_importers(&mut reg);
        reg
    })
}

/// Hard ceiling on bytes the decoder may allocate for a single
/// image. 8k × 8k × 4 = 256 MiB; round to 512 MiB so the limit
/// triggers on declared dimensions, not legitimate worst-case
/// internal scratch.
pub(crate) const MAX_ALLOC_BYTES: u64 = 512 * 1024 * 1024;

fn limits() -> image::Limits {
    let mut l = image::Limits::default();
    l.max_image_width = Some(MAX_DIMENSION);
    l.max_image_height = Some(MAX_DIMENSION);
    l.max_alloc = Some(MAX_ALLOC_BYTES);
    l
}

/// Decode an in-memory PNG buffer to [`Asset::ImageRgba8`].
/// `path_hint` is used only for error messages.
pub(crate) fn decode_png_bytes(
    bytes: &[u8],
    path_hint: Option<&Path>,
) -> Result<Asset, AssetError> {
    let cursor = Cursor::new(bytes);
    let mut decoder =
        image::codecs::png::PngDecoder::new(cursor).map_err(|e| AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: e.to_string(),
        })?;
    decoder
        .set_limits(limits())
        .map_err(|e| AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!("declared dimensions exceed limit: {e}"),
        })?;
    let img = image::DynamicImage::from_decoder(decoder).map_err(|e| AssetError::Decode {
        path: path_hint.map(Path::to_path_buf),
        message: e.to_string(),
    })?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Arc<[u8]> = Arc::from(rgba.into_raw().into_boxed_slice());
    Ok(Asset::ImageRgba8 {
        width,
        height,
        pixels,
    })
}

/// True if `path` has a PNG extension (case-insensitive). Filters
/// `.png`, `.PNG`, `.Png`, etc. — the watcher must not be tricked by
/// case variation.
pub(crate) fn is_png_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("png"))
}

/// True if `path` has a supported image extension. Used by the
/// import filter and the M14.4d drag-and-drop filter to accept user-
/// supplied sprites without reaching for the actual bytes.
///
/// ADR-0054 W3.T1.0 (2026-05-27): set expanded from {PNG, WEBP,
/// JPEG, JPG} to also include {GIF, TIFF, TIF, ORA, APNG, PSD,
/// PH2D} since the imageio registry wire-up now decodes them.
/// Multi-layer (ORA/PSD) and multi-frame (GIF/APNG) imports still
/// reject with an actionable message at decode time (see
/// `decode_via_imageio_registry`); the extension filter is a UX-
/// only gate (file-dialog filter doesn't pre-decode).
pub fn is_supported_image_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref(),
        Some(
            "png"
                | "webp"
                | "jpg"
                | "jpeg"
                | "gif"
                | "tiff"
                | "tif"
                | "ora"
                | "apng"
                | "psd"
                | "ph2d"
        )
    )
}

/// Decode an in-memory image buffer (PNG / WEBP / JPEG) to
/// [`Asset::ImageRgba8`]. Format is auto-detected by `image::guess_format`.
/// `path_hint` only annotates error messages.
///
/// HR-6 friendly: the returned `Asset` is content-only — the
/// caller's blake3 over the input bytes is the canonical `AssetId`.
pub(crate) fn decode_image_bytes(
    bytes: &[u8],
    path_hint: Option<&Path>,
) -> Result<Asset, AssetError> {
    let format = image::guess_format(bytes).map_err(|e| AssetError::Decode {
        path: path_hint.map(Path::to_path_buf),
        message: format!("guess_format: {e}"),
    })?;
    let cursor = Cursor::new(bytes);
    let img = match format {
        image::ImageFormat::Png => {
            let mut decoder =
                image::codecs::png::PngDecoder::new(cursor).map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: e.to_string(),
                })?;
            decoder
                .set_limits(limits())
                .map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: format!("declared dimensions exceed limit: {e}"),
                })?;
            image::DynamicImage::from_decoder(decoder).map_err(|e| AssetError::Decode {
                path: path_hint.map(Path::to_path_buf),
                message: e.to_string(),
            })?
        }
        image::ImageFormat::WebP => {
            let mut decoder =
                image::codecs::webp::WebPDecoder::new(cursor).map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: e.to_string(),
                })?;
            decoder
                .set_limits(limits())
                .map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: format!("declared dimensions exceed limit: {e}"),
                })?;
            image::DynamicImage::from_decoder(decoder).map_err(|e| AssetError::Decode {
                path: path_hint.map(Path::to_path_buf),
                message: e.to_string(),
            })?
        }
        image::ImageFormat::Jpeg => {
            let mut decoder =
                image::codecs::jpeg::JpegDecoder::new(cursor).map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: e.to_string(),
                })?;
            decoder
                .set_limits(limits())
                .map_err(|e| AssetError::Decode {
                    path: path_hint.map(Path::to_path_buf),
                    message: format!("declared dimensions exceed limit: {e}"),
                })?;
            image::DynamicImage::from_decoder(decoder).map_err(|e| AssetError::Decode {
                path: path_hint.map(Path::to_path_buf),
                message: e.to_string(),
            })?
        }
        other => {
            // ADR-0054 W3.T1.0 wire-up (2026-05-27): fallback to the
            // imageio registry for formats the legacy PNG/WEBP/JPEG
            // path doesn't cover (GIF, TIFF, ORA, APNG, PSD,
            // .ph2d-native). This is the single dispatch point that
            // brings the audited OOM caps, NaN reject, path-
            // traversal reject, recursion bomb caps, and EOF
            // semantics onto the user-import hot path. Closes
            // X-HIGH-1 from audits 7-12.
            return decode_via_imageio_registry(bytes, path_hint, other);
        }
    };
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: Arc<[u8]> = Arc::from(rgba.into_raw().into_boxed_slice());
    Ok(Asset::ImageRgba8 {
        width,
        height,
        pixels,
    })
}

/// ADR-0054 W3.T1.0 wire-up (2026-05-27): dispatch a non-PNG/WEBP/
/// JPEG image to the imageio registry and convert the result back to
/// [`Asset::ImageRgba8`].
///
/// Only [`DecodedImage::Flat`] flows through `AssetDb` today — sprite
/// renderer wants a single RGBA8 buffer. Layered (ORA/PSD) and
/// Animated (GIF/APNG multi-frame) are surfaced via dedicated panels
/// in W3+ (asset bridge amendment when first real client appears).
/// Until then they're rejected with an actionable message.
fn decode_via_imageio_registry(
    bytes: &[u8],
    path_hint: Option<&Path>,
    detected_format: image::ImageFormat,
) -> Result<Asset, AssetError> {
    let reg = imageio_registry();
    // Try magic-byte dispatch first (Strong > Weak).
    let importer = reg
        .find_for(MagicHint::Bytes(bytes))
        .ok_or_else(|| AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!(
                "no imageio importer registered for detected format {detected_format:?} \
                 (try `.ph2d` or one of: PNG, WEBP, JPEG, GIF, TIFF, ORA, APNG, PSD)"
            ),
        })?;
    let decoded = importer
        .import(bytes, &ImportOpts::default())
        .map_err(|e| AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!("imageio {detected_format:?}: {e}"),
        })?;
    match decoded {
        DecodedImage::Flat(buf) => {
            let width = buf.width;
            let height = buf.height;
            // Flatten Vec<SrgbRgba> to Vec<u8> for atlas packer
            // compat. SrgbRgba is `#[repr(C)] [u8; 4]` so bytemuck-
            // safe transmute would also work; the copy here is
            // ~4×width×height which is bounded by MAX_RASTER_DIMENSION.
            let mut rgba_bytes: Vec<u8> =
                Vec::with_capacity((width as usize) * (height as usize) * 4);
            for px in &buf.pixels {
                rgba_bytes.extend_from_slice(&px.0);
            }
            Ok(Asset::ImageRgba8 {
                width,
                height,
                pixels: Arc::from(rgba_bytes.into_boxed_slice()),
            })
        }
        DecodedImage::Layered(_) => Err(AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!(
                "{detected_format:?} decoded to Layered; \
                 multi-layer documents not yet bridged to AssetDb \
                 (W3+ — use `.ph2d-native` flatten or open in Painter)"
            ),
        }),
        DecodedImage::Animated(_) => Err(AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!(
                "{detected_format:?} decoded to Animated; \
                 multi-frame imports not yet bridged to AssetDb \
                 (W3+ — sprite sheet generation lands when first \
                 real animation client appears)"
            ),
        }),
        DecodedImage::FlatHdr(_) => Err(AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!(
                "{detected_format:?} decoded to FlatHdr; HDR import \
                 not yet bridged to AssetDb (W3.T3 EXR + W3.T4 HDR-\
                 Radiance + tone-map land here)"
            ),
        }),
        DecodedImage::Vector(_) => Err(AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: format!(
                "{detected_format:?} decoded to Vector; vector \
                 preservation not yet bridged to AssetDb (W3.T5 SVG)"
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive_png_filter() {
        assert!(is_png_extension(Path::new("a.png")));
        assert!(is_png_extension(Path::new("a.PNG")));
        assert!(is_png_extension(Path::new("a.Png")));
        assert!(is_png_extension(Path::new("a.pNg")));
        assert!(!is_png_extension(Path::new("a.jpg")));
        assert!(!is_png_extension(Path::new("a")));
        assert!(!is_png_extension(Path::new("a.png.bak")));
    }

    #[test]
    fn supported_image_filter_covers_png_webp_jpeg_and_imageio_formats() {
        // Legacy fast-path formats (image 0.25 direct).
        assert!(is_supported_image_extension(Path::new("a.png")));
        assert!(is_supported_image_extension(Path::new("a.PNG")));
        assert!(is_supported_image_extension(Path::new("a.webp")));
        assert!(is_supported_image_extension(Path::new("a.WEBP")));
        assert!(is_supported_image_extension(Path::new("a.jpg")));
        assert!(is_supported_image_extension(Path::new("a.JPG")));
        assert!(is_supported_image_extension(Path::new("a.jpeg")));
        // ADR-0054 W3.T1.0 wire-up: imageio registry now decodes
        // these too. Multi-layer (ORA/PSD) and multi-frame (GIF/
        // APNG) reject at decode-time with actionable message; the
        // extension filter is UX-only (file-dialog filter).
        assert!(is_supported_image_extension(Path::new("a.gif")));
        assert!(is_supported_image_extension(Path::new("a.GIF")));
        assert!(is_supported_image_extension(Path::new("a.tiff")));
        assert!(is_supported_image_extension(Path::new("a.tif")));
        assert!(is_supported_image_extension(Path::new("a.TIFF")));
        assert!(is_supported_image_extension(Path::new("a.ora")));
        assert!(is_supported_image_extension(Path::new("a.ORA")));
        assert!(is_supported_image_extension(Path::new("a.apng")));
        assert!(is_supported_image_extension(Path::new("a.psd")));
        assert!(is_supported_image_extension(Path::new("a.PSD")));
        assert!(is_supported_image_extension(Path::new("a.ph2d")));
        // Non-image extensions still rejected.
        assert!(!is_supported_image_extension(Path::new("a.txt")));
        assert!(!is_supported_image_extension(Path::new("a.bmp"))); // future W3+
        assert!(!is_supported_image_extension(Path::new("a")));
    }
}

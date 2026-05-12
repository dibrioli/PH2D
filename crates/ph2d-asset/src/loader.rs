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
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

/// Hard ceiling on decoded image dimensions (per side). 8k covers
/// every realistic 2D sprite atlas; anything larger is almost
/// certainly malicious or accidental.
pub(crate) const MAX_DIMENSION: u32 = 8192;

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

/// True if `path` has a supported image extension (PNG / WEBP / JPEG /
/// JPG, case-insensitive). Used by the import filter and the M14.4d
/// drag-and-drop filter to accept user-supplied sprites without
/// reaching for the actual bytes.
pub fn is_supported_image_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "webp" | "jpg" | "jpeg")
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
            return Err(AssetError::Decode {
                path: path_hint.map(Path::to_path_buf),
                message: format!("unsupported image format: {other:?}"),
            });
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
    fn supported_image_filter_covers_png_webp_jpeg() {
        assert!(is_supported_image_extension(Path::new("a.png")));
        assert!(is_supported_image_extension(Path::new("a.PNG")));
        assert!(is_supported_image_extension(Path::new("a.webp")));
        assert!(is_supported_image_extension(Path::new("a.WEBP")));
        assert!(is_supported_image_extension(Path::new("a.jpg")));
        assert!(is_supported_image_extension(Path::new("a.JPG")));
        assert!(is_supported_image_extension(Path::new("a.jpeg")));
        assert!(!is_supported_image_extension(Path::new("a.gif")));
        assert!(!is_supported_image_extension(Path::new("a.txt")));
        assert!(!is_supported_image_extension(Path::new("a")));
    }
}

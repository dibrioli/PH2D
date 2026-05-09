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
}

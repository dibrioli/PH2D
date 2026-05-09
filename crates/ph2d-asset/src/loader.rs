//! Internal: byte → [`Asset`] decoders.
//!
//! Lives behind a private module so we can swap `image` for a
//! lower-level codec (e.g. `png` directly) later without breaking
//! [`AssetDb`] callers. M6 ships PNG only.

use crate::asset::Asset;
use crate::error::AssetError;
use std::path::Path;
use std::sync::Arc;

/// Decode an in-memory PNG buffer to [`Asset::ImageRgba8`].
/// `path_hint` is used only for error messages.
pub(crate) fn decode_png_bytes(
    bytes: &[u8],
    path_hint: Option<&Path>,
) -> Result<Asset, AssetError> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).map_err(|e| {
        AssetError::Decode {
            path: path_hint.map(Path::to_path_buf),
            message: e.to_string(),
        }
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

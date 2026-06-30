//! Brush-image import helpers for the Painter bridge — split out of
//! `painter_bridge.rs` for the HR-18 file-LOC cap.
//!
//! The pure brush engine has no file I/O; these open a native picker, decode
//! the chosen image, and install its Rec.601 luminance as the brush **Grain**
//! (texture) or **Shape** mask. Mirrors the M14.4c import path (rfd +
//! `AssetDb` decode). Cancel or any failure reverts the kind to None.

use ph2d_editor::toast::{Toast, ToastQueue};

/// Pick an image file and decode it to row-major Rec.601 luminance `(lum, w, h)`. `Ok(None)` = the
/// user cancelled the dialog; `Err` = a read/decode failure. Shared by the Grain + Shape importers.
fn pick_brush_luminance(
    asset_db: &ph2d_asset::AssetDb,
) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Image (PNG / WEBP / JPEG)", &["png", "webp", "jpg", "jpeg"])
        .pick_file()
    else {
        return Ok(None); // cancelled
    };
    let bytes = std::fs::read(&path).map_err(|e| format!("read: {e}"))?;
    let id = asset_db
        .insert_image_bytes(&bytes)
        .map_err(|e| format!("decode: {e}"))?;
    let asset = asset_db
        .get(&id)
        .ok_or_else(|| "asset missing".to_string())?;
    match &*asset {
        ph2d_asset::Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } => {
            // Rec.601 luminance: weights 77/150/29 sum to 256, so the `>> 8` keeps `[0,255]`.
            let lum: Vec<u8> = pixels
                .chunks_exact(4)
                .map(|p| {
                    ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8)
                        as u8
                })
                .collect();
            Ok(Some((lum, *width, *height)))
        }
        _ => Err("not an RGBA image".to_string()),
    }
}

/// Import an image as the brush **Grain** (texture). Cancel or failure reverts the kind to None.
pub(super) fn load_brush_texture_image(
    painter: &mut ph2d_tool_painter::PainterTool,
    asset_db: &ph2d_asset::AssetDb,
    toasts: &mut ToastQueue,
) {
    match pick_brush_luminance(asset_db) {
        Ok(Some((lum, w, h))) => {
            painter.set_brush_texture_image(lum, w, h);
            toasts.push(Toast::success("Brush texture loaded"));
        }
        Ok(None) => painter.set_brush_texture_kind(0), // cancelled → no texture
        Err(e) => {
            painter.set_brush_texture_kind(0); // revert on failure
            toasts.push(Toast::error(format!("Texture load failed: {e}")));
        }
    }
}

/// Import an image as the brush **Shape** (silhouette mask). Cancel or failure reverts to None (falloff).
pub(super) fn load_brush_shape_image(
    painter: &mut ph2d_tool_painter::PainterTool,
    asset_db: &ph2d_asset::AssetDb,
    toasts: &mut ToastQueue,
) {
    match pick_brush_luminance(asset_db) {
        Ok(Some((lum, w, h))) => {
            painter.set_brush_shape_image(lum, w, h);
            toasts.push(Toast::success("Brush shape loaded"));
        }
        Ok(None) => painter.set_brush_shape_kind(0), // cancelled → revert Image to None (falloff)
        Err(e) => {
            painter.set_brush_shape_kind(0); // revert on failure
            toasts.push(Toast::error(format!("Shape load failed: {e}")));
        }
    }
}

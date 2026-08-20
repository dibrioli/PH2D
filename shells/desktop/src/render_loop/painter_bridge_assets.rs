//! Brush-image import helpers for the Painter bridge — split out of
//! `painter_bridge.rs` for the HR-18 file-LOC cap.
//!
//! The pure brush engine has no file I/O; these open a native picker, decode
//! the chosen image, and install its Rec.601 luminance as the brush **Grain**
//! (texture) or **Shape** mask. Mirrors the M14.4c import path (rfd +
//! `AssetDb` decode). Cancel or any failure reverts the kind to None.

use ph2d_editor::toast::{Toast, ToastQueue};

/// Pick an image file and decode it to row-major RGBA `(rgba, w, h)`. `Ok(None)` = the user
/// cancelled the dialog; `Err` = a read/decode failure. Shared by the Grain + Shape importers.
fn pick_brush_rgba(asset_db: &ph2d_asset::AssetDb) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
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
    // ⚠️ `image_rgba8` e não um `match` na variante: o documento do Painter é de 8 bits, por isso
    // converter para baixo é a resposta certa (plano `docs/Sprite_projeto/18`, auditoria da W2).
    // Casar a variante fazia uma sprite de 16 bits recusar-se a abrir no Painter com "not an RGBA
    // image", que é uma mensagem falsa sobre uma imagem que É RGBA.
    match asset.image_rgba8() {
        Some((width, height, px)) => Ok(Some((px.into_owned(), width, height))),
        None => Err("not an RGBA image".to_string()),
    }
}

/// The picked image's Rec.601 luminance — the **Grain**'s plane, and only its.
///
/// ⚠️ A Grain é um mapa de TOM (ela modula a tinta dentro da silhueta), então o alpha de um arquivo
/// não tem papel nela; a Shape é uma SILHUETA, e por isso ela leva o RGBA inteiro. Converter aqui, no
/// consumidor que precisa de cinza, é o que deixa o outro consumidor com os pixels que ele precisa.
fn to_luminance(rgba: &[u8]) -> Vec<u8> {
    // Rec.601 luminance: weights 77/150/29 sum to 256, so the `>> 8` keeps `[0,255]`.
    rgba.chunks_exact(4)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect()
}

/// Import an image as the brush **Grain** (texture). Cancel or failure reverts the kind to None.
pub(super) fn load_brush_texture_image(
    painter: &mut ph2d_tool_painter::PainterTool,
    asset_db: &ph2d_asset::AssetDb,
    toasts: &mut ToastQueue,
) {
    match pick_brush_rgba(asset_db) {
        Ok(Some((rgba, w, h))) => {
            painter.set_brush_texture_image(to_luminance(&rgba), w, h);
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
    match pick_brush_rgba(asset_db) {
        Ok(Some((rgba, w, h))) => {
            // ⚠️ **A Shape leva o RGBA, não o cinza** (Enio, 2026-08-09): a silhueta pode vir do ALPHA
            // do arquivo — *"só coloca transparência onde há transparência na imagem usada"* —, e um
            // `.png` recortado que chegasse aqui já convertido não teria mais recorte nenhum para
            // silhuetar. De quebra, o arquivo passa a ter COR: o checkbox de cores da textura, que só
            // existia para o sprite do documento, vale para uma imagem importada também. `source_doc`
            // é `None` — estes pixels não vieram de entidade nenhuma, então nada os re-captura.
            painter.set_brush_shape_image_rgba(&rgba, w, h, None);
            toasts.push(Toast::success("Brush shape loaded"));
        }
        Ok(None) => painter.set_brush_shape_kind(0), // cancelled → revert Image to None (falloff)
        Err(e) => {
            painter.set_brush_shape_kind(0); // revert on failure
            toasts.push(Toast::error(format!("Shape load failed: {e}")));
        }
    }
}

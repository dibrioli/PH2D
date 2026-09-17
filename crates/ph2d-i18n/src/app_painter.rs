//! **O QUE A FAMÍLIA app.painter DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-painter` mostra (dos VERBOS do Painter (texturas de pincel, pré-visualização na GPU)), na forma `app.painter.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): as
//! cenas de smoke, o diagnóstico de consola, os formatos de ficheiro e os **nomes por omissão de
//! objecto** — um nome que entra no `Name` é identidade durável (`stable_name_id` fecha um hash
//! sobre ele), e traduzi-lo é decisão do dono, não desta migração.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.painter.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.painter.painter_bridge_assets.shape_load_failed" => "Shape load failed: {e}",
        "app.painter.painter_bridge_assets.brush_shape_loaded" => "Brush shape loaded",
        "app.painter.painter_bridge_assets.texture_load_failed" => "Texture load failed: {e}",
        "app.painter.painter_bridge_assets.brush_texture_loaded" => "Brush texture loaded",
        "app.painter.painter_bridge_assets.not_an_rgba_image" => "not an RGBA image",
        "app.painter.painter_bridge_assets.asset_missing" => "asset missing",
        "app.painter.painter_bridge_assets.decode" => "decode: {e}",
        "app.painter.painter_bridge_assets.read" => "read: {e}",
        "app.painter.painter_bridge_assets.image_png_webp_jpeg" => "Image (PNG / WEBP / JPEG)",
        "app.painter.painter_bridge_shape_preview.shape_preview_upload_failed" => {
            "Painter: could not upload the brush-shape preview to the GPU ({e})."
        }
        "app.painter.painter_bridge_upload.preview_upload_failed" => {
            "Painter: could not upload the preview to the GPU ({e}). Trying again next frame."
        }
        "app.painter.painter_gpu_preview.premultiply_produced_no_texture" => {
            "premultiply produced no texture"
        }
        "app.painter.painter_gpu_preview.impasto_light" => "impasto light: {e_}",
        "app.painter.painter_gpu_preview.composite_produced_no_texture" => {
            "composite produced no texture"
        }
        "app.painter.painter_gpu_preview.composite" => "composite: {e}",
        "app.painter.painter_gpu_preview.gpu_preview_copy_failed" => {
            "Painter: the GPU preview copy failed ({e}). Falling back to the CPU path."
        }
        "app.painter.painter_gpu_preview.gpu_preview_failed" => {
            "Painter: the GPU preview failed ({e}). Falling back to the CPU path."
        }
        "app.painter.painter_lock.leave_the_painter_to_select_another_sprite" => {
            "Leave the Painter to select another sprite"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}

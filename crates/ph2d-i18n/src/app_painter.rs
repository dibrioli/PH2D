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
        "app.painter.presets.saved" => "Preset saved: {name}",
        "app.painter.presets.save_failed" => "The preset could not be saved: {e}",
        "app.painter.presets.unreadable" => {
            "{n} preset files could not be read (names in the terminal)"
        }
        "app.painter.presets.state_unreadable" => {
            "Preset favorites and recents could not be read: {e}"
        }
        "app.painter.presets.state_save_failed" => {
            "Preset favorites and recents could not be saved: {e}"
        }
        "app.painter.presets.texture_unreadable" => {
            "Preset {name}: its {id} image could not be read, the slot stays empty"
        }
        "app.painter.presets.no_user_folder" => "No folder for your presets (HOME is not set)",
        "app.painter.presets.new_name" => "My brush",
        "app.painter.presets.group" => "{media} · {group}",
        "app.painter.painter_bridge_assets.not_an_rgba_image" => "not an RGBA image",
        "app.painter.painter_bridge_assets.asset_missing" => "asset missing",
        "app.painter.painter_bridge_assets.decode" => "decode: {e}",
        "app.painter.painter_bridge_assets.read" => "read: {e}",
        "app.painter.painter_bridge_assets.image_png_webp_jpeg" => "Image (PNG / WEBP / JPEG)",
        "app.painter.painter_bridge_shape_preview.shape_preview_upload_failed" => {
            "Painter: could not upload the brush-shape preview to the GPU ({e})."
        }
        "app.painter.macro_reescrita.write_failed" => {
            "Macro: could not rewrite the image after the step edit ({e})."
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
        // ph2d-migrar-texto:end
        "app.painter.skin_suspend.editing_pixels_flattens_this_image" => {
            "Editing pixels flattens this image \u{2014} the bone deformation returns when you leave the tool."
        }
        "app.painter.skin_suspend.this_tool_changes_the_image_size_or_margins" => {
            "This tool changes the image size or margins: the image is shown without the bone deformation, and Apply unbinds it from the bones."
        }
        "app.painter.skin_suspend.images_unbound_from_the_bones" => {
            "The image size or margins changed: {soltas} image(s) unbound from the bones. Ctrl+Z brings the binding back."
        }
        // Os presets nos quatro meios (L1): o grupo do navegador leva o meio; um meio que este PH2D não
        // conhece é recusado; um modo que o meio não tem não se aplica.
        "app.painter.presets.group_in_media" => "{engine} · {media} · {group}",
        "app.painter.presets.unknown_media" => {
            "The brush {name} is for a paint medium this PH2D does not know ({media}): it was not applied"
        }
        "app.painter.presets.mode_not_applied" => {
            "The brush {name} asks for a tool this paint medium does not have ({mode})"
        }
        // As pastas da FÁBRICA dos presets do Wet Painter (`assets/brushes/WetPainter/<meio>/<grupo>`),
        // como o navegador as mostra; a pasta do artista mostra-se como ele a chamou.
        "app.painter.presets.folder.watercolor" => "Watercolor",
        "app.painter.presets.folder.fluid" => "Fluid",
        "app.painter.presets.folder.oil" => "Oil",
        "app.painter.presets.folder.express_oil" => "Express Oil",
        "app.painter.presets.folder.pencil" => "Pencil",
        "app.painter.presets.folder.pastel" => "Pastel",
        "app.painter.presets.folder.bristle" => "Bristle",
        "app.painter.presets.folder.round" => "Round",
        "app.painter.presets.folder.graphite" => "Graphite",
        "app.painter.presets.folder.charcoal" => "Charcoal",
        "app.painter.presets.folder.dry" => "Dry",
        "app.painter.presets.folder.ink" => "Ink",
        "app.painter.presets.folder.pen" => "Pen",
        "app.painter.presets.folder.brush" => "Brush",
        "app.painter.presets.folder.marker" => "Marker",
        "app.painter.presets.folder.airbrush" => "Airbrush",
        "app.painter.presets.folder.spray" => "Spray",
        "app.painter.presets.folder.gouache" => "Gouache",
        "app.painter.presets.folder.colored" => "Colored",
        "app.painter.presets.folder.scratch" => "Scratch",
        "app.painter.presets.folder.scrape" => "Scrape",
        "app.painter.presets.folder.dabber" => "Dabber",
        _ => return None,
    })
}

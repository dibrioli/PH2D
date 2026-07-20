//! Live GPU preview of a sprite used as the brush **Shape** while it is NOT the selected sprite — split
//! from `painter_bridge` for the HR-18 file-LOC cap. Uploads the stashed Shape-source document into a
//! SEPARATE `IndividualTextureStore` slot so brush opacity/blend remote-control edits show on it in real
//! time. Reuses `painter_bridge::release_preview_texture` for slot teardown.

use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::{Toast, ToastQueue};
use ph2d_render::{SpriteRenderer, premultiply_rgba8};

use super::painter_bridge::release_preview_texture;
use crate::app_state::PainterPreviewGpu;

/// Drive the live preview of a sprite used as the brush **Shape** while it is NOT the selected sprite, so
/// brush opacity/blend remote-control edits show on it in real time. Mirrors the active-sprite preview but
/// into a SEPARATE [`ph2d_render::IndividualTextureStore`] slot ([`crate::app_state::AppState::painter_shape_source_preview_gpu`]):
/// the painter composites the stashed Shape-source document (only when dirty) and we upload it; the next
/// frame's `sim_extract` emits a second `PreviewOverride` suppressing that sprite + sampling this slot.
/// Released when the painter deactivates, the Shape source becomes the selected sprite, or it is cleared.
pub(super) fn drive_shape_source_preview(
    tools: &mut ToolRegistry,
    renderer: &mut SpriteRenderer,
    shape_source_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) {
    let painter_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);
    let painter = painter_active
        .then(|| tools.active_mut())
        .flatten()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
        });
    let Some(painter) = painter else {
        release_preview_texture(renderer, shape_source_preview_gpu);
        return;
    };
    let Some(sprite) = painter.shape_source_preview_sprite() else {
        release_preview_texture(renderer, shape_source_preview_gpu);
        return;
    };
    // Re-acquire if the slot is stale (a different shape-source sprite).
    if shape_source_preview_gpu.is_some_and(|g| g.entity_bits != sprite) {
        release_preview_texture(renderer, shape_source_preview_gpu);
    }
    // Recomposite + upload ONLY when the source changed (dirty); otherwise the held slot is re-used every
    // frame (the override samples it), so a static shape source costs nothing.
    let Some((bytes, w, h)) = painter.take_shape_source_preview() else {
        return;
    };
    let mut premul = bytes;
    premultiply_rgba8(&mut premul);
    let result = match *shape_source_preview_gpu {
        Some(gpu) if gpu.width == w && gpu.height == h => renderer
            .replace_individual_pixels(gpu.texture_id, w, h, &premul)
            .map(|()| gpu.texture_id),
        _ => {
            release_preview_texture(renderer, shape_source_preview_gpu); // dims changed → fresh slot
            renderer.acquire_individual(w, h, &premul)
        }
    };
    match result {
        Ok(texture_id) => {
            *shape_source_preview_gpu = Some(PainterPreviewGpu {
                texture_id,
                width: w,
                height: h,
                arc_token: 0, // not Arc-cached; dirty-gated re-upload instead
                entity_bits: sprite,
            });
        }
        Err(e) => {
            toasts.push(Toast::error(format!(
                "Painter: upload da preview da imagem-shape falhou ({e})."
            )));
            release_preview_texture(renderer, shape_source_preview_gpu);
        }
    }
}

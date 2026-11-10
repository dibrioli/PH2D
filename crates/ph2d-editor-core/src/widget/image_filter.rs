//! **O filtro de imagem do app visto pela UI** (Edit ▸ Preferences ▸ Image Filter): a porta [`image_quality_for`] e o
//! valor que o shell escreve a cada quadro. Mora no `widget`, abaixo de quem pinta e de `project` (DAG da fundação);
//! o `project` reexporta-o. Global como o idioma do `tr`: passá-lo por cada painel deixava os painéis suavizados com
//! o canvas em Pixel Art (smoke do dono 09/11).

use ph2d_host::ImageFilterMode;
use ph2d_vector::ImageQuality;
use std::sync::atomic::{AtomicBool, Ordering};

/// Map the app-wide [`ImageFilterMode`] (defined in `ph2d-host`) onto
/// the peniko [`ImageQuality`] used by the Vello image preview
/// (`VectorScene::draw_image_rgba`). Companion of
/// `ph2d_render::wgpu_filter`, which maps the SAME enum onto the wgpu
/// sprite sampler — keeping the on-canvas BG-Removal preview consistent
/// with the baked sprite.
///
/// - `PixelArt` → [`ImageQuality::Low`]  (nearest-neighbor, crisp)
/// - `Smooth`   → [`ImageQuality::High`] (bicubic, smooth)
pub fn image_quality_for(mode: ImageFilterMode) -> ImageQuality {
    match mode {
        ImageFilterMode::PixelArt => ImageQuality::Low,
        ImageFilterMode::Smooth => ImageQuality::High,
    }
}

/// `true` = [`ImageFilterMode::PixelArt`]; o omisso é o do projeto novo (`Smooth`).
static UI_IMAGE_PIXEL_ART: AtomicBool = AtomicBool::new(false);

/// Escrito pelo shell a cada quadro com o filtro do projeto.
pub fn set_ui_image_filter(mode: ImageFilterMode) {
    UI_IMAGE_PIXEL_ART.store(mode == ImageFilterMode::PixelArt, Ordering::Relaxed);
}

/// ⭐ O filtro com que um painel pinta uma imagem do jogo: o mesmo do canvas ([`image_quality_for`]).
#[must_use]
pub fn ui_image_quality() -> ImageQuality {
    image_quality_for(if UI_IMAGE_PIXEL_ART.load(Ordering::Relaxed) {
        ImageFilterMode::PixelArt
    } else {
        ImageFilterMode::Smooth
    })
}

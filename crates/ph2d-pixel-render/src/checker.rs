//! O xadrez de transparência: a imagem em cache e o seu desenho recortado ao canvas.

use std::sync::Arc;

use ph2d_vector::{Affine, ImageQuality, StableImage, VectorScene};

/// O lado de uma casa do xadrez, em células do sprite (Aseprite: *Background ▸ Checked size* 16×16,
/// com *Apply Zoom*).
pub const CHECKER_CELLS: u32 = 16;

// O xadrez de transparência: cinza sobre cinza, invariante ao tema (a convenção da casa em
// `ph2d-editor-core/src/widget/color_swatch.rs`).
const CHECKER_LIGHT: [u8; 4] = [204, 204, 204, 255]; // LITERAL-COLOR-OK: casa clara do xadrez de transparência (invariante ao tema)
const CHECKER_DARK: [u8; 4] = [153, 153, 153, 255]; // LITERAL-COLOR-OK: casa escura do xadrez de transparência (invariante ao tema)

/// **O xadrez de transparência** de um sprite (casas de [`CHECKER_CELLS`] células), em cache: a
/// imagem só se refaz quando o número de casas muda. O canvas e a janela Preview pintam o MESMO.
#[derive(Clone, Default)]
pub struct Checker(Option<(u32, u32, StableImage)>);

impl Checker {
    /// O xadrez por baixo de um sprite `w × h` posto por `to_screen` (célula → ecrã).
    pub fn draw_under(&mut self, scene: &mut VectorScene, w: u32, h: u32, to_screen: Affine) {
        let cols = w.div_ceil(CHECKER_CELLS);
        let rows = h.div_ceil(CHECKER_CELLS);
        if self
            .0
            .as_ref()
            .is_none_or(|(c, r, _)| (*c, *r) != (cols, rows))
        {
            let mut px = Vec::with_capacity(cols as usize * rows as usize * 4);
            for y in 0..rows {
                for x in 0..cols {
                    px.extend_from_slice(if (x + y) % 2 == 0 {
                        &CHECKER_LIGHT
                    } else {
                        &CHECKER_DARK
                    });
                }
            }
            self.0 = StableImage::from_rgba(Arc::new(px), cols, rows).map(|img| (cols, rows, img));
        }
        draw_checker(scene, self.0.as_ref().map(|c| &c.2), w, h, to_screen);
    }
}

/// O xadrez `img` (uma célula por casa) recortado ao canvas `w × h`.
pub fn draw_checker(
    scene: &mut VectorScene,
    img: Option<&StableImage>,
    w: u32,
    h: u32,
    to_screen: Affine,
) {
    let Some(img) = img else {
        return;
    };
    let k = f64::from(CHECKER_CELLS);
    let clip = ph2d_vector::Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
    scene
        .inner_mut()
        .push_clip_layer(ph2d_vector::Fill::NonZero, to_screen, &clip);
    scene.draw_stable_image_transformed(img, to_screen * Affine::scale(k), ImageQuality::Low);
    scene.inner_mut().pop_layer();
}

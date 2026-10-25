//! **O desenho**: todo Pixel Sprite visível como imagem nítida no lugar do objecto; o que está em
//! Edit leva o xadrez por baixo, a borda, a grelha de pixels (se ligada) e o cursor de célula.
//!
//! ⚠️ P0: desenha-se na cena de sobreposição do viewport (por cima dos sprites do mundo). Entrar na
//! ordem de desenho do mundo (luz, camadas, câmera do jogo) é a P14 — «o sprite vive na cena».

use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_pixel_render::{draw_border, draw_cell_cursor, draw_pixel_grid, px_per_cell};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::PixelApp;
use crate::input::to_screen;
use crate::mode::sprites;

/// Espessura das linhas do modo Edit, em píxeis de ECRÃ (saem com `Affine::IDENTITY`).
const LINE_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de sobreposição, espessura de ecrã
/// Abaixo disto a grelha de pixels vira um borrão cinzento (uma linha por célula de 1–3 px).
const GRID_MIN_PX_PER_CELL: f64 = 4.0; // LITERAL-PX-OK: legibilidade da grelha, em píxeis de ecrã

/// Desenha os Pixel Sprites do mundo.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    camera: &Camera2d,
    window: WindowSize,
    ppm: f32,
    editing: Option<u64>,
    theme: Theme,
    scene: &mut VectorScene,
) {
    for (bits, key) in sprites(world) {
        let Some(entity) = ph2d_ecs::Entity::try_from_bits(bits) else {
            continue;
        };
        if ph2d_ecs::visibility::is_hidden_in_tree(world, entity) {
            continue;
        }
        let pending = app
            .gesture
            .as_ref()
            .filter(|g| g.key == key)
            .map(|g| g.stroke.clone());
        let Some(doc) = app.docs.get_mut(&key) else {
            continue;
        };
        let Some(m) = to_screen(world, bits, doc, camera, window, ppm) else {
            continue;
        };
        let (w, h) = (doc.sprite.width(), doc.sprite.height());
        doc.view
            .refresh(&doc.sprite, doc.frame, pending.as_ref(), doc.stamp);
        let edit = editing == Some(bits);
        if edit {
            doc.view.draw_checker_under(scene, w, h, m);
        }
        doc.view.draw(scene, m);
        if !edit {
            continue;
        }
        if app.grid && px_per_cell(m) >= GRID_MIN_PX_PER_CELL {
            draw_pixel_grid(scene, w, h, m, resolve(ColorToken::Border, theme), LINE_PX);
        }
        draw_border(
            scene,
            w,
            h,
            m,
            resolve(ColorToken::Selection, theme),
            LINE_PX,
        );
        if let Some((hb, cell)) = app.hover
            && hb == bits
            && doc.sprite.bounds().contains(cell.0, cell.1)
        {
            draw_cell_cursor(scene, cell, m, resolve(ColorToken::Text1, theme), LINE_PX);
        }
    }
}

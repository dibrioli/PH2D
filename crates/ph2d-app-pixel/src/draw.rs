//! **O desenho**: todo Pixel Sprite visível como imagem nítida no lugar do objecto; o que está em
//! Draw leva o xadrez por baixo, a borda, a grelha de pixels (se ligada) e o cursor de célula.
//!
//! ⚠️ P0: desenha-se na cena de sobreposição do viewport (por cima dos sprites do mundo). Entrar na
//! ordem de desenho do mundo (luz, camadas, câmera do jogo) é a P14 — «o sprite vive na cena».

use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_pixel_render::{
    draw_border, draw_cell_cursor, draw_pixel_grid, draw_symmetry_axes, px_per_cell,
};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::PixelApp;
use crate::input::to_screen;
use crate::mode::sprites;

/// Um Pixel Sprite como o último quadro o desenhou: o clique e a caixa do gizmo perguntam AQUI, à
/// mesma geometria que a tinta usou (nunca uma segunda conta).
#[derive(Debug, Clone, Copy)]
struct Drawn {
    bits: u64,
    to_screen: ph2d_vector::Affine,
    size: (u32, u32),
}

thread_local! {
    static DRAWN: std::cell::RefCell<Vec<Drawn>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Os Pixel Sprites sob o ponto de ecrã `p` (dentro do canvas), do de cima para o de baixo.
#[must_use]
pub fn pick_at(p: (f32, f32)) -> Vec<u64> {
    DRAWN.with(|d| {
        d.borrow()
            .iter()
            .rev()
            .filter(|s| {
                ph2d_pixel_render::cell_at(s.to_screen, (f64::from(p.0), f64::from(p.1)))
                    .is_some_and(|(x, y)| {
                        x >= 0 && y >= 0 && (x as u32) < s.size.0 && (y as u32) < s.size.1
                    })
            })
            .map(|s| s.bits)
            .collect()
    })
}

/// A meia-extensão no espaço LOCAL do objecto (metros, antes da escala) do Pixel Sprite `bits` —
/// a caixa do gizmo de objecto. `None` = não foi desenhado no último quadro.
#[must_use]
pub fn half_extents(bits: u64, ppm: f32) -> Option<[f32; 2]> {
    let ppm = ppm.max(f32::MIN_POSITIVE);
    DRAWN.with(|d| {
        d.borrow()
            .iter()
            .find(|s| s.bits == bits)
            .map(|s| [s.size.0 as f32 * 0.5 / ppm, s.size.1 as f32 * 0.5 / ppm])
    })
}

/// Espessura das linhas do modo Draw, em píxeis de ECRÃ (saem com `Affine::IDENTITY`).
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
    let mut drawn = Vec::new();
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
        let o = app.options;
        if edit && (o.tile_x || o.tile_y) {
            // O Tiled Mode mostra as vizinhas: o desenho que sai por um lado entra pelo outro.
            let (fw, fh) = (f64::from(w), f64::from(h));
            let xs: &[f64] = if o.tile_x { &[-1.0, 0.0, 1.0] } else { &[0.0] };
            let ys: &[f64] = if o.tile_y { &[-1.0, 0.0, 1.0] } else { &[0.0] };
            for &ty in ys {
                for &tx in xs {
                    if (tx, ty) != (0.0, 0.0) {
                        let k = m * ph2d_vector::Affine::translate((tx * fw, ty * fh));
                        doc.view.draw_checker_under(scene, w, h, k);
                        doc.view.draw(scene, k);
                    }
                }
            }
        }
        if edit {
            doc.view.draw_checker_under(scene, w, h, m);
        }
        doc.view.draw(scene, m);
        drawn.push(Drawn {
            bits,
            to_screen: m,
            size: (w, h),
        });
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
        if app.tool.paints() && (o.sym_h || o.sym_v || o.sym_d1 || o.sym_d2) {
            let s = crate::gesture::symmetry(&o, doc);
            draw_symmetry_axes(
                scene,
                (w, h),
                m,
                (s.axis2_x, s.axis2_y),
                (o.sym_h, o.sym_v, o.sym_d1, o.sym_d2),
                resolve(ColorToken::Accent, theme),
                LINE_PX,
            );
        }
        if let Some((hb, cell)) = app.hover
            && hb == bits
            && doc.sprite.bounds().contains(cell.0, cell.1)
        {
            draw_cell_cursor(scene, cell, m, resolve(ColorToken::Text1, theme), LINE_PX);
        }
    }
    DRAWN.with(|d| *d.borrow_mut() = drawn);
}

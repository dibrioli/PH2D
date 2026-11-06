//! A grelha de tiles da camada activa, por cima do canvas.

use ph2d_pixel_core::{FrameId, LayerId, Sprite};
use ph2d_vector::{Affine, Color, VectorScene};

use crate::mao::tileset_de;
use crate::mapa::Mapa;

/// A grelha da camada, se os tiles couberem na vista (`minimo` px de ecrã), e o contorno da célula
/// que contém o píxel `realce`. Numa camada sem tileset não desenha nada.
pub fn desenha(
    s: &Sprite,
    (layer, frame): (LayerId, FrameId),
    realce: Option<(i32, i32)>,
    m: Affine,
    (linha, minimo): (f64, f64),
    (grelha, acento): (Color, Color),
    scene: &mut VectorScene,
) {
    let Some(set) = tileset_de(s, layer) else {
        return;
    };
    let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
    let mapa = Mapa::de(s, layer, frame, set);
    if ph2d_pixel_render::px_per_cell(m) * f64::from(tw.min(th)) >= minimo {
        let (w, h) = (s.width(), s.height());
        let origem = mapa.origem(0, 0);
        ph2d_pixel_render::draw_tile_grid(scene, (w, h), (tw, th), origem, m, grelha, linha);
    }
    if let Some(cell) = realce {
        let (c, r) = mapa.celula(cell.0, cell.1);
        let (x, y) = mapa.origem(c, r);
        let em = m * Affine::translate((f64::from(x), f64::from(y)));
        ph2d_pixel_render::draw_border(scene, tw, th, em, acento, linha);
    }
}

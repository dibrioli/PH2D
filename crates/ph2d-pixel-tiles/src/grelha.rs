//! A grelha de tiles da camada activa, por cima do canvas.

use ph2d_pixel_core::{Forma, FrameId, LayerId, Sprite};
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
        if set.forma == Forma::Square {
            let origem = mapa.origem(0, 0);
            ph2d_pixel_render::draw_tile_grid(scene, (w, h), (tw, th), origem, m, grelha, linha);
        } else {
            let contornos = contornos_no_canvas(&mapa, (w as i32, h as i32), (tw, th));
            ph2d_pixel_render::draw_polygons(scene, &contornos, m, grelha, linha);
        }
    }
    if let Some(cell) = realce {
        let (c, r) = mapa.celula(cell.0, cell.1);
        let (x, y) = mapa.origem(c, r);
        if set.forma == Forma::Square {
            let em = m * Affine::translate((f64::from(x), f64::from(y)));
            ph2d_pixel_render::draw_border(scene, tw, th, em, acento, linha);
        } else {
            let p = contorno(set.forma, (x, y), (tw, th));
            ph2d_pixel_render::draw_polygons(scene, &[p], m, acento, linha);
        }
    }
}

/// O contorno da célula cuja caixa começa em `(x, y)`.
#[must_use]
pub fn contorno(forma: Forma, (x, y): (i32, i32), (tw, th): (u32, u32)) -> Vec<(f64, f64)> {
    let (w, h) = (f64::from(tw), f64::from(th));
    let u: &[(f64, f64)] = match forma {
        Forma::Square => &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        Forma::Iso => &[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)],
        Forma::Hex => &[
            (0.5, 0.0),
            (1.0, 0.25),
            (1.0, 0.75),
            (0.5, 1.0),
            (0.0, 0.75),
            (0.0, 0.25),
        ],
        Forma::HexFlat => &[
            (0.25, 0.0),
            (0.75, 0.0),
            (1.0, 0.5),
            (0.75, 1.0),
            (0.25, 1.0),
            (0.0, 0.5),
        ],
    };
    u.iter()
        .map(|(a, b)| (f64::from(x) + a * w, f64::from(y) + b * h))
        .collect()
}

/// Os contornos das células que tocam o canvas.
fn contornos_no_canvas(
    mapa: &Mapa,
    (w, h): (i32, i32),
    (tw, th): (u32, u32),
) -> Vec<Vec<(f64, f64)>> {
    let cantos = [(0, 0), (w, 0), (0, h), (w, h)].map(|(x, y)| mapa.celula(x, y));
    let de = |f: fn(&(i32, i32)) -> i32| {
        let v = cantos.iter().map(f);
        (v.clone().min().unwrap_or(0) - 1, v.max().unwrap_or(0) + 1)
    };
    let (c0, c1) = de(|p| p.0);
    let (r0, r1) = de(|p| p.1);
    let (lw, lh) = (tw as i32, th as i32);
    let mut out = Vec::new();
    for r in r0..=r1 {
        for c in c0..=c1 {
            let (x, y) = mapa.origem(c, r);
            if x < w && y < h && x + lw > 0 && y + lh > 0 {
                out.push(contorno(mapa.forma(), (x, y), (tw, th)));
            }
        }
    }
    out
}

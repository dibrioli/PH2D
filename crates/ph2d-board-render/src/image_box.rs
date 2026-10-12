//! A caixa de uma imagem do quadro e os LUGARES dela (módulo filho de [`super`]): o afim do nível à caixa,
//! o contorno da caixa (com os cantos), o lugar de uma imagem que ainda não chegou (ou que não se lê) e a
//! marca de uma estragada de que se lêem as linhas de cima (W18).

use ph2d_board_geom::corner_radius;
use ph2d_board_model::{Element, Image};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Point, Rect, RoundedRect, Shape as _, Stroke, VectorScene};

use crate::token;

/// O afim do píxel do NÍVEL (`level` de tamanho, de uma imagem natural `natural`) ao local da caixa
/// do elemento: o recorte estica-se à caixa, e o espelho vira à volta do centro.
#[must_use]
pub fn level_to_box(el: &Element, img: &Image, natural: [u32; 2], level: [u32; 2]) -> Affine {
    ph2d_board_geom::image_to_box(el, img.source(natural), img.flip)
        * Affine::scale_non_uniform(
            f64::from(natural[0]) / f64::from(level[0].max(1)),
            f64::from(natural[1]) / f64::from(level[1].max(1)),
        )
}

/// A caixa local do elemento, com os cantos do rectângulo redondo se `round`.
pub(crate) fn box_path(el: &Element, round: bool) -> ph2d_vector::BezPath {
    let r = if round {
        corner_radius(el.w.min(el.h))
    } else {
        0.0
    };
    RoundedRect::new(0.0, 0.0, el.w, el.h, r).to_path(0.1)
}

/// O lugar de uma imagem que ainda não chegou: a caixa no fundo elevado e, ao meio, o ícone de imagem —
/// a moldura, o sol e o monte (o idioma do Excalidraw, `w14_imagens.png`). W17 — `broken` = a que NÃO
/// se lê: o mesmo ícone na tinta do perigo, riscado (o Excalidraw não distingue: desenha o que o
/// navegador descodificar, ou nada — `saidas/w17`).
pub(super) fn placeholder(scene: &mut VectorScene, el: &Element, t: Affine, theme: Theme, broken: bool) {
    scene.fill_path(
        &box_path(el, el.image().is_some_and(|i| i.style.round)),
        &Brush::Solid(token(ColorToken::Bg3, theme)),
        t,
    );
    let side = el.w.min(el.h) * 0.3;
    if side <= 0.0 {
        return;
    }
    let (cx, cy) = (el.w / 2.0, el.h / 2.0);
    let (x0, y0) = (cx - side / 2.0, cy - side * 0.4);
    let (x1, y1) = (cx + side / 2.0, cy + side * 0.4);
    let ink = Brush::Solid(token(
        if broken {
            ColorToken::Danger
        } else {
            ColorToken::Text2
        },
        theme,
    ));
    let frame = Rect::new(x0, y0, x1, y1).to_rounded_rect(side * 0.08);
    scene.stroke_path(&Stroke::new(side * 0.07), t, &ink, &frame);
    if broken {
        let mut slash = ph2d_vector::BezPath::new();
        slash.move_to((x0 - side * 0.1, y0 - side * 0.1));
        slash.line_to((x1 + side * 0.1, y1 + side * 0.1));
        scene.stroke_path(&Stroke::new(side * 0.07), t, &ink, &slash);
    }
    let sun = Circle::new((x0 + side * 0.3, y0 + side * 0.25), side * 0.09);
    scene.fill_path(&sun.to_path(0.1), &ink, t);
    let mut hill = ph2d_vector::BezPath::new();
    hill.move_to((x0 + side * 0.1, y1 - side * 0.1));
    hill.line_to((x0 + side * 0.4, y0 + side * 0.4));
    hill.line_to((x0 + side * 0.6, y1 - side * 0.3));
    hill.line_to((x0 + side * 0.72, y0 + side * 0.52));
    hill.line_to((x1 - side * 0.1, y1 - side * 0.1));
    hill.close_path();
    scene.fill_path(&hill, &ink, t);
}

/// W18 — a marca de uma imagem estragada de que se lêem as linhas de cima: o círculo do perigo com o
/// risco do ícone de imagem, no canto de cima à direita (o desenho continua a mostrar o que se leu).
pub(super) fn broken_mark(scene: &mut VectorScene, el: &Element, t: Affine, theme: Theme) {
    let r = el.w.min(el.h) * 0.08;
    if r <= 0.0 {
        return;
    }
    let c = Point::new(el.w - r * 1.6, r * 1.6);
    scene.fill_path(
        &Circle::new(c, r).to_path(0.1),
        &Brush::Solid(token(ColorToken::Danger, theme)),
        t,
    );
    let mut slash = ph2d_vector::BezPath::new();
    slash.move_to((c.x - r * 0.5, c.y - r * 0.5));
    slash.line_to((c.x + r * 0.5, c.y + r * 0.5));
    scene.stroke_path(
        &Stroke::new(r * 0.25),
        t,
        &Brush::Solid(token(ColorToken::Bg1, theme)),
        &slash,
    );
}


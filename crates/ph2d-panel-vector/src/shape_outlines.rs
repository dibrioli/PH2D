//! O **catálogo de formas** cozido: o contorno de cada forma num quadrado unitário — a cara do
//! botão dela no flyout da coluna ① do vetor (D10, `docs/UI_New_and_Simple/vetor_nova_ui/`).
//!
//! ⚠️ A forma é cozida pela MESMA porta que o canvas usa (`vec_scene::cook` →
//! `vec_render::build_bezpath`): um segundo desenho divergiria dela no primeiro ajuste. A grade
//! de thumbnails e o seletor de categoria que viviam aqui saíram com a secção `Shape` do painel
//! (D10: o catálogo é o flyout da Forma).
//!
//! ⚠️ Os 48 `BezPath` são cacheados num `thread_local`: `cook` usa transcendentais, e o flyout
//! pede-os em todo quadro em que está aberto.

use ph2d_tool_vector::shapes;
use ph2d_vec_render::{build_bezpath, build_fill_bezpath, build_lines_bezpath};
use ph2d_vec_scene::cook;
use ph2d_vector::{Affine, BezPath, Shape};
use std::cell::RefCell;

/// A caixa em que cada forma é cozida para o thumbnail: 2×2 centrada na origem. É a mesma
/// ordem de grandeza de um traço real (a viewport toda tem ~10 unidades de mundo), então
/// os defaults que vêm em MUNDO — `DEFAULT_CORNER_RADIUS = 0.12` — saem na proporção que
/// o usuário verá ao desenhar, e não como um canto absurdo ou invisível.
const THUMB_BOX_MIN: [f64; 2] = [-1.0, -1.0];
const THUMB_BOX_MAX: [f64; 2] = [1.0, 1.0];

/// O preview de uma forma: o que se PREENCHE e o que se DESENHA.
///
/// São dois caminhos porque um contorno aberto **não tem interior**: as arestas internas do
/// cubo, a boca do cone, a tampa do cilindro, as barras da sub-rotina. Preencher junto com a
/// silhueta fecharia cada um implicitamente e recortaria a forma (foi o triângulo escuro no
/// cubo). E ignorá-los de todo faria o cubo virar um hexágono liso. Eles se **traçam**.
struct Thumb {
    /// Os contornos FECHADOS — a silhueta preenchível.
    fill: BezPath,
    /// Os contornos ABERTOS — as linhas de construção (vazio em 40 das 48 formas).
    lines: BezPath,
    /// A bbox da forma INTEIRA (silhueta + linhas): é ela que dita o encaixe na célula,
    /// senão as duas partes sairiam com escalas diferentes e não se alinhariam.
    bbox: ph2d_vector::Rect,
}

thread_local! {
    /// Os previews do catálogo, cozidos UMA vez por thread (o `cook` chama seno/cosseno;
    /// 48 formas por frame seria puro desperdício). Indexado pelo índice do catálogo.
    static SHAPE_THUMBS: RefCell<Vec<Thumb>> = const { RefCell::new(Vec::new()) };
}

/// Roda `f` com o preview da forma `index`, construindo o cache na primeira chamada.
fn with_thumb<R>(index: usize, f: impl FnOnce(&Thumb) -> R) -> Option<R> {
    SHAPE_THUMBS.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.is_empty() {
            cache.extend(shapes::SHAPES.iter().map(|d| {
                let path = cook(d.kind, THUMB_BOX_MIN, THUMB_BOX_MAX, &d.kind.defaults());
                let whole = build_bezpath(&path);
                Thumb {
                    fill: build_fill_bezpath(&path),
                    lines: build_lines_bezpath(&path),
                    bbox: whole.bounding_box(),
                }
            }));
        }
        cache.get(index).map(f)
    })
}

/// ⭐ **O contorno da forma `index` num quadrado unitário** (`0..1`, y para BAIXO, centrado, escala
/// uniforme) — a cara do botão dela no flyout da coluna ① (D10). O MESMO cozido do thumbnail: um
/// segundo desenho da forma divergiria dele no primeiro ajuste.
pub(crate) fn thumb_outline(index: usize) -> Option<BezPath> {
    with_thumb(index, |t| {
        let (w, h) = (t.bbox.width(), t.bbox.height());
        let side = w.max(h);
        if side <= 0.0 {
            return None;
        }
        let c = t.bbox.center();
        // y do mundo para CIMA → y do ecrã para BAIXO (a mesma inversão do `thumb_affine`).
        let to_unit = Affine::new([
            1.0 / side,
            0.0,
            0.0,
            -1.0 / side,
            0.5 - c.x / side,
            0.5 + c.y / side,
        ]);
        let mut out = to_unit * t.fill.clone();
        out.extend(to_unit * t.lines.clone());
        Some(out)
    })
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O contorno não pode sair espelhado.** O mundo é Y-para-CIMA e o botão é Y-para-baixo: a
    /// grade antiga foi fotografada com os balões de cabeça para baixo. O ápice do cone tem de
    /// ficar ACIMA da base — a única asserção que distingue o certo do espelhado.
    #[test]
    fn the_outline_flips_y_so_the_shape_is_not_upside_down() {
        let cone = shapes::SHAPES
            .iter()
            .position(|d| d.kind == ph2d_vec_scene::ShapeKind::IsoCone)
            .expect("o cone está no catálogo");
        let path = thumb_outline(cone).expect("o cone não é degenerado");
        let world = build_bezpath(&cook(
            ph2d_vec_scene::ShapeKind::IsoCone,
            THUMB_BOX_MIN,
            THUMB_BOX_MAX,
            &ph2d_vec_scene::ShapeKind::IsoCone.defaults(),
        ))
        .bounding_box();
        // O ápice é o ponto de MAIOR y no mundo; no quadrado unitário tem de ter o MENOR y.
        let side = world.width().max(world.height());
        let to_unit = |y: f64| 0.5 + world.center().y / side - y / side;
        assert!(
            to_unit(world.y1) < to_unit(world.y0),
            "o ápice tem de ficar acima da base"
        );
        let b = path.bounding_box();
        assert!(
            b.y0 >= -1e-6 && b.y1 <= 1.0 + 1e-6,
            "o cone vazou o quadrado: {b:?}"
        );
    }

    /// Toda forma do catálogo cabe no quadrado unitário — senão pinta por cima do botão vizinho.
    #[test]
    fn every_catalog_outline_fits_the_unit_square() {
        let mut medidas = 0usize;
        for i in 0..shapes::SHAPES.len() {
            let Some(p) = thumb_outline(i) else {
                continue; // degenerada: não há o que pintar
            };
            let b = p.bounding_box();
            assert!(
                b.x0 >= -1e-6 && b.y0 >= -1e-6 && b.x1 <= 1.0 + 1e-6 && b.y1 <= 1.0 + 1e-6,
                "{:?}: o contorno {b:?} vazou o quadrado unitário",
                shapes::SHAPES[i].kind
            );
            medidas += 1;
        }
        assert!(medidas >= 40, "controlo: só {medidas} formas medidas");
    }
}

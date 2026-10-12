//! W15 — **a MOLDURA de uma imagem do quadro**: o contorno do rectângulo da caixa com o estilo da
//! imagem, pela lei das formas (o traço, o tracejado, o rascunho, os cantos). O Excalidraw 0.18.1 não a
//! desenha (`ferramentas/excalidraw_oracle/saidas/w15/`: 0 píxeis do traço) — é nossa, e viaja no
//! `customData`.

use ph2d_board_model::{Element, Image, RichText, Shape, ShapeType};
use ph2d_vector::{Affine, Brush, VectorScene};

use crate::RenderCache;

/// O rectângulo (de cantos redondos se `round`) com o estilo da imagem e sem preenchimento: a forma
/// cujo contorno é a moldura.
fn shape(img: &Image) -> Shape {
    let mut style = img.style.clone();
    style.fill = None;
    Shape {
        kind: ShapeType::Rectangle,
        style,
        text: RichText::default(),
    }
}

/// Até onde a moldura sai da caixa (o traço e o tremor do rascunho).
pub(crate) fn reach(el: &Element, img: &Image) -> f64 {
    let st = &img.style;
    st.stroke_width
        + if st.sketch {
            crate::sketch::hand_reach(st, el.w, el.h)
        } else {
            0.0
        }
}

/// ⭐ A MOLDURA (W15): o contorno do rectângulo da caixa pela lei das formas — o traço e o tracejado
/// do estilo, à mão se `sketch`, a cor pela tinta do tema (`cache.ink`), por cima da imagem.
pub(crate) fn paint(
    scene: &mut VectorScene,
    cache: &mut RenderCache,
    el: &Element,
    img: &Image,
    t: Affine,
    board: u64,
) {
    let st = &img.style;
    let Some(c) = st.stroke else {
        return;
    };
    let shape = shape(img);
    let (frame, at) = (cache.frame, (board, el.id.0));
    let o = crate::cached_outline(&mut cache.outlines, frame, at, &shape, el.w, el.h);
    if st.sketch {
        debug_assert!(st.fill.is_none(), "uma imagem não tem preenchimento");
        crate::sketch::paint_shape(scene, &mut cache.sketch, frame, at, el, shape.kind, o, t);
    } else {
        let stroke = crate::style_stroke(st.stroke_width, st.dash);
        scene.stroke_path(&stroke, t, &Brush::Solid(cache.ink.color(c)), &o.fill);
    }
}


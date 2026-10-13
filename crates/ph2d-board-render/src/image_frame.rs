//! W15 — **a MOLDURA de uma imagem do quadro**: o contorno do rectângulo da caixa com o estilo da
//! imagem, pela lei das formas (o traço, o tracejado, o rascunho, os cantos). O Excalidraw 0.18.1 não a
//! desenha (`ferramentas/excalidraw_oracle/saidas/w15/`: 0 píxeis do traço) — é nossa, e viaja no
//! `customData`. W20: a MESMA moldura é a borda de um VÍDEO (o Excalidraw desenha-a num `embeddable`,
//! `saidas/w20` E2) — lá o tocador recua para dentro dela ([`reach`]), nada fica por cima dele.

use ph2d_board_model::{Element, RichText, Shape, ShapeType, Style};
use ph2d_vector::{Affine, Brush, VectorScene};

use crate::RenderCache;

/// O rectângulo (de cantos redondos se `round`) com o estilo da caixa e sem preenchimento: a forma
/// cujo contorno é a moldura.
fn shape(st: &Style) -> Shape {
    let mut style = st.clone();
    style.fill = None;
    Shape {
        kind: ShapeType::Rectangle,
        style,
        text: RichText::default(),
    }
}

/// Até onde a moldura sai da caixa (o traço e o tremor do rascunho) — e, num vídeo, quanto o tocador
/// recua para dentro dela.
pub(crate) fn reach(el: &Element, st: &Style) -> f64 {
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
    st: &Style,
    t: Affine,
    board: u64,
) {
    let Some(c) = st.stroke else {
        return;
    };
    let shape = shape(st);
    let (frame, at) = (cache.frame, (board, el.id.0));
    let o = crate::cached_outline(&mut cache.outlines, frame, at, &shape, el.w, el.h);
    if st.sketch {
        // O rascunho lê o estilo do elemento: a moldura nunca preenche (o fundo de um vídeo fica POR
        // TRÁS do tocador, `video::paint`).
        let bare;
        let el = if el.style().fill.is_some() {
            bare = {
                let mut b = el.clone();
                b.style_mut().fill = None;
                b
            };
            &bare
        } else {
            el
        };
        crate::sketch::paint_shape(scene, &mut cache.sketch, frame, at, el, shape.kind, o, t);
    } else {
        let stroke = crate::style_stroke(st.stroke_width, st.dash);
        scene.stroke_path(&stroke, t, &Brush::Solid(cache.ink.color(c)), &o.fill);
    }
}

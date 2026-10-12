//! Cena 16 — A MOLDURA, O PRETO E BRANCO E O SVG (W15), módulo filho do roteador [`super`]: num frame
//! «Frames» a fotografia da cena 15 com uma moldura de cada estilo (preta contínua, vermelha
//! tracejada, azul à mão e redonda); num frame «Black and white» a MESMA fotografia a cores ao lado da
//! cinzenta; num frame «SVG» o mesmo logótipo vectorial pequeno e grande (nítido ao zoom); por baixo
//! uma nota por lição — a última é o «Send to board…» da Hierarchy. O gate
//! `the_frames_scene_does_what_its_lessons_say` conduz o que a cena arma.

use super::images::{file, image, photo};
use super::team::{note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Dash, Element, ElementId, ImageFilter, Rgba, RichText, STICKY_SIDE,
    Shape, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    /// As três molduras: contínua, tracejada, à mão.
    pub framed: [ElementId; 3],
    pub color: ElementId,
    pub gray: ElementId,
    pub svg_small: ElementId,
    pub svg_big: ElementId,
    pub lessons: [ElementId; 6],
}

const GAP: f64 = 40.0;
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;

/// O logótipo vectorial de demonstração (120 × 120): um quadrado redondo, um anel e uma estrela.
// LITERAL-COLOR-OK: cores de um SVG de demonstração (dado do documento, não da UI).
const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120"><rect x="4" y="4" width="112" height="112" rx="22" fill="#3b5bdb"/><circle cx="60" cy="60" r="38" fill="none" stroke="#ffd43b" stroke-width="6"/><path d="M60 30 L68 52 L92 52 L73 66 L80 89 L60 75 L40 89 L47 66 L28 52 L52 52 Z" fill="#ffffff"/></svg>"##;

/// A moldura de uma imagem: a cor do documento, a espessura e o traço.
fn frame_style(el: &mut Element, c: Rgba, w: f64, dash: Dash, sketch: bool, round: bool) {
    let st = &mut el.image_mut().expect("imagem").style;
    (st.stroke, st.stroke_width, st.dash, st.sketch, st.round) = (Some(c), w, dash, sketch, round);
}

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let shot = file(b, &photo());
    let logo = b.put_file(crate::images::ingest(LOGO.as_bytes()).expect("o nosso SVG lê-se"));
    let (iw, ih) = (320.0, 200.0);
    // As molduras.
    let fw = 3.0 * iw + 4.0 * GAP;
    let frames = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.frames.frames"),
        [0.0, 0.0, fw, ih + 2.0 * GAP + 20.0],
        None,
    );
    // LITERAL-COLOR-OK: as cores das molduras de demonstração (dado do documento).
    let styles = [
        (Rgba([20, 20, 20, 255]), 4.0, Dash::Solid, false, false),
        (Rgba([224, 49, 49, 255]), 6.0, Dash::Dashed, false, false),
        (Rgba([25, 113, 194, 255]), 4.0, Dash::Solid, true, true),
    ];
    let framed: [ElementId; 3] = std::array::from_fn(|i| {
        let (c, w, d, sk, r) = styles[i];
        let x = GAP + i as f64 * (iw + GAP);
        image(b, shot, [x, GAP + 20.0, iw, ih], frames, |el| {
            frame_style(el, c, w, d, sk, r);
        })
    });
    // A cores e a preto e branco.
    let y2 = ih + 3.0 * GAP + 20.0;
    let bw_w = 2.0 * iw + 3.0 * GAP;
    let bw = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.frames.bw"),
        [0.0, y2, bw_w, ih + 2.0 * GAP + 20.0],
        None,
    );
    let color = image(b, shot, [GAP, y2 + GAP + 20.0, iw, ih], bw, |_| {});
    let gray = image(
        b,
        shot,
        [2.0 * GAP + iw, y2 + GAP + 20.0, iw, ih],
        bw,
        |el| {
            el.image_mut().expect("imagem").filter = ImageFilter::Gray;
        },
    );
    // O SVG, pequeno e grande.
    let sx = bw_w + GAP;
    let svg = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.frames.svg"),
        [sx, y2, fw - sx, ih + 2.0 * GAP + 20.0],
        None,
    );
    let svg_small = image(
        b,
        logo,
        [sx + GAP, y2 + GAP + 20.0, 60.0, 60.0],
        svg,
        |_| {},
    );
    let svg_big = image(
        b,
        logo,
        [sx + 2.0 * GAP + 60.0, y2 + GAP + 20.0, ih, ih],
        svg,
        |_| {},
    );
    let y = y2 + ih + 3.0 * GAP + 20.0;
    let lessons: [ElementId; 6] = std::array::from_fn(|i| {
        let key = format!("board.smoke.frames.l{}", i + 1);
        note(
            b,
            &key,
            1,
            [i as f64 * (side + GAP), y, side, side],
            false,
            None,
        )
    });
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, super::team::INK),
        text: RichText::plain(&tr("board.smoke.frames.title")),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, fw, 40.0],
    );
    put(b, el);
    let right = fw.max(6.0 * (side + GAP));
    let view = [
        -MARGIN_X,
        -80.0 - MARGIN_Y,
        right + MARGIN_X,
        y + side + MARGIN_Y,
    ];
    let l = Lessons {
        board: b.id,
        framed,
        color,
        gray,
        svg_small,
        svg_big,
        lessons,
    };
    (l, view)
}

/// O SVG de demonstração (o gate lê-o).
#[cfg(test)]
pub(crate) fn logo() -> ph2d_board_model::ImageFile {
    crate::images::ingest(LOGO.as_bytes()).expect("o nosso SVG lê-se")
}

/// A cena 16 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_frames(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.frames.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_frames_tests.rs"]
mod tests;

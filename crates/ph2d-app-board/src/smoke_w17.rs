//! Cena 18 — os DEFEITOS CONHECIDOS das imagens (W17), módulo filho do roteador [`super`]: num frame
//! «Damaged» a fotografia boa ao lado de uma cópia com o ficheiro cortado (o cabeçalho lê-se, os dados
//! não: fica riscada e o quadro avisa); num frame «Crop with Shift» uma cópia JÁ recortada alta (o
//! `Shift` num canto guarda a proporção dela); num frame «Same photo» a mesma fotografia (o PDF tem
//! três páginas com ela e descodifica-a uma vez); por baixo uma nota por lição. O gate
//! `the_w17_scene_does_what_its_lessons_say` conduz o que a cena arma.

use super::images::{file, image, photo};
use super::team::{note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Crop, Element, ElementId, RichText, STICKY_SIDE, Shape, ShapeType,
    Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub good: ElementId,
    pub damaged: ElementId,
    pub tall: ElementId,
    pub frames: [ElementId; 3],
    pub lessons: [ElementId; 4],
}

const GAP: f64 = 40.0;
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const LESSONS: usize = 4;

/// Os bytes de uma PNG com o ficheiro cortado a 60 %: o cabeçalho inteiro (o tamanho lê-se, a porta de
/// entrada aceita-a), os dados a meio (o descodificador recusa-a).
pub(crate) fn damaged_png() -> Vec<u8> {
    let mut png = crate::images::encode_png(&photo()).expect("a PNG escreve-se");
    png.truncate(png.len() * 6 / 10);
    png
}

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let shot = file(b, &photo());
    let bad =
        b.put_file(crate::images::ingest(&damaged_png()).expect("o cabeçalho da cortada lê-se"));
    let (iw, ih) = (320.0, 200.0);
    let top = GAP + 20.0;
    let fh = ih + 2.0 * GAP + 20.0;
    let dw = 2.0 * iw + 3.0 * GAP;
    let damaged_frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w17.damaged"),
        [0.0, 0.0, dw, fh],
        None,
    );
    let good = image(b, shot, [GAP, top, iw, ih], damaged_frame, |_| {});
    let damaged = image(b, bad, [2.0 * GAP + iw, top, iw, ih], damaged_frame, |_| {});
    // Uma janela 2:3 à volta do sol da fotografia 640 × 400, mostrada a 2/3 de escala.
    let (tw, th) = (ih * 2.0 / 3.0, ih);
    let cx = dw + GAP;
    let crop_frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w17.crop"),
        [cx, 0.0, tw + 2.0 * GAP, fh],
        None,
    );
    let tall = image(b, shot, [cx + GAP, top, tw, th], crop_frame, |el| {
        el.image_mut().expect("imagem").crop = Some(Crop {
            x: 380.0,
            y: 40.0,
            w: 200.0,
            h: 300.0,
        });
    });
    let sx = cx + tw + 3.0 * GAP;
    let same_frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w17.same"),
        [sx, 0.0, iw + 2.0 * GAP, fh],
        None,
    );
    let _ = image(b, shot, [sx + GAP, top, iw, ih], same_frame, |_| {});
    let y = fh + 2.0 * GAP;
    let lessons: [ElementId; LESSONS] = std::array::from_fn(|i| {
        let key = format!("board.smoke.w17.l{}", i + 1);
        note(
            b,
            &key,
            1,
            [i as f64 * (side + GAP), y, side, side],
            false,
            None,
        )
    });
    let right = (sx + iw + 2.0 * GAP).max(LESSONS as f64 * (side + GAP));
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, super::team::INK),
        text: RichText::plain(&tr("board.smoke.w17.title")),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, right, 40.0],
    );
    put(b, el);
    let view = [
        -MARGIN_X,
        -80.0 - MARGIN_Y,
        right + MARGIN_X,
        y + side + MARGIN_Y,
    ];
    let l = Lessons {
        board: b.id,
        good,
        damaged,
        tall,
        frames: [damaged_frame, crop_frame, same_frame],
        lessons,
    };
    (l, view)
}

/// A cena 18 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_w17(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.w17.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_w17_tests.rs"]
mod tests;

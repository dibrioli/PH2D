//! Cena 17 — RECORTAR, ESPELHAR e o GIF (W16), módulo filho do roteador [`super`]: num frame «Crop»
//! a fotografia inteira (para recortar) ao lado de uma cópia já recortada; num frame «Flip» a mesma
//! fotografia e as cópias espelhadas na horizontal e na vertical; num frame «GIF» uma bola que anda (um
//! GIF de oito quadros); por baixo uma nota por lição — o colar do Dolphin, o «Send to board…» da
//! selecção e o export SVG. O gate `the_w16_scene_does_what_its_lessons_say` conduz o que a cena arma.

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
    pub whole: ElementId,
    pub cropped: ElementId,
    /// Original, espelhada na horizontal, na vertical.
    pub flips: [ElementId; 3],
    pub gif: ElementId,
    pub lessons: [ElementId; 6],
}

const GAP: f64 = 40.0;
/// Os quadros do GIF de demonstração e a duração de cada um.
const GIF_FRAMES: u32 = 8;
const GIF_MS: u32 = 120;

/// Um GIF 160 × 100 de [`GIF_FRAMES`] quadros: uma bola que atravessa o céu.
// LITERAL-COLOR-OK: píxeis de uma animação de demonstração (dado do documento, não da UI).
pub(crate) fn ball_gif() -> Vec<u8> {
    use image::{Delay, Frame, Rgba, RgbaImage};
    let (w, h) = (160u32, 100u32);
    let mut out = Vec::new();
    {
        let mut enc = image::codecs::gif::GifEncoder::new(&mut out);
        enc.set_repeat(image::codecs::gif::Repeat::Infinite)
            .expect("o GIF escreve-se");
        for k in 0..GIF_FRAMES {
            let cx = 20.0 + f64::from(k) * (f64::from(w) - 40.0) / f64::from(GIF_FRAMES - 1);
            let img = RgbaImage::from_fn(w, h, |x, y| {
                let (dx, dy) = (f64::from(x) - cx, f64::from(y) - 50.0);
                if dx.hypot(dy) <= 16.0 {
                    Rgba([224, 49, 49, 255])
                } else {
                    Rgba([165, 216, 255, 255])
                }
            });
            enc.encode_frame(Frame::from_parts(
                img,
                0,
                0,
                Delay::from_numer_denom_ms(GIF_MS, 1),
            ))
            .expect("um quadro");
        }
    }
    out
}

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let shot = file(b, &photo());
    let gif = b.put_file(crate::images::ingest(&ball_gif()).expect("o nosso GIF lê-se"));
    let (iw, ih) = (320.0, 200.0);
    let fw = 2.0 * iw + 3.0 * GAP;
    let crop = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w16.crop"),
        [0.0, 0.0, fw, ih + 2.0 * GAP + 20.0],
        None,
    );
    let whole = image(b, shot, [GAP, GAP + 20.0, iw, ih], crop, |_| {});
    // A metade de cima à direita da fotografia 640 × 400 (o sol), na proporção dela.
    let cropped = image(b, shot, [2.0 * GAP + iw, GAP + 20.0, iw, ih], crop, |el| {
        el.image_mut().expect("imagem").crop = Some(Crop {
            x: 320.0,
            y: 0.0,
            w: 320.0,
            h: 200.0,
        });
    });
    let y2 = ih + 3.0 * GAP + 20.0;
    let ffw = 3.0 * iw + 4.0 * GAP;
    let flip = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w16.flip"),
        [0.0, y2, ffw, ih + 2.0 * GAP + 20.0],
        None,
    );
    let flips: [ElementId; 3] = std::array::from_fn(|i| {
        let x = GAP + i as f64 * (iw + GAP);
        image(b, shot, [x, y2 + GAP + 20.0, iw, ih], flip, |el| {
            el.image_mut().expect("imagem").flip = [i == 1, i == 2];
        })
    });
    let gx = fw + GAP;
    let gfr = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w16.gif"),
        [gx, 0.0, iw + 2.0 * GAP, ih + 2.0 * GAP + 20.0],
        None,
    );
    let gif = image(b, gif, [gx + GAP, GAP + 20.0, iw, ih], gfr, |_| {});
    let y = y2 + ih + 3.0 * GAP + 20.0;
    let lessons: [ElementId; 6] = std::array::from_fn(|i| {
        let key = format!("board.smoke.w16.l{}", i + 1);
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
        text: RichText::plain(&tr("board.smoke.w16.title")),
    };
    let right = ffw.max(6.0 * (side + GAP)).max(gx + iw + 2.0 * GAP);
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, right, 40.0],
    );
    put(b, el);
    let view = [0.0, -80.0, right, y + side];
    let l = Lessons {
        board: b.id,
        whole,
        cropped,
        flips,
        gif,
        lessons,
    };
    (l, view)
}

/// A cena 17 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_w16(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.w16.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_w16_tests.rs"]
pub(crate) mod tests;

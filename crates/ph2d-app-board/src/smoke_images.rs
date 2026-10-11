//! Cena 15 — AS IMAGENS NO QUADRO (W14), módulo filho do roteador [`super`]: um frame «Moodboard» com
//! uma fotografia (gerada aqui: céu, sol, colinas), a MESMA recortada ao sol e espelhada, a mesma
//! redonda, a ¾ e rodada, um ícone com transparência, e uma nota presa à fotografia por uma seta; por
//! baixo uma nota por lição (soltar · redimensionar · a barra · colar · copiar para outro quadro ·
//! exportar). O gate `the_images_scene_does_what_its_lessons_say` conduz o que a cena arma.

use super::team::{INK, note, put, tr};
use ph2d_board_model::{
    Anchor, Board, BoardId, BoardSet, Connector, Crop, Element, ElementId, End, FileId, Image,
    RichText, Route, STICKY_SIDE, Shape, ShapeType, Style,
};
use ph2d_board_render::Pixels;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub photo: ElementId,
    pub sun: ElementId,
    pub round: ElementId,
    pub icon: ElementId,
    pub arrow: ElementId,
    pub lessons: [ElementId; 6],
}

const GAP: f64 = 40.0;
/// A margem do que os painéis tapam (a da cena 14, em unidades do mundo).
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const PHOTO: [u32; 2] = [640, 400];

/// A fotografia de demonstração: céu em degradê, um sol, duas colinas.
// LITERAL-COLOR-OK: píxeis de uma fotografia de demonstração (dado do documento, não da UI).
pub(crate) fn photo() -> Pixels {
    let [w, h] = PHOTO;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (f64::from(x), f64::from(y));
            let t = fy / f64::from(h);
            let mut c = [
                (110.0 + 120.0 * t) as u8,
                (170.0 + 70.0 * t) as u8,
                (235.0 + 15.0 * t) as u8,
            ];
            if (fx - 480.0).hypot(fy - 130.0) < 60.0 {
                c = [255, 205, 70];
            }
            let hill = |base: f64, amp: f64, k: f64| base + amp * (fx / k).sin();
            if fy > hill(300.0, 25.0, 70.0) {
                c = [70, 150, 80];
            }
            if fy > hill(340.0, 15.0, 45.0) {
                c = [45, 115, 60];
            }
            rgba.extend([c[0], c[1], c[2], 255]);
        }
    }
    Pixels { size: PHOTO, rgba }
}

/// O ícone com transparência: um disco laranja de borda suave e um anel, sobre nada.
// LITERAL-COLOR-OK: píxeis de um ícone de demonstração (dado do documento, não da UI).
fn icon() -> Pixels {
    let s = 256u32;
    let mut rgba = Vec::with_capacity((s * s * 4) as usize);
    for y in 0..s {
        for x in 0..s {
            let d = (f64::from(x) - 128.0).hypot(f64::from(y) - 128.0);
            let disc = (100.0 - d).clamp(0.0, 1.0); // CLAMP-OK: cobertura em [0, 1]
            let ring = (6.0 - (d - 118.0).abs()).clamp(0.0, 1.0); // CLAMP-OK: cobertura em [0, 1]
            let a = disc.max(ring);
            let c = if ring > disc { [40, 40, 40] } else { [245, 130, 40] };
            rgba.extend([c[0], c[1], c[2], (a * 255.0).round() as u8]);
        }
    }
    Pixels {
        size: [s, s],
        rgba,
    }
}

fn file(b: &mut Board, px: &Pixels) -> FileId {
    let png = crate::images::encode_png(px).expect("a PNG escreve-se");
    b.put_file(crate::images::ingest(&png).expect("a nossa PNG lê-se"))
}

fn image(b: &mut Board, file: FileId, bx: [f64; 4], home: ElementId, edit: impl Fn(&mut Element)) -> ElementId {
    let img = Image::new(file, Style::new(None, None, INK));
    let mut el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, bx);
    el.frame = Some(home);
    edit(&mut el);
    put(b, el)
}

/// Monta o moodboard em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let (fw, fh) = (1500.0, 560.0);
    let frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.images.frame"),
        [0.0, 0.0, fw, fh],
        None,
    );
    let shot = file(b, &photo());
    let logo = file(b, &icon());
    let photo = image(b, shot, [GAP, 60.0, 640.0, 400.0], frame, |_| {});
    let sun = image(b, shot, [720.0, 60.0, 200.0, 200.0], frame, |el| {
        let i = el.image_mut().expect("imagem");
        i.crop = Some(Crop {
            x: 380.0,
            y: 30.0,
            w: 200.0,
            h: 200.0,
        });
        i.flip = [true, false];
    });
    let round = image(b, shot, [980.0, 70.0, 480.0, 300.0], frame, |el| {
        el.angle = -0.12;
        let i = el.image_mut().expect("imagem");
        (i.style.round, i.style.opacity) = (true, 75);
    });
    let icon = image(b, logo, [740.0, 300.0, 160.0, 160.0], frame, |_| {});
    let cover = note(b, "board.smoke.images.cover", 1, [1060.0, 400.0, 150.0, 150.0], false, Some(frame));
    let mut c = Connector::new(
        End::Bound {
            target: cover,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: photo,
            anchor: Anchor::Center,
        },
        Route::Curved,
        Style::new(None, Some(INK), INK),
    );
    c.style.stroke_width = 2.0;
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    let arrow = put(b, el);
    let y = fh + GAP;
    let lessons: [ElementId; 6] = std::array::from_fn(|i| {
        let key = format!("board.smoke.images.l{}", i + 1);
        note(b, &key, 1, [i as f64 * (side + GAP), y, side, side], false, None)
    });
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(&tr("board.smoke.images.title")),
    };
    let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), title, [0.0, -80.0, fw, 40.0]);
    put(b, el);
    let view = [-MARGIN_X, -80.0 - MARGIN_Y, fw + MARGIN_X, y + side + MARGIN_Y];
    let l = Lessons {
        board: b.id,
        photo,
        sun,
        round,
        icon,
        arrow,
        lessons,
    };
    (l, view)
}

/// A cena 15 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_images(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.images.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_images_tests.rs"]
mod tests;

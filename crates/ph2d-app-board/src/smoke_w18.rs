//! Cena 19 — o VÍDEO do YouTube e os abertos das imagens (W18), módulo filho do roteador [`super`]: num
//! frame «YouTube» um vídeo (o Big Buck Bunny da Blender, o das medições do ADR-0185); num frame «GIF» a
//! bola da cena 17 (a apresentação toca-a); num frame «SVG text» um SVG com letras — uma família que este
//! computador tem e uma que não tem (o aviso nomeia-a); num frame «Cut picture» a fotografia cortada a
//! 60 % (mostra as linhas de cima, com a marca); por baixo uma nota por lição. O gate
//! `the_w18_scene_does_what_its_lessons_say` conduz o que a cena arma.

use super::images::{file, image, photo};
use super::team::{note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Element, ElementId, Embed, Rgba, RichText, STICKY_SIDE, Shape,
    ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub video: ElementId,
    pub gif: ElementId,
    pub svg: ElementId,
    pub cut: ElementId,
    pub frames: [ElementId; 4],
    pub lessons: [ElementId; 5],
}

const GAP: f64 = 40.0;
const LESSONS: usize = 5;

/// O vídeo da cena (Big Buck Bunny, Blender — o das medições).
pub(crate) const LINK: &str = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

/// Um SVG 320 × 200 com letras: «Hello» na DejaVu Sans (instalada), «board» numa família que não existe
/// (cai na de substituição, e o aviso nomeia-a).
pub(crate) const SVG_TEXT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200">
<rect width="320" height="200" rx="16" fill="#f4f1e8"/>
<text x="160" y="90" font-family="DejaVu Sans" font-size="56" font-weight="bold" text-anchor="middle" fill="#1e3a8a">Hello</text>
<text x="160" y="160" font-family="No Such Font PH2D" font-size="40" text-anchor="middle" fill="#b91c1c">board</text>
</svg>"##;

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let video = ph2d_board_embed::youtube(LINK).expect("o link da cena é um vídeo");
    let [vw, vh] = ph2d_board_embed::natural_size(&video);
    let top = GAP + 20.0;
    let fh = vh + 2.0 * GAP + 20.0;
    let video_frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w18.youtube"),
        [0.0, 0.0, vw + 2.0 * GAP, fh],
        None,
    );
    let mut st = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
    st.round = true;
    let e = Embed {
        link: LINK.to_owned(),
        video,
        style: st,
    };
    let mut v = Element::new_embed(b.doc.mint_id(), b.doc.z_on_top(), e, [GAP, top, vw, vh]);
    v.frame = Some(video_frame);
    let video_id = v.id;
    put(b, v);
    // O GIF, o SVG e a cortada, cada um no seu frame, à direita.
    let (iw, ih) = (320.0, 200.0);
    let mut x = vw + 3.0 * GAP;
    let mut framed = |b: &mut Board, key: &str, bytes: &[u8]| {
        let f = super::organize::frame(&mut b.doc, &tr(key), [x, 0.0, iw + 2.0 * GAP, fh], None);
        let file = b.put_file(crate::images::ingest(bytes).expect("a imagem da cena entra"));
        let id = image(b, file, [x + GAP, top, iw, ih], f, |_| {});
        x += iw + 3.0 * GAP;
        (f, id)
    };
    let (gif_frame, gif) = framed(b, "board.smoke.w18.gif", &super::w16::ball_gif());
    let (svg_frame, svg) = framed(b, "board.smoke.w18.svg", SVG_TEXT.as_bytes());
    let (cut_frame, cut) = framed(b, "board.smoke.w18.cut", &super::w17::damaged_png());
    let _ = file(b, &photo());
    let right = x - GAP;
    let y = fh + 2.0 * GAP;
    let lessons: [ElementId; LESSONS] = std::array::from_fn(|i| {
        let key = format!("board.smoke.w18.l{}", i + 1);
        note(
            b,
            &key,
            1,
            [i as f64 * (side + GAP), y, side, side],
            false,
            None,
        )
    });
    let right = right.max(LESSONS as f64 * (side + GAP));
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, super::team::INK),
        text: RichText::plain(&tr("board.smoke.w18.title")),
    };
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
        video: video_id,
        gif,
        svg,
        cut,
        frames: [video_frame, gif_frame, svg_frame, cut_frame],
        lessons,
    };
    (l, view)
}

/// A cena 19 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_w18(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.w18.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_w18_tests.rs"]
mod tests;

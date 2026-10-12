//! ⭐ A cena 19 ARMA o que as lições dizem: o play do vídeo activa-o pelo rato real e o tocador fica vivo
//! (inteiro à vista, descoberto, ≥ 200 px); uma nota por cima tira-o de vivo; colar um link faz outro
//! vídeo; as letras do SVG desenham-se e a família que falta é nomeada; a fotografia cortada mostra as
//! linhas de cima e o quadro avisa.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_model::BoardOp;

fn embeds(h: &HeroScreen) -> usize {
    let b = h.documents.active_board().unwrap();
    b.doc.live().filter(|e| e.embed().is_some()).count()
}

#[test]
fn the_w18_scene_does_what_its_lessons_say() {
    crate::images::install();
    let mut h = hero("Ana");
    let l = scene_w18(&mut h);
    Hand::new().paint(&mut h);
    let (el, b_id) = {
        let b = h.documents.active_board().unwrap();
        (b.doc.get(l.video).unwrap().clone(), b.id)
    };
    assert_eq!(b_id, l.board);
    assert_eq!(el.embed().unwrap().link, LINK);
    assert!(
        h.documents.videos_mut().on_screen.contains_key(&l.video),
        "o vídeo está à vista"
    );
    // Lição 1: o play (o meio do vídeo) activa-o, e ele fica vivo.
    let mid = [el.x + el.w / 2.0, el.y + el.h / 2.0];
    Mouse::new().click(&mut h, mid);
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.videos_mut().active, Some(l.video));
    let pl = h.documents.videos_mut().on_screen[&l.video];
    assert!(pl.live, "vivo na vista da cena: {:?}", pl.screen);
    // Lição 2: uma nota POR CIMA tira-o de vivo (as regras do YouTube).
    {
        let b = h.documents.active_board_mut().unwrap();
        let s = Shape {
            kind: ShapeType::Sticky,
            style: Style::new(None, None, super::super::team::INK),
            text: RichText::default(),
        };
        let n = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            s,
            [el.x + 10.0, el.y + 10.0, 80.0, 80.0],
        );
        let _ = BoardOp::Put(n).apply(&mut b.doc);
    }
    Hand::new().paint(&mut h);
    assert!(
        !h.documents.videos_mut().on_screen[&l.video].live,
        "tapado não está vivo"
    );
    // Lição 3: colar um link faz OUTRO vídeo.
    let before = embeds(&h);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let ctrl = ph2d_host::Modifiers {
        ctrl: true,
        ..ph2d_host::Modifiers::default()
    };
    use ph2d_editor_core::screens::hero::board_keys::{BoardKey, key};
    h.documents.videos_mut().active = None;
    let out = key(
        &mut h,
        &mut ts,
        BoardKey::Char('v'),
        ctrl,
        true,
        None,
        Some("https://youtu.be/aqz-KE-bpKQ?t=30".into()),
    );
    assert!(out.consumed);
    assert_eq!(embeds(&h), before + 1);
    // Lição 4: as letras do SVG desenham-se (a tinta azul do «Hello») e o aviso nomeia a família que falta.
    let px = crate::images::decode(SVG_TEXT.as_bytes(), None).expect("o SVG desenha-se");
    let blue = px
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[2] > 100 && p[0] < 80 && p[1] < 100)
        .count();
    assert!(blue > 200, "o «Hello» azul: {blue} píxeis");
    let gaps = ph2d_imageio_svg::raster::text_gaps(SVG_TEXT.as_bytes());
    assert_eq!(gaps.missing_families, vec!["No Such Font PH2D".to_owned()]);
    // Lição 5: a cortada mostra as linhas de cima, e o quadro avisa que está estragada.
    let saved =
        crate::salvage::salvage(&super::super::w17::damaged_png()).expect("as linhas de cima");
    let rows = (0..saved.size[1])
        .take_while(|&y| saved.rgba[((y * saved.size[0]) * 4 + 3) as usize] != 0)
        .count();
    assert!(
        rows > 0 && rows < saved.size[1] as usize,
        "{rows} linhas lidas"
    );
    let t0 = std::time::Instant::now();
    let mut warned = None;
    while t0.elapsed().as_secs() < 20 {
        Hand::new().paint(&mut h);
        warned = warned.or(h.documents.notice());
        if !h.documents.images_pending() && warned == Some("board.images.unreadable") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(warned, Some("board.images.unreadable"));
    let _ = (l.gif, l.svg, l.cut, l.frames, l.lessons);
}

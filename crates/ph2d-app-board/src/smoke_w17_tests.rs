//! ⭐ A cena 18 ARMA o que as lições dizem: a fotografia cortada fica riscada e o quadro avisa (com o
//! descodificador do APP), o `Shift` num canto da recortada alta guarda a proporção DELA pelo rato
//! real, e a mesma fotografia vai em três frames (três páginas do PDF).

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_model::Image;

fn img(h: &HeroScreen, id: ElementId) -> (Element, Image) {
    let b = h.documents.active_board().unwrap();
    let el = b.doc.get(id).unwrap().clone();
    let i = el.image().unwrap().clone();
    (el, i)
}

#[test]
fn the_w17_scene_does_what_its_lessons_say() {
    crate::images::install();
    assert!(
        crate::images::decode(&damaged_png(), None).is_none(),
        "a cortada lê-se — a lição 1 não mostraria nada"
    );
    let mut h = hero("Ana");
    let l = scene_w17(&mut h);
    // Lição 1: pinta até as duas imagens assentarem; a cortada avisa.
    let t0 = std::time::Instant::now();
    let mut warned = None;
    while t0.elapsed().as_secs() < 20 {
        Hand::new().paint(&mut h);
        warned = warned.or(h.documents.notice());
        if !h.documents.images_pending() && warned.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(warned, Some("board.images.unreadable"));
    let b = h.documents.active_board().unwrap();
    assert_eq!(b.id, l.board);
    assert_eq!(b.files.len(), 2, "a fotografia e a cortada, uma vez cada");
    assert_ne!(img(&h, l.good).1.file, img(&h, l.damaged).1.file);
    // Lição 3: a mesma fotografia em cada um dos três frames.
    let shot = img(&h, l.good).1.file;
    for f in l.frames {
        assert!(
            b.doc
                .live()
                .any(|e| e.frame == Some(f) && e.image().is_some_and(|i| i.file == shot)),
            "um frame sem a fotografia"
        );
    }
    assert_eq!(l.lessons.len(), 4);
    // Lição 2 pelo rato REAL: duplo-clique na recortada alta, e o canto de baixo à direita puxado
    // para dentro com `Shift` — a janela continua 2:3 (antes saltava para os 16:10 da fotografia).
    let (tall, _) = img(&h, l.tall);
    let mut hand = Mouse::new();
    let mid = [tall.x + tall.w / 2.0, tall.y + tall.h / 2.0];
    hand.click(&mut h, mid);
    hand.click(&mut h, mid);
    hand.mods.shift = true;
    let corner = [tall.x + tall.w, tall.y + tall.h];
    hand.drag(
        &mut h,
        corner,
        [tall.x + tall.w * 0.8, tall.y + tall.h * 0.9],
    );
    let crop = img(&h, l.tall).1.crop.expect("recortada");
    assert!(crop.h < 299.0, "o arrasto não recortou: {crop:?}");
    assert!(
        (crop.w / crop.h - 2.0 / 3.0).abs() < 1e-6,
        "o Shift não guardou a proporção do recorte: {crop:?}"
    );
}

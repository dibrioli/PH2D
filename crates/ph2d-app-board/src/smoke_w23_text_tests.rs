//! ⭐ A cena 26 ARMA o que a lição 1 diz, pelo rato REAL sobre o quadro (a porta da shell): a seta
//! curva abre seleccionada com «next step» a 3/10 da linha, e arrastar as letras para a frente leva-as
//! ao longo da linha — a linha não muda.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_route::RouteCache;

fn caption(h: &HeroScreen, arrow: ElementId) -> (f64, [f64; 2], Vec<[f64; 2]>) {
    let b = h.documents.active_board().unwrap();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let r = routes.get(arrow).unwrap();
    let c = b.doc.get(arrow).unwrap().connector().unwrap();
    let f = r.caption_frame(c.captions[0].at, c.turn);
    (c.captions[0].at, f.center, r.polyline())
}

#[test]
fn the_w23_text_scene_opens_with_the_words_in_reach() {
    let mut h = hero("Ana");
    let arrow = scene_text(&mut h);
    Hand::new().paint(&mut h);
    let (at, center, line) = caption(&h, arrow);
    assert_eq!(at, FIRST_AT);
    // Ao lado do centro, longe da bolinha do meio da linha.
    let grab = center;
    let b = h.documents.active_board().unwrap();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let (ahead, _) = routes.get(arrow).unwrap().point_at(0.6);
    Mouse::new().drag(&mut h, grab, ahead);
    let (now, moved, after) = caption(&h, arrow);
    assert!((now - 0.6).abs() < 0.02, "pelo rato: {now}");
    assert!(
        (moved[0] - ahead[0]).hypot(moved[1] - ahead[1]) < 2.0,
        "{moved:?} ≠ {ahead:?}"
    );
    assert_eq!(after, line, "a linha não muda");
}

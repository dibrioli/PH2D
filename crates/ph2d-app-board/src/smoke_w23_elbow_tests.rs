//! ⭐ A cena 25 ARMA o que a lição 1 diz, pelo rato REAL sobre o quadro (a porta da shell): a seta
//! abre seleccionada com o segmento do meio em x = 250, e arrastar a pílula 60 px para o lado leva-o
//! a x = 310 sem criar ponto de ajuste.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use crate::smoke::w16::tests::Mouse;
use ph2d_board_route::RouteCache;

fn middle(h: &HeroScreen, arrow: ElementId) -> [f64; 2] {
    let b = h.documents.active_board().unwrap();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    routes.get(arrow).unwrap().elbows[0].expect("um Z").at
}

#[test]
fn the_w23_elbow_scene_opens_with_the_middle_bar_in_reach() {
    let mut h = hero("Ana");
    let arrow = scene_elbow(&mut h);
    Hand::new().paint(&mut h);
    let at = middle(&h, arrow);
    assert!((at[0] - 250.0).abs() < 1e-6, "{at:?}");
    Mouse::new().drag(&mut h, at, [at[0] + 60.0, at[1]]);
    let now = middle(&h, arrow);
    assert!((now[0] - 310.0).abs() < 1.0, "pelo rato: {now:?}");
    let b = h.documents.active_board().unwrap();
    let c = b.doc.get(arrow).unwrap().connector().unwrap();
    assert!(c.waypoints.is_empty() && c.mid(0).is_some());
}

//! ⭐ A cena 28 ARMA as quatro estações: o laço à volta do frame ① leva as seis ideias e a «quase toda
//! dentro», não a «meio fora»; o aviso do Rui está no topo ao abrir, com a bolinha; o «Plan» existe e
//! as caixas de ④ estão ligadas pela seta.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};

#[test]
fn the_block6_scene_arms_its_four_stations() {
    let mut h = hero("Ana");
    let s = scene_block6(&mut h);
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.active(), Some(s.board));
    assert!(h.documents.boards().get(s.plan).is_some(), "o Plan por trás");
    // ③ — o aviso do Rui chegou ao abrir, e a bolinha do quadro.
    let alert = h.documents.live_alert_text().expect("o aviso do Rui");
    assert!(alert.starts_with("Rui mentioned you"), "{alert}");
    assert!(notify::board_has_news(&h.documents, s.board));
    // ① — o laço à volta do frame (um contorno um pouco por fora dele).
    let (x0, y0, x1, y1) = (F1[0] - 10.0, F1[1] - 10.0, F1[0] + F1[2] + 10.0, F1[1] + F1[3] + 10.0);
    let lasso = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
    let taken = h.documents.lasso_for_tests(&lasso);
    for id in &s.ideas {
        assert!(taken.contains(id), "a ideia {id:?} ficou de fora");
    }
    assert!(taken.contains(&s.most), "95 % dentro entra");
    assert!(!taken.contains(&s.half), "metade fora não entra");
    // ④ — a seta liga as duas caixas.
    let doc = &h.documents.boards().get(s.board).unwrap().doc;
    let c = doc.get(s.send[2]).and_then(|e| e.connector().cloned()).expect("a seta");
    assert!(matches!(c.start, End::Bound { target, .. } if target == s.send[0]));
    assert!(matches!(c.end, End::Bound { target, .. } if target == s.send[1]));
}

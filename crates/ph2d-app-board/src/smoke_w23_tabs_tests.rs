//! ⭐ A cena 27 ARMA o que a lição 1 diz: apagar o «Archive» e carregar `Ctrl+Z` no quadro aberto pela
//! porta do teclado devolve-o no mesmo lugar, com a caixa dele, sem mudar a aba aberta.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_host::Modifiers;

#[test]
fn the_w23_tabs_scene_brings_the_archive_back_with_ctrl_z() {
    let mut h = hero("Ana");
    let ids = scene_tabs(&mut h);
    Hand::new().paint(&mut h);
    assert_eq!(
        h.documents.active(),
        Some(ids[1]),
        "abre no Plan, com as lições"
    );
    let boxes =
        |h: &HeroScreen, i: usize| h.documents.boards().get(ids[i]).map(|b| b.doc.live_len());
    assert_eq!(boxes(&h, 1), Some(1 + LESSONS.len()));
    assert_eq!(boxes(&h, 2), Some(1));
    assert!(h.documents.remove(ids[2]));
    assert_eq!(boxes(&h, 2), None);

    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mods = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let z = BoardKey::Char('z');
    assert!(board_keys::key(&mut h, &mut ts, z, mods, true, None, None).consumed);
    let order: Vec<_> = h.documents.boards().boards().iter().map(|b| b.id).collect();
    assert_eq!(order, ids, "no mesmo lugar");
    assert_eq!(boxes(&h, 2), Some(1), "com a caixa dele");
    assert_eq!(h.documents.active(), Some(ids[1]), "a aba aberta não muda");
}

//! Cena 31 — D17 do Kanban (dono 30/10): Quadros e Kanbans são documentos do projecto e o painel de
//! **Assets** lista-os (abrir com duplo-clique, arrastar um Kanban para o Quadro, apagar pelo botão
//! direito). A cena 30 (o Kanban no «Production board») mais um Kanban VAZIO — que fica no projecto
//! até ser apagado — e um 2.º Quadro, com os Assets abertos.

use ph2d_editor_core::HeroScreen;
use ph2d_i18n::tr;

/// O `PANEL_ID` do `ph2d-panel-asset-browser` (esta crate não depende do painel).
const ASSETS: &str = "asset_browser";

pub(crate) fn scene_d17_assets(hero: &mut HeroScreen) {
    let s = super::w4_kanban::scene_w4_kanban(hero);
    let shared = hero.documents.kanbans_shared();
    shared
        .borrow_mut()
        .create(tr("board.smoke.d17.empty_kanban"));
    let ideas = hero
        .documents
        .create(tr("board.smoke.d17.second_board").to_owned());
    ph2d_editor_core::screens::hero::document_tabs::register_board(&mut hero.store, ideas);
    hero.documents.activate(Some(s.board));
    hero.panel_visibility.insert(ASSETS, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::NodeId;
    use ph2d_editor_core::interaction::drag_payload::DragPayload;

    #[test]
    fn the_d17_scene_lists_two_kanbans_and_two_boards_with_the_assets_open() {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut h = HeroScreen::new(NodeId(1));
        scene_d17_assets(&mut h);
        let all = h.documents.asset_listing();
        let kanbans = all
            .iter()
            .filter(|(p, _)| matches!(p, DragPayload::Kanban { .. }))
            .count();
        let boards = all
            .iter()
            .filter(|(p, _)| matches!(p, DragPayload::Board { .. }))
            .count();
        assert_eq!((kanbans, boards), (2, 2), "{all:?}");
        assert!(
            all.iter()
                .any(|(_, n)| n == tr("board.smoke.d17.empty_kanban")),
            "o Kanban vazio também está lá"
        );
        assert_eq!(h.panel_visibility.get(ASSETS), Some(&true));
        assert!(h.documents.active_board().is_some(), "o Quadro à frente");
    }
}

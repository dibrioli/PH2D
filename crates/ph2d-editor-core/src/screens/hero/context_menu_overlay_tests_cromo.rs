//! A pele do menu troca com `PH2D_CARDS`; os alvos não (`docs/CartoesUI/15_CHROME_CENSO.md`, M1).

use super::super::context_menu_overlay::paint_context_menu_overlay;
use crate::ids;
use crate::interaction::{ContextMenuKind as K, ContextMenuRequest, HitIndex, WidgetStore};
use crate::motion::UiMotion;
use crate::paint::{fill_rounded_rect, resolve};
use crate::project::ProjectSettings;
use crate::widget::lista_flutuante::force_cromo_on_this_thread;
use crate::widget::{Tooltip, paint_tooltip};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

const TEMA: Theme = Theme::Dark;

fn janela() -> Rect {
    Rect::new(0.0, 0.0, 1930.0, 1040.0)
}

fn menu(kind: K, hot: Option<NodeId>, on: bool) -> (VectorScene, HitIndex) {
    force_cromo_on_this_thread(Some(on));
    let mut store = WidgetStore::default();
    store.open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 60.0,
        kind,
    });
    store.set_hot(hot);
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    paint_context_menu_overlay(
        &mut scene,
        &mut TextSystem::without_system_fonts(),
        TEMA,
        &mut hits,
        &store,
        &ProjectSettings::default(),
        &UiMotion::default(),
        &crate::luz_edits::PreferenciaDaLuz::default(),
        janela(),
    );
    force_cromo_on_this_thread(None);
    (scene, hits)
}

/// A cena traz um preenchimento nesta cor?
fn pinta_a_cor(scene: &VectorScene, token: ColorToken) -> bool {
    let mut uma = VectorScene::new();
    let r = Rect::new(0.0, 0.0, 1.0, 1.0);
    fill_rounded_rect(&mut uma, r, 0.0, resolve(token, TEMA).with_alpha(1.0));
    let cor = uma.inner().encoding().draw_data.clone();
    scene
        .inner()
        .encoding()
        .draw_data
        .windows(cor.len())
        .any(|w| w == cor.as_slice())
}

#[test]
fn cada_linha_de_cada_menu_regista_o_mesmo_alvo_nos_dois_modos() {
    let mut com_alvos = 0;
    for kind in [
        K::HierarchyRow { row: NodeId(1) },
        K::MenuBarFile,
        K::MenuBarView,
        K::SettingsMenu,
        K::SceneList,
        K::ThemeSelector,
    ] {
        let rects = |on| {
            let (_, hits) = menu(kind, None, on);
            hits.iter_registrations().collect::<Vec<_>>()
        };
        let (novos, antigos) = (rects(true), rects(false));
        assert_eq!(novos, antigos, "{kind:?}: o cromo moveu um alvo");
        com_alvos += usize::from(!novos.is_empty());
    }
    assert_eq!(com_alvos, 6, "um menu sem alvos não prova nada");
}

#[test]
fn o_menu_em_cartoes_pinta_a_superficie_da_lista_e_o_de_antes_a_antiga() {
    let kind = K::HierarchyRow { row: NodeId(1) };
    let (novo, _) = menu(kind, None, true);
    assert!(pinta_a_cor(&novo, ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&novo, ColorToken::BgElev));
    let (antigo, _) = menu(kind, None, false);
    assert!(pinta_a_cor(&antigo, ColorToken::BgElev));
    assert!(!pinta_a_cor(&antigo, ColorToken::KanbanCardRaised));
}

#[test]
fn a_linha_sob_o_rato_acende_no_tom_do_campo_do_cartao() {
    let kind = K::HierarchyRow { row: NodeId(1) };
    let hot = Some(ids::CTX_MENU_HIER_DUPLICATE);
    assert!(!pinta_a_cor(&menu(kind, None, true).0, ColorToken::CardFieldHover));
    assert!(pinta_a_cor(&menu(kind, hot, true).0, ColorToken::CardFieldHover));
    assert!(pinta_a_cor(&menu(kind, hot, false).0, ColorToken::Bg2));
}

#[test]
fn a_lista_de_cenas_e_o_transbordo_vestem_a_mesma_superficie() {
    let (cenas, _) = menu(K::SceneList, None, true);
    assert!(pinta_a_cor(&cenas, ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&cenas, ColorToken::BgElev));
}

#[test]
fn a_dica_em_cartoes_nao_e_a_antiga() {
    let dica = |on| {
        force_cromo_on_this_thread(Some(on));
        let mut scene = VectorScene::new();
        paint_tooltip(
            &Tooltip::new(NodeId(1), "Save"),
            Rect::new(0.0, 0.0, 120.0, 24.0),
            &mut scene,
            &mut TextSystem::without_system_fonts(),
            TEMA,
        );
        force_cromo_on_this_thread(None);
        scene
    };
    assert!(pinta_a_cor(&dica(true), ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&dica(true), ColorToken::Bg3));
    assert!(pinta_a_cor(&dica(false), ColorToken::Bg3));
}

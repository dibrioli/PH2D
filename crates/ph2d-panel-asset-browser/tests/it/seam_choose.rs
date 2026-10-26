//! ⭐ **O SELETOR DE IMAGEM** (plano GEF 12): o shell pede o seletor para uma sprite ⇒ o painel pinta só
//! imagens com a faixa ⇒ um clique numa imagem emite a escolha para AQUELA sprite e fecha; o `✕` desiste.

use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_asset_browser::state::AssetBrowserState;
use ph2d_panel_asset_browser::{AssetBrowserPanel, PANEL_ID, ids};
use ph2d_ui_testkit::MockPanelHost;

const SPRITE: u64 = 0x5107_0042;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn publish() {
    let mut ix = AssetIndex::new();
    ix.push(AssetEntry::new(
        AssetRef::Component { stable_id: 9 },
        "a house",
    ));
    ix.push(AssetEntry::new(
        AssetRef::Texture { asset: [1; 32] },
        "hills",
    ));
    ix.push(AssetEntry::new(
        AssetRef::Texture { asset: [2; 32] },
        "sunset",
    ));
    ph2d_panel_asset_browser::set_current_index(ix);
}

fn painted() -> Vec<AssetRef> {
    (0..8)
        .map_while(ph2d_panel_asset_browser::probe_painted_at)
        .collect()
}

fn aberto_a_escolher() -> (MockPanelHost, AssetBrowserState) {
    publish();
    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    let mut st = AssetBrowserState::default();
    ph2d_panel_asset_browser::choose_image_for(SPRITE, "Hills".into());
    host.set_panel_visible(PANEL_ID, true);
    host.paint::<AssetBrowserPanel>(&mut st, VIEWPORT);
    (host, st)
}

/// **Mutações que devem sangrar:** o `kind` forçado a imagem no `paint_grid`; o braço do clique no
/// `event.rs`; o `entity_bits` do alvo trocado.
#[test]
fn o_seletor_so_oferece_imagens_e_um_clique_escolhe_para_a_sprite() {
    let (mut host, mut st) = aberto_a_escolher();
    let cartoes = painted();
    assert_eq!(
        cartoes.len(),
        2,
        "o seletor ofereceu o que não é imagem: {cartoes:?}"
    );
    assert!(
        cartoes
            .iter()
            .all(|k| matches!(k, AssetRef::Texture { .. }))
    );
    let AssetRef::Texture { asset } = cartoes[1] else {
        unreachable!()
    };
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::asset_cell_id(1)),
    );
    let sent = host.drained_actions();
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::ChooseImage { entity_bits, asset: a2 }
                if *entity_bits == SPRITE && *a2 == asset
        )),
        "o clique não escolheu a imagem para a sprite: {sent:?}"
    );
    assert!(!host.panel_visible(PANEL_ID), "o seletor não fechou");
    assert_eq!(st.choosing, None);
}

/// O `✕` da faixa desiste sem escolher; e sem alvo, um clique num cartão não escolhe nada (navegar).
#[test]
fn desistir_nao_escolhe_e_sem_alvo_o_clique_navega() {
    let (mut host, mut st) = aberto_a_escolher();
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ids::ASSET_CHOOSE_CANCEL),
    );
    assert!(!host.panel_visible(PANEL_ID));
    assert_eq!(st.choosing, None);
    assert!(
        !host
            .drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::ChooseImage { .. }))
    );
    // Reaberto sem pedido: a grade volta a ter tudo e o clique não escolhe.
    host.set_panel_visible(PANEL_ID, true);
    host.paint::<AssetBrowserPanel>(&mut st, VIEWPORT);
    assert_eq!(painted().len(), 3);
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::asset_cell_id(1)),
    );
    assert!(
        !host
            .drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::ChooseImage { .. }))
    );
}

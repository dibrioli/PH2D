//! A costura da janela GRID & BACKGROUND: um `WidgetEvent` real pelo `MockPanelHost` chega ao canal,
//! e o laço da família devolve as preferências do documento mudadas (o efeito OBSERVÁVEL).

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_panel_pixel_prefs::canal::Caixa;
use ph2d_panel_pixel_prefs::{PixelPrefsPanel, PixelPrefsPanelState, ids};
use ph2d_pixel_core::PixelPrefs;
use ph2d_pixel_render::prefs_canal as fio;
use ph2d_ui_testkit::MockPanelHost;

#[test]
fn o_clique_numa_caixa_chega_as_preferencias_do_documento() {
    let doc = PixelPrefs::default();
    let _ = fio::troca_em(Some(doc), None);
    let mut host = MockPanelHost::with_panel::<PixelPrefsPanel>();
    let mut st = PixelPrefsPanelState::default();
    let caixa = ids::caixa(Caixa::GridVisible);
    let r = host.apply_panel_event::<PixelPrefsPanel>(&mut st, WidgetEvent::Click(caixa));
    assert_eq!(
        r,
        EventOutcome::Consumed,
        "a caixa Visible da grade não chegou ao painel"
    );
    let novo = fio::troca_em(Some(doc), None).expect("o clique mudou o documento");
    assert_eq!(novo.grid.visible, !doc.grid.visible);
}

#[test]
fn os_cabecalhos_dos_cartoes_dobram() {
    let host = MockPanelHost::with_panel::<PixelPrefsPanel>();
    for s in [
        ids::SEC_SCOPE,
        ids::PREFS_SEC_GRID,
        ids::SEC_PIXEL_GRID,
        ids::SEC_CHECKER,
        ids::SEC_ONION,
    ] {
        assert!(
            host.store().is_collapsible_section(s),
            "um cabeçalho de cartão não dobra"
        );
    }
}

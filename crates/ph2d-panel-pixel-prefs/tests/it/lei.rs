//! A lei da janela, sem ecrã: o gesto → o pedido → o que a família recebe.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_panel_pixel_prefs::canal::{self, Caixa, Numero, PrefsRequest, Scope};
use ph2d_panel_pixel_prefs::ids;
use ph2d_pixel_core::PixelPrefs;

#[test]
fn cada_controlo_escreve_o_seu_campo_e_set_as_defaults_vai_aos_novos() {
    let dir = std::env::temp_dir().join(format!("ph2d-pixel-prefs-it-{}", std::process::id()));
    let f = dir.join("pixel-prefs.txt");
    let doc = PixelPrefs::default();
    assert_eq!(canal::troca_em(Some(doc), Some(&f)), None);
    // W da grade = 24 no documento: a família recebe as preferências novas.
    let mut p = doc;
    canal::poe_numero(&mut p, Numero::GridW, 24.0);
    canal::alterna(&mut p, Caixa::GridVisible);
    canal::alterna(&mut p, Caixa::CheckerZoom);
    canal::request(PrefsRequest::Set(Scope::Document, p));
    let novo = canal::troca_em(Some(doc), Some(&f)).expect("mudou");
    assert_eq!(
        (novo.grid.w, novo.grid.visible, novo.checker.apply_zoom),
        (24, true, false)
    );
    // Nada mudou: nada volta (o documento não fica «modificado» sem razão).
    assert_eq!(canal::troca_em(Some(novo), Some(&f)), None);
    // Set as Defaults grava o ficheiro dos novos, que o New Sprite lê.
    canal::request(PrefsRequest::SetAsDefaults);
    assert_eq!(canal::troca_em(Some(novo), Some(&f)), None);
    assert_eq!(canal::view().new, novo);
    assert_eq!(ph2d_pixel_render::prefs_view::load_defaults_from(&f), novo);
    // Reset do documento: os de fábrica.
    canal::request(PrefsRequest::Reset(Scope::Document));
    assert_eq!(
        canal::troca_em(Some(novo), Some(&f)),
        Some(PixelPrefs::default())
    );
    // As faixas: opacidade 0–255, tamanho ≥ 1.
    let mut q = doc;
    canal::poe_numero(&mut q, Numero::GridOpacity, 900.0);
    canal::poe_numero(&mut q, Numero::CheckerW, -3.0);
    assert_eq!((q.grid.opacity, q.checker.w), (255, 1));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn o_clique_numa_caixa_pede_as_preferencias_com_ela_alternada() {
    let p = PixelPrefs::default();
    let ev = WidgetEvent::Click(ids::caixa(Caixa::PixelVisible));
    let Some(PrefsRequest::Set(Scope::New, q)) =
        ph2d_panel_pixel_prefs::pedido(&ev, Scope::New, p, |_| None)
    else {
        panic!("pedido");
    };
    assert!(q.pixel_grid.visible);
    let ev = WidgetEvent::ValueChanged(ids::numero(Numero::GridH));
    let r = ph2d_panel_pixel_prefs::pedido(&ev, Scope::Document, p, |_| Some(5.0));
    assert!(matches!(r, Some(PrefsRequest::Set(Scope::Document, q)) if q.grid.h == 5));
}

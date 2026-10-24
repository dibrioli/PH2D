//! ⭐ **Fechar o seletor de cor desafixa-o, por QUALQUER porta** (auditoria 24/10). O pino vale para ESTA abertura;
//! antes só o «×» o tirava, e os ~15 outros fechos (a amostra clicada outra vez, a vista Layers, o Painter fechado,
//! o Inspector a trocar de entidade…) deixavam-no preso — a abertura seguinte, de outro dono, já nascia fixada e
//! um clique fora deixava de a fechar. A cura mora na porta única (`WidgetStore::set_picker_target(None)`); aqui
//! três fechos reais do painel, cada um com o controlo de que o pino estava posto.

use ph2d_editor_core::ids::PAINTER_COLOR_THUMB;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{
    PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn janela() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

/// O painel na vista do pincel, com a amostra da cor clicável.
fn painel() -> (MockPanelHost, PainterLayersPanelState, Rect) {
    set_current_brush(Some(PainterTool::default().brush_settings()));
    set_current_dock_shows_layers(false);
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let r = host
        .paint::<PainterLayersPanel>(&mut st, janela())
        .into_iter()
        .find(|(i, r)| *i == PAINTER_COLOR_THUMB && r.w > 0.0)
        .map(|(_, r)| r)
        .expect("a amostra da cor pintada");
    (host, st, r)
}

fn clica(host: &mut MockPanelHost, st: &mut PainterLayersPanelState, r: Rect) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
}

/// Aberto pela amostra e fixado (o controlo: o pino está mesmo posto).
fn aberto_e_fixado() -> (MockPanelHost, PainterLayersPanelState, Rect) {
    let (mut host, mut st, r) = painel();
    clica(&mut host, &mut st, r);
    assert_eq!(host.store().picker_target(), Some(PAINTER_COLOR_THUMB));
    host.store_mut().set_picker_pinned(true);
    assert!(host.store().picker_pinned(), "controlo: o pino não pegou");
    (host, st, r)
}

fn fechado_e_solto(host: &MockPanelHost, porta: &str) {
    assert_eq!(host.store().picker_target(), None, "{porta}: não fechou");
    assert!(
        !host.store().picker_pinned(),
        "{porta}: fechou e ficou fixado"
    );
}

#[test]
fn a_amostra_clicada_outra_vez_fecha_e_desafixa() {
    let (mut host, mut st, r) = aberto_e_fixado();
    clica(&mut host, &mut st, r);
    fechado_e_solto(&host, "a amostra");
    // A abertura seguinte nasce solta.
    clica(&mut host, &mut st, r);
    assert_eq!(host.store().picker_target(), Some(PAINTER_COLOR_THUMB));
    assert!(!host.store().picker_pinned(), "reabriu fixado");
}

#[test]
fn a_vista_layers_fecha_e_desafixa() {
    let (mut host, mut st, _) = aberto_e_fixado();
    set_current_dock_shows_layers(true);
    let _ = host.paint::<PainterLayersPanel>(&mut st, janela());
    fechado_e_solto(&host, "a vista Layers");
}

#[test]
fn o_painter_fechado_fecha_e_desafixa() {
    let (mut host, mut st, _) = aberto_e_fixado();
    host.paint_hidden::<PainterLayersPanel>(&mut st, janela());
    fechado_e_solto(&host, "o Painter fechado");
}
